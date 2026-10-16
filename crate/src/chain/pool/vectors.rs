//! The chain-conformance §12 sample and vectors.

use super::*;
use crate::genesis::{Id, hash32, token_of};

/// The chain-conformance §12 vectors as `conformance/chain.snap` entries
/// (`chain::pool@v1`), computed against the linked hemera. The test record
/// is `token("pussy")`, value 5, `sk = 0x01 × 32`, `rho = 0x02 × 32`,
/// `r = 0x03 × 32`, `data = 0x04 × 32`, no predicate; the spend names it as
/// its one input and a second record (value 4, `rho = 0x05`, `r = 0x06`)
/// as its one output, pays fee 1 and links `token("boot") → token("pussy")`.
pub fn vectors() -> Vec<(&'static str, Id)> {
    let (sk, record, record2, spend) = sample();
    vec![
        ("tags digest", tags_digest()),
        ("empty leaf", empty_leaf()),
        ("empty root depth 40", empty_root()),
        ("turnstile key pussy", turnstile_key(&token_of("pussy"))),
        ("record", hash32(&record.encode())),
        ("commitment", record.commitment()),
        ("nullifier", nullifier(&sk, &record.rho)),
        ("root of one leaf", root_of(&[record.commitment()])),
        ("root of two leaves", root_of(&[record.commitment(), record2.commitment()])),
        ("spend public", hash32(&spend.encode().expect("sample spend is canonical"))),
        ("spend id", spend.id().expect("sample spend is canonical")),
        ("spend public, withdrawal and note", hash32(&sample_withdrawal().encode().expect("canonical"))),
        ("block statement one spend", {
            let vk = vk_set_digest(&[]);
            block_statement_id(&block_statement(&[0x44; 32], 1, &vk, &[spend.id().unwrap()]))
        }),
    ]
}

/// The §12 sample: `(sk, input record, output record, spend)`.
pub fn sample() -> (Id, Record, Record, SpendPublic) {
    let sk = [0x01u8; 32];
    let owner = owner_of(&sk);
    let token = token_of("pussy");
    let record = Record { token, value: 5, owner, rho: [0x02; 32], r: [0x03; 32], data: [0x04; 32], predicate: [0; 32] };
    let record2 = Record { token, value: 4, owner, rho: [0x05; 32], r: [0x06; 32], data: [0x04; 32], predicate: [0; 32] };
    let spend = SpendPublic {
        chain: [0x44; 32],
        anchor: root_of(&[record.commitment()]),
        nullifiers: vec![nullifier(&sk, &record.rho)],
        commitments: vec![record2.commitment()],
        links: vec![Link { p: token_of("boot"), q: token, weight: 1, payout: owner }],
        withdrawal: None,
        fee_token: token,
        fee: 1,
        predicates: vec![[0; 32], [0; 32]],
        notes: vec![],
    };
    (sk, record, record2, spend)
}

/// The §12 second spend: the same input, an output of value 2
/// (`rho = 0x05`, `r = 0x06`), a withdrawal of 2 PUSSY to `0x77 × 32`,
/// fee 1, no link, and one 3-byte note `ab ab ab`.
pub fn sample_withdrawal() -> SpendPublic {
    let (_, _, record2, spend) = sample();
    let out = Record { value: 2, ..record2 };
    SpendPublic {
        commitments: vec![out.commitment()],
        links: vec![],
        withdrawal: Some(Withdrawal { to: [0x77; 32], token: token_of("pussy"), amount: 2 }),
        notes: vec![vec![0xab; 3]],
        ..spend
    }
}
