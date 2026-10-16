//! chain.md §11 / chain-conformance §12: the pool's formats.

use super::*;
use crate::genesis::token_of;

#[test]
fn wide_digest_extends_hemera_and_is_canonical() {
    for x in [&b""[..], b"pussy", &[7u8; 200][..]] {
        let w = wide(x);
        assert_eq!(&w[..32], hemera::hash(x).as_bytes(), "the first four limbs are hemera(x)");
        assert!(canonical(&w));
        assert_eq!(wide_secret(x), w, "constant-time variant, same digest");
    }
    assert!(!canonical(&[0xff; 8]));
    assert!(canonical(&(P - 1).to_le_bytes()));
    assert!(!canonical(&P.to_le_bytes()), "p aliases 0 and is refused");
}

#[test]
fn wide_digest_is_two_squeezes_of_the_xof() {
    let x = b"soft3/pool/test\0";
    let mut xof = {
        let mut h = hemera::Hasher::new();
        h.update(x);
        h.finalize_xof()
    };
    let mut first = [0u8; 32];
    let mut second = [0u8; 32];
    xof.fill(&mut first);
    xof.fill(&mut second);
    let w = wide(x);
    assert_eq!(&w[..32], &first);
    assert_eq!(&w[32..], &second);
}

#[test]
fn tree_node_is_the_tagged_wide_digest() {
    let (a, b) = (wide(b"a"), wide(b"b"));
    assert_eq!(node(&a, &b), wide(&[D_NODE, &a[..], &b[..]].concat()));
    assert_ne!(node(&a, &b), node(&b, &a), "order matters");
    assert!(canonical(&node(&a, &b)));
}

#[test]
fn tree_roots_and_paths() {
    let e = empty_roots();
    assert_eq!(e.len(), 41);
    assert_eq!(e[0], wide(D_EMPTY));
    assert_eq!(e[1], node(&e[0], &e[0]));
    assert_eq!(e[40], node(&e[39], &e[39]));
    assert_eq!(root_of(&[]), e[40]);
    let (_, record, record2, _) = sample();
    let (c0, c1) = (record.commitment(), record2.commitment());
    let mut path: Vec<Wide> = e[..40].to_vec();
    assert_eq!(root_from_path(&c0, 0, &path), Some(root_of(&[c0])));
    let root2 = root_of(&[c0, c1]);
    path[0] = c1;
    assert_eq!(root_from_path(&c0, 0, &path), Some(root2));
    path[0] = c0;
    assert_eq!(root_from_path(&c1, 1, &path), Some(root2), "bit 0 set: sibling on the left");
    assert_ne!(root_from_path(&c1, 0, &path), Some(root2), "wrong side");
    assert_eq!(root_from_path(&c1, 1 << 40, &path), None, "position out of range");
    assert_eq!(root_from_path(&c1, 1, &path[..39]), None);
    assert_ne!(root_of(&[c0, c1]), root_of(&[c1, c0]), "position matters");
}

#[test]
fn record_owner_rho_commitment_nullifier() {
    let (sk, record, record2, _) = sample();
    let bytes = record.encode();
    assert_eq!(bytes.len(), 296);
    assert_eq!(&bytes[..32], &token_of("pussy"));
    assert_eq!(&bytes[32..40], &5u64.to_le_bytes());
    assert_eq!(&bytes[40..104], &owner_of(&sk));
    assert_eq!(&bytes[104..168], &rho_of_deposit(&SAMPLE_DEPOSIT_TXID));
    assert_eq!(Record::decode(&bytes).unwrap(), record);
    assert!(Record::decode(&bytes[..295]).is_err());
    for at in [0, 40, 104, 168, 200, 232] {
        let mut bad = bytes;
        bad[at..at + 8].copy_from_slice(&P.to_le_bytes());
        assert!(Record::decode(&bad).is_err(), "limb >= p at {at}");
    }
    let mut big = bytes;
    big[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(Record::decode(&big).is_ok(), "a value is any u64");
    assert_eq!(record.commitment(), wide(&[D_CM, &bytes[..]].concat()));
    assert_eq!(owner_of(&sk), wide(&[D_OWNER, &sk[..]].concat()));
    let nf = nullifier(&SAMPLE_CHAIN, &sk, &record.rho);
    assert_eq!(nf, wide(&[D_NULL, &SAMPLE_CHAIN[..], &sk[..], &record.rho[..]].concat()));
    assert_ne!(nf, nullifier(&[0x45; 32], &sk, &record.rho), "the chain id is in the nullifier");
    assert_eq!(record2.rho, wide(&[D_RHO, &nf[..], &[0u8][..]].concat()));
    assert_ne!(rho_of_output(&nf, 0), rho_of_output(&nf, 1));
    assert_eq!(rho_of_deposit(&SAMPLE_DEPOSIT_TXID), wide(&[D_RHO, &SAMPLE_DEPOSIT_TXID[..], &[0u8][..]].concat()));
    assert_eq!(predicate_of(b"prog"), wide(&[D_PRED, &b"prog"[..]].concat()));
    assert_eq!(turnstile_key(&token_of("pussy")), crate::genesis::hash32(&[D_TURNSTILE, &token_of("pussy")[..]].concat()));
    assert_ne!(turnstile_key(&token_of("pussy")), bbg::state::balance_key(&[0; 32], &token_of("pussy")));
}

#[test]
fn spend_encoding_is_canonical_and_strict() {
    let (_, _, _, spend) = sample();
    let enc = spend.encode().unwrap();
    assert_eq!(enc.len(), 32 + 64 + 1 + 64 + 1 + 64 + 1 + 136 + 1 + 32 + 8 + 128 + 1);
    assert_eq!(enc.len(), 533);
    assert_eq!(SpendPublic::decode(&enc).unwrap(), spend);
    assert_eq!(spend.id().unwrap(), wide(&[D_SPEND, &enc[..]].concat()));
    let mut t = enc.clone();
    t.push(0);
    assert!(SpendPublic::decode(&t).is_err(), "trailing byte");
    assert!(SpendPublic::decode(&enc[..enc.len() - 1]).is_err(), "truncated");
    let mut n5 = enc.clone();
    n5[96] = 5;
    assert!(SpendPublic::decode(&n5).is_err(), "5 inputs");
    let mut n0 = enc.clone();
    n0[96] = 0;
    assert!(SpendPublic::decode(&n0).is_err(), "0 inputs");
    let mut anchor = enc.clone();
    anchor[32..40].copy_from_slice(&P.to_le_bytes());
    assert!(SpendPublic::decode(&anchor).is_err(), "anchor limb >= p");
    let bad = |f: &dyn Fn(&mut SpendPublic), why: &str| {
        let mut s = spend.clone();
        f(&mut s);
        assert!(s.encode().is_err(), "{why}");
    };
    bad(&|s| s.nullifiers.push(s.nullifiers[0]), "duplicate nullifier");
    bad(&|s| {
        s.commitments.push(s.commitments[0]);
        s.predicates.push([0; 64]);
    }, "duplicate commitment");
    bad(&|s| s.links[0].q = s.links[0].p, "p == q");
    bad(&|s| s.notes = vec![vec![0; MAX_NOTE_BYTES + 1]], "note over the cap");
    bad(&|s| s.notes = vec![vec![1], vec![2]], "more notes than outputs");
    bad(&|s| s.predicates.pop().map(|_| ()).unwrap_or(()), "predicate count");
    bad(&|s| s.withdrawal = Some(Withdrawal { to: [0x77; 32], token: token_of("pussy"), amount: 0 }), "withdrawal of 0");
    bad(&|s| s.links[0].payout[..8].copy_from_slice(&u64::MAX.to_le_bytes()), "payout limb >= p");
    bad(&|s| s.nullifiers = vec![], "no input");
    let w = sample_withdrawal();
    let enc2 = w.encode().unwrap();
    assert_eq!(enc2.len(), 533 - 136 + 72 + 2 + 3);
    assert_eq!(SpendPublic::decode(&enc2).unwrap(), w);
    let mut flag = enc2.clone();
    flag[32 + 64 + 1 + 64 + 1 + 64 + 1] = 2;
    assert!(SpendPublic::decode(&flag).is_err(), "withdrawal flag 2");
    assert_eq!((DEPOSIT_BODY_BYTES, DEPOSIT_BYTES), (280, 384));
    assert_eq!((MAX_SPEND_BYTES, MAX_SPEND_TX_BYTES), (9_981, 9_985));
    let full = SpendPublic {
        nullifiers: (1..=4).map(|i| [i; 64]).collect(),
        commitments: (5..=8).map(|i| [i; 64]).collect(),
        links: (0..4).map(|i| Link { p: [0x10 + i; 32], q: [0x20 + i; 32], weight: 1, payout: [0x30; 64] }).collect(),
        withdrawal: Some(Withdrawal { to: [0x77; 32], token: token_of("pussy"), amount: 1 }),
        predicates: vec![[0; 64]; 8],
        notes: vec![vec![0; MAX_NOTE_BYTES]; 4],
        ..spend.clone()
    };
    assert_eq!(full.encode().unwrap().len(), MAX_SPEND_BYTES);
}

