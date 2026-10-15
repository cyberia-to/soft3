//! chain-conformance.md §1, §6, §7 — the vectors a wallet can pin without a
//! node: hemera anchors, tx bytes, statements, txids, identities; plus the
//! signature checks the kernel's step 7 makes (wrong chain, wrong key,
//! truncation) against this module's own `verify`.

use super::tx::*;
use super::*;

const CHAIN: [u8; 32] = [0x44; 32];
const SPELL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn h32(s: &str) -> [u8; 32] {
    parse32(s).unwrap()
}

fn signer() -> Signer {
    Signer::from_spell(SPELL, "").unwrap()
}

// §1 hemera anchors
#[test]
fn hemera_anchors() {
    assert_eq!(hex(hemera::hash(b"").as_bytes()), "a67a71b221e6bdd6442a20432bf5d74c885d89e5dfbeec3ec4e334cb806d563c");
    assert_eq!(hex(&token("boot")), "10275cf6703891a515f64ada465137cd36f2a5d3f0b99a8dafc62e6b95ad1269");
    assert_eq!(hex(&token("pussy")), "8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa");
    assert_eq!(hex(hemera::hash(&[0u8; 168]).as_bytes()), "ba560d1fd2ed380e09ef8e526e8817504cce6e71363c12081537751e4f761754");
}

// §6 coinbase
#[test]
fn coinbase_vector() {
    let tx = Tx::Coinbase { height: 1, to: [0x55; 32], amount: 10_000, extra: [0; 32] };
    let bytes = tx.encode_unsigned();
    assert_eq!(bytes.len(), 82);
    assert_eq!(hex(&bytes[..10]), "01000100000000000000");
    assert_eq!(hex(&bytes[42..50]), "1027000000000000");
    assert_eq!(hex(&txid(&CHAIN, &bytes)), "d7662bdbf363831e4f2efd77967f684db927cf04576f7e82bb8f1872e333cc51");
    let l = leaf(&bytes);
    assert_eq!(hex(&l), "c2cb6adb74ed69c5672c495ccfb659e12b8f33db3be8caf6167a0d50b41893e6");
    assert_eq!(tx_root(std::slice::from_ref(&bytes)), Some(l));
    assert_eq!(hex(&node(&l, &l)), "3685e0a5b8bce069a4ef54da15f56393cd5e64dc6d10701d247a4663b7d922c1");
    assert_eq!(tx_root(&[bytes.clone(), bytes.clone()]), Some(node(&l, &l)));
    // three leaves: the odd one is promoted, not duplicated
    assert_eq!(tx_root(&[bytes.clone(), bytes.clone(), bytes.clone()]), Some(node(&node(&l, &l), &l)));
    let (decoded, sig) = Tx::decode(&bytes).unwrap();
    assert_eq!(decoded, tx);
    assert!(sig.is_none());
}

// §6 transfer: unsigned bytes and statement
#[test]
fn transfer_statement_vector() {
    let tx = Tx::Transfer {
        from: [0x66; 32],
        nonce: 0,
        token: token("pussy"),
        amount: 5,
        to: [0x77; 32],
        fee: 224,
    };
    let unsigned = tx.encode_unsigned();
    assert_eq!(unsigned.len(), 122);
    let expected = format!(
        "0101{}0000000000000000{}0500000000000000{}e000000000000000",
        "66".repeat(32),
        "8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa",
        "77".repeat(32)
    );
    assert_eq!(hex(&unsigned), expected);
    assert_eq!(hex(&statement(&CHAIN, &unsigned)), "c7e4a223871018bb9f3f64466b45df3a2962ff344776df2960600dfc49cf4a6f");
}

// §7 identities and the known-answer key
#[test]
fn known_answer_key_identities() {
    let s = signer();
    assert_eq!(hex(&s.pubkey), "024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62");
    assert_eq!(s.neuron_hex(), "31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741");
    assert_eq!(hex(&s.account_id), "28ff5c6d57d8cfd492b6fb42614536ed648e01fd");
    assert_eq!(s.address("pussy").unwrap(), "pussy19rl4cm2hmr8afy4kldpxz3fka4jguq0a4l8rzm");
    assert_eq!(hex(&s.legacy_owner("pussy")), "cca6f468fe834711fc1e672c21ba95ee7028ed9af6915fbc7b95b449ef0cdbc0");
    assert_eq!(hex(&legacy_owner("pussy", &[0x99; 20])), "d5e54c499456ee690e5c82a3d2119a8180a2e13f832df5254757a41f5e6d509d");
    assert_eq!(hex(&topic(&CHAIN, "headers")), "1f003a48b0506c0ea85cb893abc83c6b6a2603adea7e62f9fe6997692f9f8b8b");
}

// §6 signed transfer: verifies; flipped bytes, truncations and another chain fail
#[test]
fn signed_transfer_verifies_and_rejects_tampering() {
    let s = signer();
    let signed = s.transfer(&CHAIN, 0, token("pussy"), 5, [0x77; 32], None).unwrap();
    assert_eq!(signed.bytes.len(), 224);
    assert_eq!(signed.txid, txid(&CHAIN, &signed.bytes));
    let (tx, who) = verify(&CHAIN, &signed.bytes).unwrap();
    assert_eq!(tx, signed.tx);
    assert_eq!(who, Some(s.neuron));
    if let Tx::Transfer { from, fee, .. } = tx {
        assert_eq!(from, s.neuron);
        assert_eq!(fee, 224);
    }
    // cross-chain replay
    assert!(matches!(verify(&[0x45; 32], &signed.bytes), Err(Error::Key(_))));
    // every byte of the signature, the body and the head
    for i in [0, 1, 2, 40, 121, 122, 127, 160, 223] {
        let mut t = signed.bytes.clone();
        t[i] ^= 1;
        assert!(verify(&CHAIN, &t).is_err(), "byte {i} flipped still verifies");
    }
    // 223 and 225 bytes
    assert!(matches!(verify(&CHAIN, &signed.bytes[..223]), Err(Error::Encoding(_))));
    let mut longer = signed.bytes.clone();
    longer.push(0);
    assert!(matches!(verify(&CHAIN, &longer), Err(Error::Encoding(_))));
}

#[test]
fn wrong_key_and_wrong_nonce_are_distinguishable() {
    let s = signer();
    let other = Signer::from_secret(&[7u8; 32]).unwrap();
    // a transfer claiming `from = s.neuron` signed by another key never leaves the signer
    let forged = other.sign(
        &CHAIN,
        Tx::Transfer { from: s.neuron, nonce: 0, token: token("pussy"), amount: 1, to: [0x77; 32], fee: 224 },
    );
    assert!(matches!(forged, Err(Error::Key(_))));
    // the same body with a different nonce is a different statement and txid
    let a = s.transfer(&CHAIN, 0, token("pussy"), 1, [0x77; 32], None).unwrap();
    let b = s.transfer(&CHAIN, 1, token("pussy"), 1, [0x77; 32], None).unwrap();
    assert_ne!(a.txid, b.txid);
    assert_eq!(a.tx.nonce(), Some(0));
    assert_eq!(b.tx.nonce(), Some(1));
    // splicing a's signature onto b's body fails: the nonce is under the statement
    let mut spliced = b.bytes[..122].to_vec();
    spliced.extend_from_slice(&a.bytes[122..]);
    assert!(matches!(verify(&CHAIN, &spliced), Err(Error::Key(_))));
}

// §6 cyberlink: 257 bytes, valence 2 and p == q rejected
#[test]
fn cyberlink_rules() {
    let s = signer();
    let link = s.cyberlink(&CHAIN, 0, token("boot"), token("pussy"), 1, token("pussy"), 0, None).unwrap();
    assert_eq!(link.bytes.len(), 257);
    assert_eq!(link.tx.fee(), Some(257));
    assert!(verify(&CHAIN, &link.bytes).is_ok());
    assert!(matches!(
        s.cyberlink(&CHAIN, 0, token("boot"), token("pussy"), 2, token("pussy"), 0, None),
        Err(Error::Rule(_))
    ));
    assert!(matches!(
        s.cyberlink(&CHAIN, 0, token("boot"), token("boot"), 1, token("pussy"), 0, None),
        Err(Error::Rule(_))
    ));
    // a valence byte of 2 on the wire does not decode
    let mut t = link.bytes.clone();
    t[2 + 32 + 8 + 32 + 8 + 32 + 32] = 2;
    assert!(matches!(Tx::decode(&t), Err(Error::Encoding(_))));
    // negative valence round-trips
    let neg = s.cyberlink(&CHAIN, 3, [1; 32], [2; 32], -1, token("pussy"), 0, Some(300)).unwrap();
    let (tx, _) = Tx::decode(&neg.bytes).unwrap();
    assert!(matches!(tx, Tx::Cyberlink { valence: -1, fee: 300, nonce: 3, .. }));
}

// §6 claim: 140 bytes; account id must be the signing key's
#[test]
fn claim_rules() {
    let s = signer();
    let claim = s.claim(&CHAIN, 0, None).unwrap();
    assert_eq!(claim.bytes.len(), 140);
    assert_eq!(&claim.bytes[2..22], &unhex("28ff5c6d57d8cfd492b6fb42614536ed648e01fd")[..]);
    let (tx, who) = verify(&CHAIN, &claim.bytes).unwrap();
    assert_eq!(who, Some(s.neuron));
    assert_eq!(tx.fee(), Some(140));
    // an account id the key does not derive
    let mut t = claim.bytes.clone();
    t[2] ^= 1;
    assert!(matches!(verify(&CHAIN, &t), Err(Error::Key(_))));
}

#[test]
fn fee_floors_and_encoding_rejections() {
    assert_eq!(Kind::Transfer.min_fee(), 224);
    assert_eq!(Kind::Cyberlink.min_fee(), 257);
    assert_eq!(Kind::Claim.min_fee(), 140);
    assert_eq!(Kind::Coinbase.total_bytes(), 82);
    let s = signer();
    assert!(matches!(s.transfer(&CHAIN, 0, token("pussy"), 1, [0x77; 32], Some(223)), Err(Error::Rule(_))));
    let ok = s.transfer(&CHAIN, 0, token("pussy"), 1, [0x77; 32], Some(225)).unwrap();
    let mut v2 = ok.bytes.clone();
    v2[0] = 2;
    assert!(matches!(Tx::decode(&v2), Err(Error::Encoding(_))));
    let mut k4 = ok.bytes.clone();
    k4[1] = 4;
    assert!(matches!(Tx::decode(&k4), Err(Error::Encoding(_))));
    assert!(Tx::decode(&[]).is_err());
    assert!(Tx::decode(&[1]).is_err());
}

#[test]
fn cards_and_tokens_by_name_or_hex() {
    assert_eq!(card(&"ab".repeat(32)), [0xab; 32]);
    assert_eq!(card(&format!("0x{}", "ab".repeat(32))), [0xab; 32]);
    assert_eq!(card("pussy"), token("pussy"));
    assert_eq!(token_arg("boot"), token("boot"));
    assert_eq!(token_arg(&h32(&"10".repeat(32)).iter().map(|b| format!("{b:02x}")).collect::<String>()), [0x10; 32]);
    assert!(parse32("abc").is_err());
}
