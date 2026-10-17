//! The adapter on a private test genesis (target 2^255, chain-conformance
//! §8–§9): the two codecs agree byte for byte, and the wire's verbs —
//! header-first acceptance, NeedBodies, reorg events, held headers, the
//! txid index — behave on the real kernel.

use std::path::PathBuf;

use super::*;
use crate::chain::net::codec;
use crate::chain::tests::{solve, Net, NOW};

/// A kernel chain behind the port in a scratch directory.
pub(crate) struct Scratch {
    pub chain: KernelChain,
    dir: PathBuf,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub(crate) fn scratch(net: &Net, label: &str) -> Scratch {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("soft3-kernel-view-{label}-{}-{seq}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    Scratch { chain: KernelChain::open(&net.genesis.bytes, &dir).unwrap(), dir }
}

/// Template on the tip at `time`, solved (about two hashes at 2^255).
pub(crate) fn next_block(chain: &KernelChain, to: &Id, time: u64, txs: &[Vec<u8>]) -> (Header, Vec<u8>) {
    let (h, body) = chain.template(to, txs, time).unwrap();
    (from_kernel(&solve(to_kernel(&h))), body)
}

pub(crate) fn mine(chain: &KernelChain, to: &Id, txs: &[Vec<u8>]) -> (Header, Vec<u8>) {
    let time = chain.header_at(chain.tip().height).unwrap().time + 30;
    let (h, b) = next_block(chain, to, time, txs);
    assert_eq!(chain.accept_block(&h, &b, NOW), Ok(BlockVerdict::NewTip));
    (h, b)
}

#[test]
fn the_wire_codec_and_the_kernel_agree() {
    let net = Net::new(2);
    let s = scratch(&net, "agree");
    let tx = net.transfer(0, 0, [7; 32], 5, 300);
    let (h, body) = next_block(&s.chain, &net.neurons[1], NOW - 10, &[tx.clone()]);
    let k = to_kernel(&h);
    assert_eq!(h.id(), k.id());
    assert_eq!(codec::Header::decode(&k.encode()).unwrap(), h);
    let chain_id = s.chain.chain_id();
    assert_eq!(codec::txid(&chain_id, &tx), crate::chain::sigs::txid(&chain_id, &tx));
    assert_eq!(codec::decode_body(&body).unwrap(), ktx::split_body(&body).unwrap());
    assert_eq!(codec::tx_root(&s.chain.txs_of(&body)), k.tx_root);
    assert_eq!(codec::token("pussy"), s.chain.native_token());
    let wire = codec::Tx::decode(&tx).unwrap();
    assert!(wire.verify(&chain_id));
    assert_eq!(wire.encode(), tx);
}

#[test]
fn header_first_then_bodies_then_a_reorg_with_events() {
    let net = Net::new(2);
    let (a, b) = (scratch(&net, "a"), scratch(&net, "b"));
    let mut events = b.chain.events();
    let tx = net.transfer(0, 0, [7; 32], 5, 300);
    let blocks: Vec<_> = (0..3).map(|i| mine(&a.chain, &net.neurons[1], if i == 1 { std::slice::from_ref(&tx) } else { &[] })).collect();
    // headers first: b learns the best header chain, the bodies lag
    for (h, _) in &blocks {
        assert_eq!(b.chain.accept_header(h, NOW), Ok(HeaderVerdict::Accepted));
    }
    assert_eq!(b.chain.best_header().id, blocks[2].0.id());
    assert_eq!(b.chain.tip().height, 0);
    assert_eq!(b.chain.missing_bodies(10).iter().map(|m| m.0).collect::<Vec<_>>(), vec![1, 2, 3]);
    // the top body first: the kernel names the bodies it lacks
    let need = b.chain.accept_block(&blocks[2].0, &blocks[2].1, NOW).unwrap();
    assert_eq!(need, BlockVerdict::NeedBodies(vec![blocks[0].0.id(), blocks[1].0.id()]));
    for (h, body) in &blocks[..2] {
        assert!(matches!(b.chain.accept_block(h, body, NOW), Ok(BlockVerdict::NewTip)));
    }
    assert_eq!(b.chain.tip().id, blocks[2].0.id(), "the stored top block was adopted once complete");
    assert!(b.chain.missing_bodies(10).is_empty());
    let txid = codec::txid(&b.chain.chain_id(), &tx);
    assert_eq!(b.chain.tx_location(&txid), Some((2, blocks[1].0.id(), 1)));
    let mut tips = Vec::new();
    while let Ok(ChainEvent::NewTip { height, .. }) = events.try_recv() {
        tips.push(height);
    }
    assert_eq!(tips.last(), Some(&3));

    // a heavier fork from height 1 on a third chain; b reorganizes onto it
    let c = scratch(&net, "c");
    assert!(matches!(c.chain.accept_block(&blocks[0].0, &blocks[0].1, NOW), Ok(BlockVerdict::NewTip)));
    let fork: Vec<_> = (0..3).map(|_| mine(&c.chain, &net.neurons[0], &[])).collect();
    for (h, body) in &fork {
        let _ = b.chain.accept_block(h, body, NOW).unwrap();
    }
    assert_eq!(b.chain.tip().id, fork[2].0.id());
    assert_eq!(b.chain.tip().height, 4);
    let mut out = None;
    while let Ok(ChainEvent::NewTip { reorged_out, .. }) = events.try_recv() {
        if !reorged_out.is_empty() {
            out = Some(reorged_out);
        }
    }
    assert_eq!(out.expect("a reorg event"), vec![blocks[1].1.clone(), blocks[2].1.clone()]);
    assert_eq!(b.chain.tx_location(&txid), None, "the reorged-out tx left the index");
    assert!(b.chain.conserved());
}

#[test]
fn future_headers_are_held_then_retried_and_bad_blocks_are_invalid() {
    let net = Net::new(2);
    let (a, b) = (scratch(&net, "ha"), scratch(&net, "hb"));
    let now = 10_000;
    let (h, body) = next_block(&a.chain, &net.neurons[0], now + 500, &[]);
    assert_eq!(b.chain.accept_header(&h, now), Ok(HeaderVerdict::Held));
    assert!(b.chain.retry_held(now).is_empty());
    assert_eq!(b.chain.retry_held(now + 400), vec![h]);
    assert!(matches!(b.chain.accept_block(&h, &body, now + 400), Ok(BlockVerdict::NewTip)));

    let (h2, mut body2) = next_block(&b.chain, &net.neurons[0], now + 430, &[]);
    *body2.last_mut().unwrap() ^= 1;
    let e = b.chain.accept_block(&h2, &body2, now + 430).unwrap_err();
    assert!(matches!(e, BlockError::Invalid(_)), "{e:?}");
    assert_eq!(e.score(), 10);
    let orphan = Header { prev: [9; 32], ..h2 };
    assert_eq!(b.chain.accept_header(&orphan, now + 430), Ok(HeaderVerdict::Orphan));
    let bad_pow = Header { nonce: [0xff; 16], ..h2 };
    let bad_pow = if crate::chain::pow::valid_pow(&to_kernel(&bad_pow)) { Header { nonce: [0xfe; 16], ..h2 } } else { bad_pow };
    if !crate::chain::pow::valid_pow(&to_kernel(&bad_pow)) {
        assert_eq!(b.chain.accept_header(&bad_pow, now + 430), Err(HeaderError::BadPow));
    }
}

/// A heavier chain the reorg limit refuses must not capture body sync
/// (chain-wire §3.2 downloads along the best header chain). Found by the
/// chain simulator (simulation audit/network-2026-10.md, `reorg-140`): once
/// a private fork more than 119 deep is released, every honest node's
/// most-work header is the refused chain; a node whose tip sits on a stale
/// honest branch then asks only for the refused chain's bodies, never for
/// the honest best chain's, and stalls until a block happens to extend its
/// own tip.
#[test]
fn a_refused_deep_fork_does_not_capture_body_sync() {
    use crate::chain::params::MAX_REORG;
    let net = Net::new(2);
    let (a, c, b) = (scratch(&net, "deep-a"), scratch(&net, "deep-c"), scratch(&net, "deep-b"));
    let honest: Vec<_> = (0..MAX_REORG + 6).map(|_| mine(&a.chain, &net.neurons[0], &[])).collect();
    // the attacker forks at genesis and mines a longer (heavier) private chain
    let attack: Vec<_> = (0..MAX_REORG + 12).map(|_| mine(&c.chain, &net.neurons[1], &[])).collect();
    for (h, body) in &attack {
        let _ = a.chain.accept_block(h, body, NOW);
    }
    assert_eq!(a.chain.tip().id, honest.last().unwrap().0.id(), "the deep rewrite is refused");
    // an honest competitor replaces a's last block and extends: b's branch is the best chain a can take
    for (h, body) in &honest[..honest.len() - 1] {
        assert!(matches!(b.chain.accept_block(h, body, NOW), Ok(BlockVerdict::NewTip)));
    }
    let branch: Vec<_> = (0..2).map(|_| mine(&b.chain, &net.neurons[1], &[])).collect();
    for (h, _) in &branch {
        assert_eq!(a.chain.accept_header(h, NOW), Ok(HeaderVerdict::Accepted));
    }
    // a's sync must fetch the honest branch's bodies, not the refused chain's
    let missing: Vec<Id> = a.chain.missing_bodies(512).into_iter().map(|m| m.1).collect();
    assert_eq!(missing, branch.iter().map(|(h, _)| h.id()).collect::<Vec<_>>(), "body sync follows the best chain a can move to");
    assert_eq!(a.chain.best_header().id, branch[1].0.id());
    for (h, body) in &branch {
        let _ = a.chain.accept_block(h, body, NOW).unwrap();
    }
    assert_eq!(a.chain.tip().id, branch[1].0.id(), "a reorganizes onto the honest branch");
}

/// chain.md §5.3: a checkpoint forbids reorganizing *below* it. A node whose
/// tip is still under the checkpoint is not reorganizing when it moves up
/// through it; it must download that chain. Found by the chain simulator
/// (`late-join`): a late joiner that fetched the foundation checkpoint
/// before its headers never asked for a body.
#[test]
fn a_fresh_node_holding_a_checkpoint_downloads_through_it() {
    let net = Net::new(2);
    let (a, b) = (scratch(&net, "cp-a"), scratch(&net, "cp-b"));
    let blocks: Vec<_> = (0..130).map(|_| mine(&a.chain, &net.neurons[0], &[])).collect();
    let id = blocks[119].0.id();
    let cp = crate::chain::select::Checkpoint::sign(&a.chain.chain_id(), 120, id, &net.checkpoint_key).unwrap();
    assert_eq!(b.chain.accept_checkpoint(120, &id, &cp.sig), Ok(true));
    for (h, _) in &blocks {
        assert_eq!(b.chain.accept_header(h, NOW), Ok(HeaderVerdict::Accepted));
    }
    assert_eq!(b.chain.missing_bodies(512).len(), 130, "the checkpointed chain is the one to download");
    let top = blocks.last().unwrap();
    assert!(matches!(b.chain.accept_block(&top.0, &top.1, NOW), Ok(BlockVerdict::NeedBodies(_))), "the top body first names its ancestors");
    for (h, body) in &blocks {
        let _ = b.chain.accept_block(h, body, NOW).unwrap();
    }
    assert_eq!(b.chain.tip().id, top.0.id());
}
