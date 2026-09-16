// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Migration claims — binding a legacy Cosmos key to a native neuron.
//!
//! A migrated network's neuron records are moved by snapshot, but that leaves
//! them orphaned: native authentication is `Hemera(secret)` of a *new* secret
//! nobody holds. A **claim** returns control to the original holder. They sign,
//! with the secp256k1 key they already have, a message binding their legacy
//! address to a native neuron. Anyone can then verify the binding — this is the
//! single point where a classical signature is structurally required.
//!
//! The message is signed as an **ADR-036** "offline" sign doc (Cosmos's scheme
//! for signing arbitrary data — what Keplr's `signArbitrary` produces), so a
//! holder can build a claim from any standard wallet without an on-chain tx.
//!
//! Phase 1 verifies natively. Phase 2 re-expresses [`verify`] as a nox program
//! that emits a zheng proof, so the binding needs no trusted verifier.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};

use crate::{cosmos, Error};

/// Domain-separation tag for the binding message. Versioned so the Phase-2
/// proof and any future format can be told apart.
const BIND_TAG: &str = "cyber:migrate:v1:";

/// A migration claim: proof that the holder of `address` authorizes `neuron`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    /// The legacy bech32 address (e.g. `pussy1…`) the holder controls.
    pub address: String,
    /// The 33-byte SEC1-compressed secp256k1 public key.
    pub pubkey: [u8; 33],
    /// The native neuron id the legacy key binds to.
    pub neuron: [u8; 32],
    /// The 64-byte secp256k1 ECDSA signature over the ADR-036 sign doc.
    pub signature: [u8; 64],
}

impl Claim {
    /// Encode a claim as one transmissible line: `address pubkey neuron signature`
    /// (the three byte fields in hex). A holder pastes this; a verifier decodes it.
    pub fn encode(&self) -> String {
        format!(
            "{} {} {} {}",
            self.address,
            to_hex(&self.pubkey),
            to_hex(&self.neuron),
            to_hex(&self.signature)
        )
    }

    /// Parse a claim from its [`encode`](Claim::encode)d form. Returns `None` on
    /// any malformed field (wrong hex, wrong length, extra tokens).
    pub fn decode(s: &str) -> Option<Claim> {
        let mut it = s.split_whitespace();
        let address = it.next()?.to_string();
        let pubkey: [u8; 33] = from_hex(it.next()?)?.try_into().ok()?;
        let neuron: [u8; 32] = from_hex(it.next()?)?.try_into().ok()?;
        let signature: [u8; 64] = from_hex(it.next()?)?.try_into().ok()?;
        if it.next().is_some() {
            return None;
        }
        Some(Claim { address, pubkey, neuron, signature })
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

/// The canonical native neuron id for a legacy account: `Hemera(pubkey)`.
///
/// This is the deterministic self-neuron every migrated account maps to when
/// its pubkey is known. A holder re-keying to a fresh post-quantum secret binds
/// to `Hemera(new_secret)` instead — [`create`] takes the target neuron as
/// input, so either works.
pub fn neuron_of(compressed_pubkey: &[u8]) -> [u8; 32] {
    *hemera::hash(compressed_pubkey).as_bytes()
}

/// The bytes a holder signs: the tag followed by the hex neuron id.
pub fn bind_message(neuron: &[u8; 32]) -> Vec<u8> {
    let mut s = String::with_capacity(BIND_TAG.len() + 64);
    s.push_str(BIND_TAG);
    for b in neuron {
        s.push_str(&format!("{b:02x}"));
    }
    s.into_bytes()
}

/// The ADR-036 sign doc for arbitrary `data` signed by `signer`.
///
/// Built by hand to guarantee the amino-JSON canonical form: keys sorted
/// alphabetically, no whitespace. This byte string is what gets signed
/// (ECDSA over its sha256), matching what a Cosmos wallet produces.
pub fn adr036_doc(signer: &str, data: &[u8]) -> String {
    let data_b64 = B64.encode(data);
    format!(
        "{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\
         \"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":\
         {{\"data\":\"{data_b64}\",\"signer\":\"{signer}\"}}}}],\"sequence\":\"0\"}}"
    )
}

/// Sign arbitrary `data` as an ADR-036 offline doc, `signer` filling the
/// doc's address field. General-purpose — [`create`] is the one caller inside
/// this module (passing [`bind_message`] as `data`), but any caller with an
/// ADR-036-shaped signing need (a different neuron's own event stream, say)
/// can use this directly rather than reimplementing the doc shape.
pub fn sign_arbitrary(key: &SigningKey, signer: &str, data: &[u8]) -> [u8; 64] {
    let doc = adr036_doc(signer, data);
    let sig: Signature = key.sign(doc.as_bytes());
    let mut out = [0u8; 64];
    out.copy_from_slice(&sig.to_bytes());
    out
}

/// Verify an ADR-036 signature over `data`, `signer` filling the doc's
/// address field. The counterpart to [`sign_arbitrary`] — does not check
/// that `pubkey` derives any particular address; callers that need that
/// binding (like [`verify`]) check it themselves first.
pub fn verify_arbitrary(pubkey: &[u8; 33], signer: &str, data: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(vk) = VerifyingKey::from_sec1_bytes(pubkey) else { return false };
    let Ok(sig) = Signature::from_slice(signature) else { return false };
    let doc = adr036_doc(signer, data);
    vk.verify(doc.as_bytes(), &sig).is_ok()
}

/// Build a claim: bind `neuron` to the account `key` controls under `hrp`.
pub fn create(key: &SigningKey, hrp: &str, neuron: [u8; 32]) -> Result<Claim, Error> {
    let pubkey = cosmos::compressed(key.verifying_key());
    let address = cosmos::address(&pubkey, hrp)?;
    let signature = sign_arbitrary(key, &address, &bind_message(&neuron));
    Ok(Claim { address, pubkey, neuron, signature })
}

/// Verify a claim under `hrp`: the pubkey must derive the claimed address, and
/// the signature must verify over the canonical binding doc. Returns `true`
/// only if the holder genuinely authorized `neuron` from `address`.
pub fn verify(claim: &Claim, hrp: &str) -> bool {
    // 1. the pubkey must actually derive the claimed address
    let Ok(derived) = cosmos::address(&claim.pubkey, hrp) else { return false };
    if derived != claim.address {
        return false;
    }
    // 2. the signature must verify over the exact binding doc
    verify_arbitrary(&claim.pubkey, &claim.address, &bind_message(&claim.neuron), &claim.signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    const MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn key() -> SigningKey {
        seed::cosmos_key(MNEMONIC, "").unwrap()
    }

    #[test]
    fn round_trip_claim_verifies() {
        let k = key();
        let neuron = neuron_of(&cosmos::compressed(k.verifying_key()));
        let claim = create(&k, cosmos::PUSSY, neuron).unwrap();
        assert!(verify(&claim, cosmos::PUSSY));
        assert!(claim.address.starts_with("pussy1"));
    }

    #[test]
    fn tampered_neuron_is_rejected() {
        let k = key();
        let neuron = neuron_of(&cosmos::compressed(k.verifying_key()));
        let mut claim = create(&k, cosmos::PUSSY, neuron).unwrap();
        claim.neuron[0] ^= 0x01; // forge a binding to a different neuron
        assert!(!verify(&claim, cosmos::PUSSY), "signature no longer covers the neuron");
    }

    #[test]
    fn wrong_hrp_is_rejected() {
        let k = key();
        let neuron = neuron_of(&cosmos::compressed(k.verifying_key()));
        let claim = create(&k, cosmos::PUSSY, neuron).unwrap();
        // address says pussy1…, but we verify as if it were bostrom → mismatch
        assert!(!verify(&claim, cosmos::BOSTROM));
    }

    #[test]
    fn swapped_pubkey_is_rejected() {
        let k = key();
        let neuron = neuron_of(&cosmos::compressed(k.verifying_key()));
        let mut claim = create(&k, cosmos::PUSSY, neuron).unwrap();
        // a different account's pubkey no longer matches the address or sig
        let other = seed::cosmos_key(MNEMONIC, "other").unwrap();
        claim.pubkey = cosmos::compressed(other.verifying_key());
        assert!(!verify(&claim, cosmos::PUSSY));
    }

    #[test]
    fn claim_encodes_and_decodes_round_trip() {
        let k = key();
        let neuron = neuron_of(&cosmos::compressed(k.verifying_key()));
        let claim = create(&k, cosmos::PUSSY, neuron).unwrap();
        let wire = claim.encode();
        let back = Claim::decode(&wire).expect("decodes");
        assert_eq!(claim, back);
        assert!(verify(&back, cosmos::PUSSY));
    }

    #[test]
    fn decode_rejects_malformed() {
        assert!(Claim::decode("not enough fields").is_none());
        assert!(Claim::decode("addr zz zz zz").is_none());
    }

    #[test]
    fn neuron_of_is_deterministic() {
        let k = key();
        let pk = cosmos::compressed(k.verifying_key());
        assert_eq!(neuron_of(&pk), neuron_of(&pk));
    }
}
