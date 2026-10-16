//! chain.md §11 / chain-conformance §12: the pool's formats.

use super::*;
use crate::genesis::{hex, token_of};

fn hx(b: &[u8]) -> String {
    hex(b)
}

#[test]
fn wide_digest_is_the_whole_rate_of_one_finalization() {
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
fn wide_node_is_two_permutations_and_not_the_narrow_node() {
    let (a, b) = (wide(b"a"), wide(b"b"));
    // by hand: flags and length in the capacity, left, permute, right, permute
    let mut s = [Goldilocks::new(0); 16];
    s[9] = Goldilocks::new(2);
    s[10] = Goldilocks::new(16);
    for child in [&a, &b] {
        for i in 0..8 {
            s[i] = s[i] + Goldilocks::new(u64::from_le_bytes(child[i * 8..i * 8 + 8].try_into().unwrap()));
        }
        hemera::permutation::permute(&mut s);
    }
    assert_eq!(node(&a, &b, false), rate_bytes(&s));
    assert_ne!(node(&a, &b, false), node(&b, &a, false), "order matters");
    assert_ne!(node(&a, &b, false), node(&a, &b, true), "the root flag separates");
    let narrow = hemera::tree::hash_node(&hemera::Hash::from_bytes(a[..32].try_into().unwrap()), &hemera::Hash::from_bytes(b[..32].try_into().unwrap()), false);
    assert_ne!(&node(&a, &b, false)[..32], narrow.as_bytes());
}

#[test]
fn tree_roots_and_paths() {
    let e = empty_roots();
    assert_eq!(e.len(), 41);
    assert_eq!(e[0], wide(D_EMPTY));
    assert_eq!(e[1], node(&e[0], &e[0], false));
    assert_eq!(e[40], node(&e[39], &e[39], true));
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

/// The 64-byte values chain-conformance §12 quotes.
#[test]
fn section_12_values() {
    let (sk, record, record2, spend) = sample();
    let nf0 = nullifier(&SAMPLE_CHAIN, &sk, &record.rho);
    let stmt = block_statement_id(&block_statement(&SAMPLE_CHAIN, 1, &vk_set_digest(&[]), &[spend.id().unwrap()]));
    let e = empty_roots();
    for (got, want) in [
        (hx(&tags_digest()), "9ecbb70a03e0db94653d96e7a23a3684cc9a43b78592b7bdd1afbd94afbc3c15882a865b62acb7492544f0e8a59c9397c33e9853ef5ac0b89ff0377f8fd734d0"),
        (hx(&Pool::genesis().encode()), "0100000028e8030000409ecbb70a03e0db94653d96e7a23a3684cc9a43b78592b7bdd1afbd94afbc3c15882a865b62acb7492544f0e8a59c9397c33e9853ef5ac0b89ff0377f8fd734d0"),
        (hx(&owner_of(&sk)), "4424760142c6837ead0542693ce6ad7d827c8e2c836bea4fa5bab44f7c22c192b978e845e5c1219f6fee304ed47a87c6ad3a346e7f1143daaea29ef3d1c00bde"),
        (hx(&record.rho), "b0558434e4c530cb32f74f2716538f3a4018f687f3485a5cb87baac7519a6278c8a4969f892fe7ca56f7eb77fad1bed44ebe26d48f8f4b78a587417ecca41c08"),
        (hx(&nf0), "eb97b0b124d3a6661de612cdbff2ca7f15f2caa36516a5e71421c10fd94d2fa50c8e0e6b44043394fde12c2c9651be839797ec82c2683415081940149549501c"),
        (hx(&record2.rho), "0bf55e96541553a99343c89533f1cb5ba822ddbf65870e2342f014d7e95b6d6421226be82763c54e940a0b1e4d3e07dc4191667c9249c4fc8adf4a9212a79879"),
        (hx(&e[0]), "ddba4580f44954d39ca84d40621f45c67865e630f98367b4afe012335af4d697259e100146589dd16eb0ed611797ccb79fe81a7badc7fdda09c18f87a5b49887"),
        (hx(&e[1]), "3fbbafe208f8321e1c3ff25b545672b94ffd75b16e582553b07671d69e1c93fe89641d4dc38f7a47241686809694c52da9ea6873275b1b78852412c33d03d101"),
        (hx(&e[40]), "80331b2b29ebebc85b67013ed49756242fbaf0c6c9938adeef7999d28bbed7bc056f47d0e1ab35c2df52ff6bd8310bc75ff63e159cb93a335838fdaaf4143bdc"),
        (hx(&record.commitment()), "d9f0d13cd7c4d70db9562a981fc091a49b37e28bd56c6a3cda341e7755da0148a9bc454ec62af618638e51f672206563c4dc74de577cfad25afee514d5ef7882"),
        (hx(&record2.commitment()), "d1a3a6df8e585762a8afad0e69f410e25b24087aa519cf94488b11556b4d41a5ba0eaf6ef0ae6613337eda7050cc2fbd9444255221343e753f74caf8a3cbbed5"),
        (hx(&root_of(&[record.commitment()])), "16bc60964ede197fb4b0f9a917563bc446c383021ca99dbceccfd394255d27480b64bcc3d74e623f49735336d1b74f78ff3f2800ffb8b08a8eb199a42a3e12dd"),
        (hx(&root_of(&[record.commitment(), record2.commitment()])), "3520b02a96ba0cfb674ff905cf38c223ae045e74dc1f3fe834948f7ad6fbd9bea0df59c94501d942f358297b2db88e56ca702ecc1d25aa3d8a5e60d5aa16e13d"),
        (hx(&spend.id().unwrap()), "e69de85afc6829df8718d1f181936dbfd7ff9cdd2b14e6ff565791a4732aad181e816d935850c0e2eb021c9e53ce0b9c42d248a12e01872cd7ee55183d507a1a"),
        (hx(&stmt), "783d66bfba635b5fb2c298b8c014f06f92a8dfe9d90dc04d7bcb2a6fc6e1fb725d58647b66b75a26c41438d9f87fb7d9b4cf0e627d8e8a8ca5f413c7da46c6db"),
    ] {
        assert_eq!(got, want);
    }
}
