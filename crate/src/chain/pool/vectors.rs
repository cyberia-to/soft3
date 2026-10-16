//! The chain-conformance §12 sample and vectors.

use super::*;
use crate::genesis::{Id, hash32, token_of};

/// The chain id of the sample (`0x44 × 32`).
pub const SAMPLE_CHAIN: Id = [0x44; 32];
/// The deposit txid the sample input was created by (`0x33 × 32`).
pub const SAMPLE_DEPOSIT_TXID: Id = [0x33; 32];

/// The chain-conformance §12 vectors as `conformance/chain.snap` entries
/// (`chain::pool@v1`). A wide value is entered as `hemera(value)`, its
/// 32-byte fingerprint, because a snapshot line holds 32 bytes; §12 quotes
/// the 64-byte values themselves.
pub fn vectors() -> Vec<(&'static str, Id)> {
    let (sk, record, record2, spend) = sample();
    let fp = |w: &[u8]| hash32(w);
    let nf0 = nullifier(&SAMPLE_CHAIN, &sk, &record.rho);
    vec![
        ("tags digest", fp(&tags_digest())),
        ("genesis pool record", fp(&Pool::genesis().encode())),
        ("owner", fp(&owner_of(&sk))),
        ("rho of deposit", fp(&rho_of_deposit(&SAMPLE_DEPOSIT_TXID))),
        ("rho of output 0", fp(&rho_of_output(&nf0, 0))),
        ("predicate of the empty program", fp(&predicate_of(&[]))),
        ("empty leaf", fp(&empty_leaf())),
        ("node E1", fp(&empty_roots()[1])),
        ("empty root depth 40", fp(&empty_root())),
        ("turnstile key pussy", turnstile_key(&token_of("pussy"))),
        ("record", fp(&record.encode())),
        ("commitment", fp(&record.commitment())),
        ("nullifier", fp(&nf0)),
        ("output commitment", fp(&record2.commitment())),
        ("root of one leaf", fp(&root_of(&[record.commitment()]))),
        ("root of two leaves", fp(&root_of(&[record.commitment(), record2.commitment()]))),
        ("spend public", fp(&spend.encode().expect("sample spend is canonical"))),
        ("spend id", fp(&spend.id().expect("sample spend is canonical"))),
        ("spend public, withdrawal and note", fp(&sample_withdrawal().encode().expect("canonical"))),
        ("vk set of no keys", fp(&vk_set_digest(&[]))),
        ("block statement one spend", {
            let vk = vk_set_digest(&[]);
            fp(&block_statement_id(&block_statement(&SAMPLE_CHAIN, 1, &vk, &[spend.id().unwrap()])))
        }),
    ]
}

/// The §12 sample: `(sk, input record, output record, spend)`. The input
/// was deposited by txid `0x33 × 32` (its rho is that deposit's); the
/// output is output 0 of the spend (its rho derives from the spend's first
/// nullifier).
pub fn sample() -> (Id, Record, Record, SpendPublic) {
    let sk = [0x01u8; 32];
    let owner = owner_of(&sk);
    let token = token_of("pussy");
    let record = Record { token, value: 5, owner, rho: rho_of_deposit(&SAMPLE_DEPOSIT_TXID), r: [0x03; 32], data: [0x04; 32], predicate: [0; 64] };
    let nf0 = nullifier(&SAMPLE_CHAIN, &sk, &record.rho);
    let record2 = Record { value: 4, rho: rho_of_output(&nf0, 0), r: [0x06; 32], ..record };
    let spend = SpendPublic {
        chain: SAMPLE_CHAIN,
        anchor: root_of(&[record.commitment()]),
        nullifiers: vec![nf0],
        commitments: vec![record2.commitment()],
        links: vec![Link { p: token_of("boot"), q: token, weight: 1, payout: owner }],
        withdrawal: None,
        fee_token: token,
        fee: 1,
        predicates: vec![[0; 64], [0; 64]],
        notes: vec![],
    };
    (sk, record, record2, spend)
}

/// The §12 second spend: the same input, output 0 of value 2, a withdrawal
/// of 2 PUSSY to `0x77 × 32`, fee 1, no link, and one 3-byte note `ab ab ab`.
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
