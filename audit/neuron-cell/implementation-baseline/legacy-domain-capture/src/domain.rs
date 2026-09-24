// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Fresh, domain-scoped secp256k1 identity — no legacy account to reproduce.
//!
//! [`seed`] recovers an *existing* Cosmos-SDK account: the BIP-32 path is
//! fixed because a real chain already recognizes that key, there is no
//! freedom in the derivation. This module is the other case: a brand-new
//! identity, generated in a browser, with no prior account and no BIP-32
//! ladder to walk. `d = Hemera(entropy ‖ 0x00 ‖ domain) mod n` — one hash,
//! not bip32's HMAC-SHA512 chain — because the caller (a wasm tracker) pays
//! for every dependency it links, and bip32 was never load-bearing here:
//! unlinkability across domains only needs the hash to be domain-bound, which
//! `Hemera(entropy ‖ domain)` already is.
//!
//! secp256k1 here is not the legacy bridge. mudra is post-quantum and
//! signature-free by design everywhere else — a *native* neuron authenticates
//! with a zheng proof of `Hemera(secret)` preimage, no classical signature at
//! all. this module exists because a *browser* is not that native context:
//! secp256k1 is what a browser extension wallet (`window.ethereum`, Keplr) or
//! a ~4KB JS library (noble-secp256k1) already speaks, at zero-to-minimal
//! wasm cost. that is a permanent property of the browser as a platform, not
//! a migration this module outgrows — unlike [`crate::claim`], which exists
//! only until every legacy account has bound itself to a native neuron and
//! can be deleted.
//!
//! reuses [`crate::cosmos`] for bech32 encoding and the same `Hemera(pubkey)`
//! native-id convention [`crate::claim::neuron_of`] uses — one bech32
//! implementation, one native-id formula, for both use cases.

use k256::elliptic_curve::ops::Reduce;
use k256::ecdsa::SigningKey;

use crate::{claim, cosmos, Error};

/// The per-domain secret scalar: `Hemera(entropy ‖ 0x00 ‖ domain) mod n`.
/// `domain` must already be the registrable domain (eTLD+1) — the 0x00 byte
/// separates entropy from domain so no `(entropy, domain)` pair collides with
/// another by concatenation. bias from the mod-n reduction is ~2^-128 (n is
/// ~2^256); the digest is zero with probability ~2^-256 — reduce always
/// yields a valid scalar in practice.
pub fn domain_scalar(entropy: &[u8; 32], domain: &str) -> k256::Scalar {
    let mut input = Vec::with_capacity(33 + domain.len());
    input.extend_from_slice(entropy);
    input.push(0x00);
    input.extend_from_slice(domain.as_bytes());
    let h = hemera::hash(&input);
    <k256::Scalar as Reduce<k256::U256>>::reduce(k256::U256::from_be_slice(h.as_bytes()))
}

/// A domain-scoped identity: the derived key and its wire identities.
pub struct DomainKey {
    signing: SigningKey,
    /// SEC1-compressed pubkey, 33 bytes.
    pub pubkey: [u8; 33],
    /// bech32 wire form, under the caller's HRP.
    pub bech32: String,
    /// native id — `Hemera(compressed pubkey)`, the same formula
    /// [`crate::claim::neuron_of`] uses for the legacy bridge.
    pub native: [u8; 32],
}

impl std::fmt::Debug for DomainKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainKey").field("bech32", &self.bech32).finish()
    }
}

impl DomainKey {
    /// Derive the identity `entropy` presents to `domain`, wire-encoded under `hrp`.
    pub fn derive(entropy: &[u8; 32], domain: &str, hrp: &str) -> Result<Self, Error> {
        let scalar = domain_scalar(entropy, domain);
        let signing = SigningKey::from_bytes(&scalar.to_bytes())
            .map_err(|e| Error::Derive(e.to_string()))?;
        let pubkey_point = signing.verifying_key().to_encoded_point(true);
        let mut pubkey = [0u8; 33];
        pubkey.copy_from_slice(pubkey_point.as_bytes());
        let bech32 = cosmos::address(&pubkey, hrp)?;
        let native = claim::neuron_of(&pubkey);
        Ok(Self { signing, pubkey, bech32, native })
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_entropy_same_domain_same_key() {
        let e = [3u8; 32];
        let a = DomainKey::derive(&e, "example.com", "lytics").unwrap();
        let b = DomainKey::derive(&e, "example.com", "lytics").unwrap();
        assert_eq!(a.bech32, b.bech32);
        assert_eq!(a.native, b.native);
    }

    #[test]
    fn different_domains_different_keys() {
        let e = [3u8; 32];
        let a = DomainKey::derive(&e, "example.com", "lytics").unwrap();
        let b = DomainKey::derive(&e, "cyber.page", "lytics").unwrap();
        assert_ne!(a.bech32, b.bech32);
    }

    #[test]
    fn domain_scalar_is_deterministic_and_domain_bound() {
        let e = [3u8; 32];
        assert_eq!(domain_scalar(&e, "example.com"), domain_scalar(&e, "example.com"));
        assert_ne!(domain_scalar(&e, "example.com"), domain_scalar(&e, "cyber.page"));
    }

    #[test]
    fn bech32_uses_hrp() {
        let k = DomainKey::derive(&[7u8; 32], "example.com", "lytics").unwrap();
        assert!(k.bech32.starts_with("lytics1"), "got {}", k.bech32);
    }

    #[test]
    fn native_id_matches_the_bridge_formula() {
        let k = DomainKey::derive(&[7u8; 32], "example.com", "lytics").unwrap();
        assert_eq!(k.native, claim::neuron_of(&k.pubkey));
    }
}
