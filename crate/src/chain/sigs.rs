//! Identities and signatures: txid and statement (§2.1), NSIG1 through mudra
//! (§2.7), the legacy owner and claim rules (§2.8), tokens (§3.1), the
//! checkpoint statement (§5.3) and gossip topic ids (chain-wire §2).

use super::header::hash32;
use super::params::DOMAIN_CHECKPOINT;
use super::tx::{Body, SIG_BYTES, Tx};

pub use crate::wallet::tx::{legacy_owner, neuron_of, statement, token, topic, txid};

pub type NeuronId = [u8; 32];

fn tagged(tag: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(tag.len() + parts.iter().map(|p| p.len()).sum::<usize>());
    buf.extend_from_slice(tag);
    for p in parts {
        buf.extend_from_slice(p);
    }
    hash32(&buf)
}

/// Statement a checkpoint signer signs (§5.3).
pub fn checkpoint_statement(chain: &[u8; 32], height: u64, block_id: &[u8; 32]) -> [u8; 32] {
    tagged(DOMAIN_CHECKPOINT, &[chain, &height.to_le_bytes(), block_id])
}

/// The 33-byte public key inside an NSIG1 envelope, if well-formed.
pub fn sig_pubkey(sig: &[u8]) -> Option<[u8; 33]> {
    if sig.len() != SIG_BYTES || &sig[..5] != b"NSIG1" {
        return None;
    }
    sig[5..38].try_into().ok()
}

/// The subject a transaction's signature must prove: `from` for kinds 1,
/// 2, 6 and 7, `neuron_of(sig.pubkey)` for a claim, none for a coinbase.
pub fn subject(tx: &Tx) -> Option<NeuronId> {
    match &tx.body {
        Body::Coinbase { .. } => None,
        Body::Claim { .. } => sig_pubkey(&tx.sig).map(|pk| neuron_of(&pk)),
        other => other.from(),
    }
}

/// `verify_statement(subject, statement, sig)` over the chain-bound
/// statement (§2.7). A coinbase has no signature and passes.
pub fn verify(chain: &[u8; 32], tx: &Tx) -> bool {
    let Some(subject) = subject(tx) else {
        return tx.is_coinbase() && tx.sig.is_empty();
    };
    let st = statement(chain, &tx.encode_unsigned());
    let key = verified::key(&subject, &st, &tx.sig);
    if verified::contains(&key) {
        return true;
    }
    #[cfg(test)]
    VERIFICATIONS.with(|c| c.set(c.get() + 1));
    let ok = mudra::neuron::verify_statement(subject, st, &tx.sig);
    if ok {
        verified::insert(key);
    }
    ok
}

/// Signatures that verified, so a transaction is verified once however
/// often it is executed: mempool admission replays an account's pending
/// lower nonces, re-validation after every tip re-admits the whole pool,
/// the template executes it again and so does the block. `verify` is a
/// pure function of (subject, statement, signature) and the key commits to
/// all three, so a hit returns exactly what a recomputation would; only
/// successes are kept, in a bounded first-in-first-out set.
mod verified {
    use std::collections::{HashSet, VecDeque};
    use std::sync::{Mutex, OnceLock};

    use sha2::{Digest, Sha256};

    /// Two mempools' worth (chain-wire §4: 8,192 transactions).
    pub const CAPACITY: usize = 1 << 16;

    #[derive(Default)]
    struct Set {
        keys: HashSet<[u8; 32]>,
        order: VecDeque<[u8; 32]>,
    }

    fn set() -> &'static Mutex<Set> {
        static SET: OnceLock<Mutex<Set>> = OnceLock::new();
        SET.get_or_init(Default::default)
    }

    pub fn key(subject: &[u8; 32], statement: &[u8; 32], sig: &[u8]) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(subject);
        h.update(statement);
        h.update(sig);
        h.finalize().into()
    }

    pub fn contains(key: &[u8; 32]) -> bool {
        set().lock().map(|s| s.keys.contains(key)).unwrap_or(false)
    }

    pub fn insert(key: [u8; 32]) {
        if let Ok(mut s) = set().lock() {
            if s.keys.insert(key) {
                s.order.push_back(key);
                if s.order.len() > CAPACITY {
                    if let Some(old) = s.order.pop_front() {
                        s.keys.remove(&old);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    static VERIFICATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// secp256k1 verifications run on this thread (tests count them).
#[cfg(test)]
pub fn verifications() -> u64 {
    VERIFICATIONS.with(|c| c.get())
}

/// Sign a kind-1/2/3/6/7 transaction with `key`; the subject is derived from the
/// key for a claim and must equal `from` otherwise.
pub fn sign(chain: &[u8; 32], tx: &mut Tx, key: &mudra::SigningKey) -> Result<(), mudra::Error> {
    let pubkey = mudra::cosmos::compressed(key.verifying_key());
    let subject = match &tx.body {
        Body::Coinbase { .. } => return Ok(()),
        Body::Claim { .. } => neuron_of(&pubkey),
        other => other.from().expect("signed kinds carry from"),
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
pub(crate) mod tests {
    use super::*;

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

    pub(crate) const ABANDON: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    pub fn abandon_key() -> mudra::SigningKey {
        mudra::spell::cosmos_key(ABANDON, "").unwrap()
    }

    #[test]
    fn conformance_section_1_and_7_anchors() {
        assert_eq!(hex(&hash32(b"")), "9e89f2487be432e48c27e24e6945adffe6918c8b5d6a62061e70e0bebd657326");
        assert_eq!(hex(&token("boot")), "1d443e1103abbd0aebe4abaf3fadc203160a471e9accb9b6e0a73804ea785b3b");
        assert_eq!(hex(&token("pussy")), "87a70ff9ed0638fc9fafb1a30f57c5ad2ebe5155c48702c50bdbb2ece87f6a71");
        assert_eq!(hex(&hash32(&[0u8; 168])), "355fd03315a5ceb056cda53d11ff7b9755167182d59ff9e31134d69d33bdf0f6");
        assert_eq!(hex(&topic(&[0x44; 32], "headers")), "598b569691c3dc0cc6ee22e1cc7a595ee747e33c1b49defc69bc26c831bbd7a8");
        assert_eq!(
            hex(&legacy_owner("pussy", &[0x99; 20])),
            "5384913f850b453826161f40676a431d912c38c112d944ed85aab22c59889f22"
        );
    }

    #[test]
    fn conformance_section_6_transfer_statement() {
        let tx = Tx {
            body: Body::Transfer { from: [0x66; 32], nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 },
            sig: vec![],
        };
        let unsigned = tx.encode_unsigned();
        assert_eq!(unsigned.len(), 122);
        assert_eq!(hex(&statement(&[0x44; 32], &unsigned)), "41e149559c98af20b62db3e02ac2dfd4c62568f695d92986f9c274a7d62d12a7");
        let cb = super::super::tx::tests::coinbase().encode();
        assert_eq!(hex(&txid(&[0x44; 32], &cb)), "6cf0e4d280e6d5cbb5c7fef566d28913b17249cb4a6fdecf9c59ad3f0995c3f9");
    }

    #[test]
    fn abandon_key_identity_and_legacy_owner() {
        let key = abandon_key();
        let pk = mudra::cosmos::compressed(key.verifying_key());
        assert_eq!(hex(&pk), "024f4e2ad99c34d60b9ba6283c9431a8418af8673212961f97a77b6377fcd05b62");
        let neuron = neuron_of(&pk);
        assert_eq!(hex(&neuron), "55c76c9f172b850eaa82e4c6defe13b1d80d983c29d81526eb94d2c8199b0fd2");
        let account = mudra::cosmos::account_id(&pk);
        assert_eq!(hex(&account), "28ff5c6d57d8cfd492b6fb42614536ed648e01fd");
        assert_eq!(hex(&legacy_owner("pussy", &account)), "4382c7f5d008648955902bb95ef7a6a3cdce9e9c9181a553b9858a73341ce373");
    }

    #[test]
    fn signed_transfer_verifies_and_every_mutation_fails() {
        let chain = [0x44; 32];
        let key = abandon_key();
        let from = neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
        let mut tx = Tx {
            body: Body::Transfer { from, nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 },
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
        if let Body::Transfer { amount, .. } = &mut t.body {
            *amount = 6;
        }
        assert!(!verify(&chain, &t), "statement changed");
        let mut t = tx.clone();
        if let Body::Transfer { from, .. } = &mut t.body {
            from[0] ^= 1;
        }
        assert!(!verify(&chain, &t), "subject changed");
        // signing a subject the key does not own fails
        let mut other = Tx {
            body: Body::Transfer { from: [0x66; 32], nonce: 0, token: token("pussy"), amount: 5, to: [0x77; 32], fee: 224 },
            sig: vec![],
        };
        assert!(sign(&chain, &mut other, &key).is_err());
        assert_eq!(unhex32("55c76c9f172b850eaa82e4c6defe13b1d80d983c29d81526eb94d2c8199b0fd2"), from);
    }

    #[test]
    fn claim_subject_is_the_key_and_account_must_match() {
        let chain = [0x44; 32];
        let key = abandon_key();
        let pk = mudra::cosmos::compressed(key.verifying_key());
        let mut tx = Tx {
            body: Body::Claim { account_id: mudra::cosmos::account_id(&pk), nonce: 0, fee: 140 },
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
