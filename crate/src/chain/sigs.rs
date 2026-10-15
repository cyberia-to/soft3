//! Identities and signatures: txid and statement (§2.1), NSIG1 through mudra
//! (§2.7), the legacy owner and claim rules (§2.8), tokens (§3.1), the
//! checkpoint statement (§5.3) and gossip topic ids (chain-wire §2).

use super::header::hash32;
use super::params::{DOMAIN_CHECKPOINT, DOMAIN_LEGACY, DOMAIN_SIGN, DOMAIN_TOPIC, DOMAIN_TXID};
use super::tx::{SIG_BYTES, Tx, TxBody};

pub type NeuronId = [u8; 32];

fn tagged(tag: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(tag.len() + parts.iter().map(|p| p.len()).sum::<usize>());
    buf.extend_from_slice(tag);
    for p in parts {
        buf.extend_from_slice(p);
    }
    hash32(&buf)
}

/// `txid = hemera("soft3/chain/txid/v1\0" ‖ chain ‖ tx_bytes)`.
pub fn txid(chain: &[u8; 32], tx_bytes: &[u8]) -> [u8; 32] {
    tagged(DOMAIN_TXID, &[chain, tx_bytes])
}

/// `statement = hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx_bytes_without_sig)`.
pub fn statement(chain: &[u8; 32], unsigned: &[u8]) -> [u8; 32] {
    tagged(DOMAIN_SIGN, &[chain, unsigned])
}

/// `token(denom) = hemera(denom_bytes)` (§3.1).
pub fn token(denom: &str) -> [u8; 32] {
    hash32(denom.as_bytes())
}

/// `neuron_of(pubkey) = hemera(compressed pubkey)` — mudra's native id.
pub fn neuron_of(pubkey: &[u8; 33]) -> NeuronId {
    mudra::claim::neuron_of(pubkey)
}

/// `legacy_owner(hrp, account_id)`: the dormant owner of an address-only
/// snapshot account (§2.8).
pub fn legacy_owner(hrp: &str, account_id: &[u8; 20]) -> NeuronId {
    tagged(DOMAIN_LEGACY, &[hrp.as_bytes(), account_id])
}

/// Statement a checkpoint signer signs (§5.3).
pub fn checkpoint_statement(chain: &[u8; 32], height: u64, block_id: &[u8; 32]) -> [u8; 32] {
    tagged(DOMAIN_CHECKPOINT, &[chain, &height.to_le_bytes(), block_id])
}

/// Gossip topic id (chain-wire §2).
pub fn topic(chain: &[u8; 32], name: &str) -> [u8; 32] {
    tagged(DOMAIN_TOPIC, &[chain, name.as_bytes()])
}

/// The 33-byte public key inside an NSIG1 envelope, if well-formed.
pub fn sig_pubkey(sig: &[u8]) -> Option<[u8; 33]> {
    if sig.len() != SIG_BYTES || &sig[..5] != b"NSIG1" {
        return None;
    }
    sig[5..38].try_into().ok()
}

/// The subject a transaction's signature must prove: `from` for kinds 1
/// and 2, `neuron_of(sig.pubkey)` for a claim, none for a coinbase.
pub fn subject(tx: &Tx) -> Option<NeuronId> {
    match &tx.body {
        TxBody::Coinbase(_) => None,
        TxBody::Transfer(t) => Some(t.from),
        TxBody::Cyberlink(l) => Some(l.from),
        TxBody::Claim(_) => sig_pubkey(&tx.sig).map(|pk| neuron_of(&pk)),
    }
}

/// `verify_statement(subject, statement, sig)` over the chain-bound
/// statement (§2.7). A coinbase has no signature and passes.
pub fn verify(chain: &[u8; 32], tx: &Tx) -> bool {
    let Some(subject) = subject(tx) else {
        return tx.is_coinbase() && tx.sig.is_empty();
    };
    let st = statement(chain, &tx.encode_unsigned());
    mudra::neuron::verify_statement(subject, st, &tx.sig)
}

/// Sign a kind-1/2/3 transaction with `key`; the subject is derived from the
/// key for a claim and must equal `from` otherwise.
pub fn sign(chain: &[u8; 32], tx: &mut Tx, key: &mudra::SigningKey) -> Result<(), mudra::Error> {
    let pubkey = mudra::cosmos::compressed(key.verifying_key());
    let subject = match &tx.body {
        TxBody::Coinbase(_) => return Ok(()),
        TxBody::Transfer(t) => t.from,
        TxBody::Cyberlink(l) => l.from,
        TxBody::Claim(_) => neuron_of(&pubkey),
    };
    let st = statement(chain, &tx.encode_unsigned());
    tx.sig = mudra::neuron::sign(key, subject, st)?;
    Ok(())
}

/// `ripemd160(sha256(pubkey)) == account_id` (§2.8).
pub fn claim_key_matches(sig: &[u8], account_id: &[u8; 20]) -> bool {
    sig_pubkey(sig).is_some_and(|pk| mudra::cosmos::account_id(&pk) == *account_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tx::{Claim, Transfer};

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn unhex32(s: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    pub const ABANDON: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    pub fn abandon_key() -> mudra::SigningKey {
        mudra::spell::cosmos_key(ABANDON, "").unwrap()
    }

    #[test]
    fn conformance_section_1_and_7_anchors() {
        assert_eq!(hex(&hash32(b"")), "a67a71b221e6bdd6442a20432bf5d74c885d89e5dfbeec3ec4e334cb806d563c");
        assert_eq!(hex(&token("boot")), "10275cf6703891a515f64ada465137cd36f2a5d3f0b99a8dafc62e6b95ad1269");
        assert_eq!(hex(&token("pussy")), "8cffbea0ac348d13febcee5b2848f916f501237d3ed8966167a31bafddf18ffa");
        assert_eq!(hex(&hash32(&[0u8; 168])), "ba560d1fd2ed380e09ef8e526e8817504cce6e71363c12081537751e4f761754");
        assert_eq!(hex(&topic(&[0x44; 32], "headers")), "1f003a48b0506c0ea85cb893abc83c6b6a2603adea7e62f9fe6997692f9f8b8b");
        assert_eq!(
            hex(&legacy_owner("pussy", &[0x99; 20])),
            "d5e54c499456ee690e5c82a3d2119a8180a2e13f832df5254757a41f5e6d509d"
        );
    }

    #[test]
    fn conformance_section_6_transfer_statement() {
        let tx = Tx {
            body: TxBody::Transfer(Transfer { from: [0x66; 32], nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 }),
            sig: vec![],
        };
        let unsigned = tx.encode_unsigned();
        assert_eq!(unsigned.len(), 122);
        assert_eq!(hex(&statement(&[0x44; 32], &unsigned)), "c7e4a223871018bb9f3f64466b45df3a2962ff344776df2960600dfc49cf4a6f");
        let cb = super::super::tx::tests::coinbase().encode();
        assert_eq!(hex(&txid(&[0x44; 32], &cb)), "d7662bdbf363831e4f2efd77967f684db927cf04576f7e82bb8f1872e333cc51");
    }

    #[test]
    fn abandon_key_identity_and_legacy_owner() {
        let key = abandon_key();
        let pk = mudra::cosmos::compressed(key.verifying_key());
        assert_eq!(hex(&pk), "024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62");
        let neuron = neuron_of(&pk);
        assert_eq!(hex(&neuron), "31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741");
        let account = mudra::cosmos::account_id(&pk);
        assert_eq!(hex(&account), "28ff5c6d57d8cfd492b6fb42614536ed648e01fd");
        assert_eq!(hex(&legacy_owner("pussy", &account)), "cca6f468fe834711fc1e672c21ba95ee7028ed9af6915fbc7b95b449ef0cdbc0");
    }

    #[test]
    fn signed_transfer_verifies_and_every_mutation_fails() {
        let chain = [0x44; 32];
        let key = abandon_key();
        let from = neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
        let mut tx = Tx {
            body: TxBody::Transfer(Transfer { from, nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 }),
            sig: vec![],
        };
        sign(&chain, &mut tx, &key).unwrap();
        assert_eq!(tx.encode().len(), 224);
        assert!(verify(&chain, &tx));
        assert!(!verify(&[0x45; 32], &tx), "replay across chains");
        for i in 0..SIG_BYTES {
            let mut t = tx.clone();
            t.sig[i] ^= 1;
            assert!(!verify(&chain, &t), "sig byte {i}");
        }
        let mut t = tx.clone();
        t.sig.truncate(101);
        assert!(!verify(&chain, &t));
        let mut t = tx.clone();
        t.sig.push(0);
        assert!(!verify(&chain, &t));
        let mut t = tx.clone();
        if let TxBody::Transfer(x) = &mut t.body {
            x.amount = 6;
        }
        assert!(!verify(&chain, &t), "statement changed");
        let mut t = tx.clone();
        if let TxBody::Transfer(x) = &mut t.body {
            x.from[0] ^= 1;
        }
        assert!(!verify(&chain, &t), "subject changed");
        // signing a subject the key does not own fails
        let mut other = Tx {
            body: TxBody::Transfer(Transfer { from: [0x66; 32], nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 }),
            sig: vec![],
        };
        assert!(sign(&chain, &mut other, &key).is_err());
        assert_eq!(unhex32("31284576301bacb9c2cad6379244f856dc32c751c38c3b0af4c5c886fd0ed741"), from);
    }

    #[test]
    fn claim_subject_is_the_key_and_account_must_match() {
        let chain = [0x44; 32];
        let key = abandon_key();
        let pk = mudra::cosmos::compressed(key.verifying_key());
        let mut tx = Tx {
            body: TxBody::Claim(Claim { account_id: mudra::cosmos::account_id(&pk), nonce: 0, fee: 140 }),
            sig: vec![],
        };
        sign(&chain, &mut tx, &key).unwrap();
        assert_eq!(tx.encode().len(), 140);
        assert!(verify(&chain, &tx));
        assert_eq!(subject(&tx), Some(neuron_of(&pk)));
        assert!(claim_key_matches(&tx.sig, &mudra::cosmos::account_id(&pk)));
        assert!(!claim_key_matches(&tx.sig, &[0x99; 20]));
    }
}
