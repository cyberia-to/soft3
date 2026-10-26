//! The conformance corpus (proven-present S0): the node writes every block
//! with its native verdict and R; the generator's invalid blocks each fail at
//! the step they target; a fresh chain replays the file to the same answers.

use std::collections::HashSet;
use std::path::PathBuf;

use super::*;
use crate::chain::api::{BlockVerdict, ChainError};
use crate::chain::apply::{BlockError, TxError};
use crate::chain::corpus::{self, CorpusWriter, invalid_blocks, record_accept, replay, verdict_code};
use crate::chain::header::HeaderError;

fn scratch(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("soft3-corpus-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn corpus_records_every_mutation_and_replays_on_a_fresh_chain() {
    let net = Net::new(3);
    let mut c = net.open("corpus-a");
    let dir = scratch("a");
    let mut w = CorpusWriter::open(&dir).unwrap();
    let miner = net.neurons[2];
    let (p, q) = (token("boot"), token("pussy"));
    let plan: Vec<Vec<Vec<u8>>> = vec![
        vec![],
        vec![net.transfer(0, 0, net.neurons[1], 500, 224)],
        vec![],
        vec![net.transfer(0, 1, net.neurons[1], 7, 224), net.transfer(1, 0, net.neurons[0], 9, 224)],
        vec![net.link(0, 2, p, q, 0, 1, 257)],
        vec![],
        vec![net.transfer(1, 1, net.neurons[2], 3, 224)],
    ];
    let mut names = HashSet::new();
    let mut expected_records = 1usize; // genesis
    for txs in &plan {
        let tpl = template(&mut c, miner, 0, txs);
        let valid = block_of(&tpl);
        let tip = c.tip();
        let bad = invalid_blocks(&c, &valid);
        assert!(bad.len() >= 12, "height {}: only {} mutations", valid.header.height, bad.len());
        for inv in &bad {
            let got = record_accept(&mut c, &mut w, &inv.block, NOW, Some(inv.name)).unwrap();
            assert_eq!(verdict_code(&got), inv.expected, "height {} mutation {}: {got:?}", valid.header.height, inv.name);
            assert_eq!(c.tip(), tip, "{} moved the tip", inv.name);
            assert_eq!(c.tip().header.state_root, tip.header.state_root, "{} moved R", inv.name);
            names.insert(inv.name);
            expected_records += 1;
        }
        let got = record_accept(&mut c, &mut w, &valid, NOW, None).unwrap();
        assert_eq!(got, Ok(BlockVerdict::Extended));
        assert_eq!(c.tip().id, valid.id());
        expected_records += 1;
    }
    for n in ["wrong_chain", "unknown_parent", "wrong_height", "time_not_after_mtp", "wrong_bits", "bad_pow", "tx_root", "state_root", "coinbase_amount", "bad_signature", "duplicate_tx", "replayed_tx", "coinbase_position"] {
        assert!(names.contains(n), "mutation {n} never produced");
    }
    assert_eq!(w.len(), expected_records);

    let text = std::fs::read_to_string(dir.join(corpus::FILE)).unwrap();
    assert_eq!(text.lines().count(), expected_records);
    let first: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    assert_eq!(first["verdict"], "genesis");
    assert_eq!(first["height"], 0);
    assert!(first["mutation"].is_null());

    // a fresh chain on the same genesis reproduces every verdict and root
    let mut fresh = net.open("corpus-b");
    assert_eq!(replay(&dir, &mut fresh, NOW), Ok(expected_records - 1));
    assert_eq!(fresh.tip(), c.tip());
    assert_eq!(fresh.tip().header.state_root, c.tip().header.state_root);

    // and a tampered file is caught at its line
    let tampered = text.replacen("\"verdict\":\"extended\"", "\"verdict\":\"stored\"", 1);
    let line = tampered.lines().position(|l| l.contains("\"verdict\":\"stored\"")).unwrap() + 1;
    let tdir = scratch("tampered");
    std::fs::create_dir_all(&tdir).unwrap();
    std::fs::write(tdir.join(corpus::FILE), tampered).unwrap();
    let mut third = net.open("corpus-c");
    assert_eq!(replay(&tdir, &mut third, NOW).unwrap_err().line, line);
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&tdir);
}

#[test]
fn verdict_codes_are_distinct_per_mutation_and_stable() {
    let net = Net::new(3);
    let mut c = net.open("corpus-codes");
    let miner = net.neurons[2];
    mine(&mut c, miner, 0, &[net.transfer(0, 0, net.neurons[1], 5, 224)]);
    let tpl = template(&mut c, miner, 0, &[net.transfer(0, 1, net.neurons[1], 5, 224)]);
    let valid = block_of(&tpl);
    let bad = invalid_blocks(&c, &valid);
    let expected: HashSet<_> = bad.iter().map(|b| b.expected.clone()).collect();
    assert_eq!(expected.len(), bad.len(), "two mutations share a code: {:?}", bad.iter().map(|b| (b.name, &b.expected)).collect::<Vec<_>>());
    let names: HashSet<_> = bad.iter().map(|b| b.name).collect();
    assert_eq!(names.len(), bad.len());
    let mut seen = HashSet::new();
    for inv in &bad {
        assert!(seen.insert(verdict_code(&c.accept_block(&inv.block, NOW))), "{} repeated a code", inv.name);
    }
    assert_eq!(seen, expected);

    assert_eq!(verdict_code(&Ok(BlockVerdict::Extended)), "extended");
    assert_eq!(verdict_code(&Ok(BlockVerdict::Reorganized { old_tip: [0; 32], depth: 3 })), "reorganized");
    assert_eq!(verdict_code(&Ok(BlockVerdict::Stored)), "stored");
    assert_eq!(verdict_code(&Ok(BlockVerdict::NeedBodies(vec![[1; 32]]))), "need_bodies");
    assert_eq!(verdict_code(&Ok(BlockVerdict::Known)), "known");
    assert_eq!(verdict_code(&Err(ChainError::Pow)), "reject:pow");
    assert_eq!(verdict_code(&Err(ChainError::Header(HeaderError::WrongChain))), "reject:header:wrong_chain");
    assert_eq!(verdict_code(&Err(ChainError::Block(BlockError::TxRoot))), "reject:block:tx_root");
    assert_eq!(
        verdict_code(&Err(ChainError::Block(BlockError::Tx { index: 4, error: TxError::Nonce { expected: 1, actual: 0 } }))),
        "reject:block:tx:nonce"
    );
}
