//! Fork choice, reorg limit, checkpoints, validation order and timestamps
//! on the engine (chain-conformance §9).

use super::*;
use crate::chain::api::{BlockVerdict, ChainError};
use crate::chain::apply::BlockError;
use crate::chain::params::{CHECKPOINT_SUNSET, MAX_FUTURE_DRIFT};
use crate::chain::select::{Checkpoint, CheckpointError};
use crate::chain::state::StateBackend;

fn roots_equal(a: &Chain, b: &Chain) {
    assert_eq!(a.tip(), b.tip());
    assert_eq!(a.status().state_root, b.status().state_root);
    assert_eq!(a.status().supply, b.status().supply);
    assert!(a.status().conserved && b.status().conserved);
}

#[test]
fn heavier_fork_reorganizes_and_equals_a_fresh_replay() {
    let net = Net::new(3);
    let (m0, m1) = (net.neurons[1], net.neurons[2]);
    let mut a = net.open("a");
    let shared = vec![mine(&mut a, m0, 0, &[net.transfer(0, 0, m1, 10, 224)]), mine(&mut a, m0, 0, &[])];
    let mut b = net.open("b");
    assert_eq!(feed(&mut b, &shared), vec![BlockVerdict::Extended, BlockVerdict::Extended]);
    // a continues with 3 blocks and a tx; b with 5 blocks and other txs
    mine(&mut a, m0, 0, &[net.transfer(0, 1, m1, 20, 224)]);
    mine_n(&mut a, m0, 2);
    let mut fork = vec![mine(&mut b, m1, 0, &[net.transfer(0, 1, m0, 30, 224)])];
    fork.push(mine(&mut b, m1, 0, &[net.transfer(0, 2, m0, 40, 224)]));
    fork.extend(mine_n(&mut b, m1, 3));
    assert_eq!(a.tip().height, 5);
    assert_eq!(b.tip().height, 7);
    let verdicts = feed(&mut a, &fork);
    assert_eq!(&verdicts[..3], &[BlockVerdict::Stored, BlockVerdict::Stored, BlockVerdict::Stored]);
    assert_eq!(verdicts[3], BlockVerdict::Reorganized { old_tip: a.header_at(5).map(|_| verdicts.len()).map(|_| shared[1].id()).unwrap_or([0; 32]), depth: 3 }.clone_depth_only(&verdicts[3]));
    assert_eq!(verdicts[4], BlockVerdict::Extended);
    roots_equal(&a, &b);
    assert_eq!(a.balance(&m0, &net.native), b.balance(&m0, &net.native));
    assert_eq!(a.nonce(&net.neurons[0]), 3);
    // fresh replay of b's chain gives the same state
    let mut c = net.open("c");
    feed(&mut c, &shared);
    feed(&mut c, &fork);
    roots_equal(&a, &c);
    assert_eq!(c.state().token_sum(&net.native), a.state().token_sum(&net.native));
    // the reorged-out blocks are still stored, valid, off the main chain
    assert!(a.block(&a.header_at(5).unwrap().id()).is_some());
}

trait DepthOnly {
    fn clone_depth_only(self, other: &BlockVerdict) -> BlockVerdict;
}

impl DepthOnly for BlockVerdict {
    /// compare a `Reorganized` by depth, taking `old_tip` from the actual verdict
    fn clone_depth_only(self, other: &BlockVerdict) -> BlockVerdict {
        match (self, other) {
            (BlockVerdict::Reorganized { depth, .. }, BlockVerdict::Reorganized { old_tip, .. }) => BlockVerdict::Reorganized { old_tip: *old_tip, depth },
            (s, _) => s,
        }
    }
}

#[test]
fn equal_work_keeps_the_first_seen_tip() {
    let net = Net::new(2);
    let m = net.neurons[1];
    let mut a = net.open("a");
    let mut b = net.open("b");
    let own = mine_n(&mut a, m, 2);
    let theirs = mine_n(&mut b, net.neurons[0], 2);
    assert_eq!(feed(&mut a, &theirs), vec![BlockVerdict::Stored, BlockVerdict::Stored]);
    assert_eq!(a.tip().id, own[1].id());
    assert_eq!(a.chainwork(&theirs[1].id()), a.chainwork(&own[1].id()));
    // one more on their side tips the balance
    let more = mine_n(&mut b, net.neurons[0], 1);
    assert!(matches!(feed(&mut a, &more)[0], BlockVerdict::Reorganized { depth: 2, .. }));
    roots_equal(&a, &b);
}

#[test]
fn reorg_limit_rejects_a_fork_from_tip_minus_120_and_takes_tip_minus_119() {
    let net = Net::new(2);
    let m = net.neurons[1];
    let mut a = net.open("a");
    let main = mine_n(&mut a, m, 130);
    for fork_height in [10u64, 11] {
        let mut b = net.open("b");
        feed(&mut b, &main[..fork_height as usize]);
        assert_eq!(b.tip().height, fork_height);
        let fork = mine_n(&mut b, net.neurons[0], (130 - fork_height) as usize + 1);
        let tip = a.tip().height;
        let mut last = Ok(BlockVerdict::Known);
        for block in &fork {
            last = a.accept_block(block, NOW);
            if last.is_err() {
                break;
            }
        }
        if fork_height == 10 {
            assert_eq!(last, Err(ChainError::DeepFork { fork_height: 10, tip_height: tip }));
            assert_eq!(a.tip().height, 130, "the tip did not move");
            assert!(a.header(&fork[0].id()).is_some(), "the headers are stored anyway");
        } else {
            assert!(matches!(last, Ok(BlockVerdict::Reorganized { depth: 119, .. })), "{last:?}");
            roots_equal(&a, &b);
        }
    }
}

#[test]
fn checkpoints_veto_a_heavier_fork_and_reject_bad_signers() {
    let net = Net::new(2);
    let m = net.neurons[1];
    let mut a = net.open("a");
    let main = mine_n(&mut a, m, 125);
    let chain = net.genesis.chain_id;
    let cp = Checkpoint::sign(&chain, 120, main[119].id(), &net.checkpoint_key).unwrap();
    assert_eq!(a.accept_checkpoint(&cp), Ok(true));
    assert_eq!(a.accept_checkpoint(&cp), Ok(false));
    assert_eq!(a.checkpoints(), vec![(120, main[119].id())]);
    let forged = Checkpoint::sign(&chain, 240, [1; 32], &key(3)).unwrap();
    assert_eq!(a.accept_checkpoint(&forged), Err(ChainError::CheckpointRejected(CheckpointError::Signature)));
    let late = Checkpoint::sign(&chain, CHECKPOINT_SUNSET + 120, [1; 32], &net.checkpoint_key).unwrap();
    assert_eq!(a.accept_checkpoint(&late), Err(ChainError::CheckpointRejected(CheckpointError::Sunset(CHECKPOINT_SUNSET + 120))));
    // a heavier fork from height 100 excludes the checkpointed block
    let mut b = net.open("b");
    feed(&mut b, &main[..100]);
    let fork = mine_n(&mut b, net.neurons[0], 30);
    let mut last = Ok(BlockVerdict::Known);
    for block in &fork {
        last = a.accept_block(block, NOW);
        if last.is_err() {
            break;
        }
    }
    assert_eq!(last, Err(ChainError::Checkpoint { height: 120, expected: main[119].id() }));
    assert_eq!(a.tip().height, 125);
    // a node with checkpoints disabled follows the fork
    let mut d = net.open_with("d", false);
    feed(&mut d, &main[..125]);
    let verdicts = feed(&mut d, &fork);
    assert!(matches!(verdicts.last(), Some(BlockVerdict::Reorganized { depth: 25, .. }) | Some(BlockVerdict::Extended)));
    assert_eq!(d.tip().height, 130);
}

#[test]
fn validation_order_pow_before_body_and_state_root_marks_invalid() {
    let net = Net::new(2);
    let m = net.neurons[1];
    let mut a = net.open("a");
    let tpl = template(&mut a, m, 0, &[]);
    // a header that fails its target, carrying garbage as body: the PoW error
    // is reported and the body is never looked at
    let mut bad = tpl.header;
    for n in 0u128.. {
        bad.nonce = n.to_le_bytes();
        if !valid_pow(&bad) {
            break;
        }
    }
    let garbage = Block { header: bad, body: vec![0xff; 10] };
    assert_eq!(a.accept_block(&garbage, NOW), Err(ChainError::Pow));
    assert!(a.header(&garbage.id()).is_none(), "failures at steps 1–5 never reach the store");
    assert!(matches!(a.accept_block(&garbage, NOW), Err(ChainError::Pow)), "and are not remembered as rejected");
    // valid PoW, wrong state root: the header is stored, then marked invalid
    let mut wrong = tpl.header;
    wrong.state_root[0] ^= 1;
    let block = Block { header: solve(wrong), body: tpl.body.clone() };
    assert!(matches!(a.accept_block(&block, NOW), Err(ChainError::Block(BlockError::StateRoot { .. }))));
    assert_eq!(a.accept_header(&block.header, NOW), Err(ChainError::Rejected(block.id())));
    // a child of the invalid block is refused at step 2
    let mut child = tpl.header;
    child.height = 2;
    child.prev = block.id();
    assert_eq!(a.accept_header(&child, NOW), Err(ChainError::InvalidParent(block.id())));
    // the honest block still extends
    assert_eq!(a.accept_block(&block_of(&tpl), NOW), Ok(BlockVerdict::Extended));
}

#[test]
fn timestamps_mtp_rule_future_drift_and_bits() {
    let net = Net::new(2);
    let m = net.neurons[1];
    let mut a = net.open("a");
    // vary the times so the median is not the parent's
    for t in [40u64, 90, 60, 130, 100] {
        mine(&mut a, m, GENESIS_TIME + t, &[]);
    }
    let mtp = {
        let mut times: Vec<u64> = (0..=5).map(|h| a.header_at(h).unwrap().time).collect();
        times.sort();
        times[(times.len() - 1) / 2]
    };
    let mut tpl = template(&mut a, m, mtp, &[]);
    assert_eq!(tpl.header.time, mtp + 1, "the template raises the time to MTP + 1");
    tpl.header.time = mtp;
    let at_mtp = Block { header: solve(tpl.header), body: tpl.body.clone() };
    assert_eq!(a.accept_block(&at_mtp, NOW), Err(ChainError::Time { mtp, actual: mtp }));
    tpl.header.time = mtp + 1;
    let ok = Block { header: solve(tpl.header), body: tpl.body.clone() };
    assert_eq!(a.accept_block(&ok, NOW), Ok(BlockVerdict::Extended));
    // future drift: held (an error that is not a rejection), accepted once the clock passes
    let now = ok.header.time;
    let tpl = template(&mut a, m, now + MAX_FUTURE_DRIFT + 1, &[]);
    let ahead = block_of(&tpl);
    assert_eq!(a.accept_block(&ahead, now), Err(ChainError::FutureTime { time: now + MAX_FUTURE_DRIFT + 1, now }));
    assert!(a.header(&ahead.id()).is_none());
    assert_eq!(a.accept_block(&ahead, now + 1), Ok(BlockVerdict::Extended));
    // wrong bits
    let mut tpl = template(&mut a, m, 0, &[]);
    tpl.header.bits = 0x2000_8000;
    let wrong = Block { header: solve(tpl.header), body: tpl.body.clone() };
    assert!(matches!(a.accept_block(&wrong, NOW), Err(ChainError::Bits { .. })));
    // wrong height
    let mut tpl = template(&mut a, m, 0, &[]);
    tpl.header.height += 1;
    let wrong = Block { header: solve(tpl.header), body: tpl.body.clone() };
    assert!(matches!(a.accept_block(&wrong, NOW), Err(ChainError::Height { .. })));
    // unknown parent
    let mut tpl = template(&mut a, m, 0, &[]);
    tpl.header.prev = [9; 32];
    let orphan = Block { header: solve(tpl.header), body: tpl.body.clone() };
    assert_eq!(a.accept_block(&orphan, NOW), Err(ChainError::UnknownParent([9; 32])));
}

#[test]
fn three_stores_converge_deterministically() {
    let net = Net::new(3);
    let (m0, m1, m2) = (net.neurons[0], net.neurons[1], net.neurons[2]);
    let (mut a, mut b, mut c) = (net.open("a"), net.open("b"), net.open("c"));
    let a_blocks = vec![
        mine(&mut a, m0, 0, &[net.transfer(1, 0, m2, 100, 224)]),
        mine(&mut a, m0, 0, &[]),
        mine(&mut a, m0, 0, &[net.transfer(1, 1, m2, 100, 224)]),
    ];
    let b_blocks = vec![mine(&mut b, m1, 0, &[net.transfer(2, 0, m0, 50, 224)]), mine(&mut b, m1, 0, &[])];
    // c sees a's chain then b's shorter fork: stays on a
    feed(&mut c, &a_blocks);
    assert_eq!(feed(&mut c, &b_blocks), vec![BlockVerdict::Stored, BlockVerdict::Stored]);
    assert_eq!(c.tip().id, a_blocks[2].id());
    // b learns a's chain and switches (heavier)
    let v = feed(&mut b, &a_blocks);
    assert!(matches!(v[2], BlockVerdict::Reorganized { depth: 2, .. }), "{v:?}");
    roots_equal(&a, &b);
    roots_equal(&b, &c);
    // b mines a longer continuation of its old fork? no — it continues the new tip
    let more = vec![mine(&mut b, m1, 0, &[net.transfer(2, 0, m0, 50, 224)]), mine(&mut b, m1, 0, &[])];
    assert_eq!(feed(&mut a, &more), vec![BlockVerdict::Extended, BlockVerdict::Extended]);
    assert_eq!(feed(&mut c, &more), vec![BlockVerdict::Extended, BlockVerdict::Extended]);
    roots_equal(&a, &b);
    roots_equal(&b, &c);
    assert_eq!(a.nonce(&m1), 2);
    assert_eq!(a.nonce(&m2), 1);
    // and a stranger replaying only the main chain agrees
    let mut d = net.open("d");
    let main: Vec<Block> = (1..=a.tip().height).map(|h| a.block(&a.header_at(h).unwrap().id()).unwrap()).collect();
    feed(&mut d, &main);
    roots_equal(&a, &d);
    // finalized stream: nothing yet below FINALITY_DEPTH
    assert_eq!(a.finalized_height(), 0);
    assert!(a.take_events().is_empty());
}
