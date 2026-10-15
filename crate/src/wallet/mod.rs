//! wallet — build, sign and submit phase-1 root chain transactions.
//!
//! The client half of [`specs/chain.md`](../../../specs/chain.md): what cyb
//! and true-cyber share so neither carries its own encoding or signing
//! path. Three things a neuron can do — transfer, cyberlink, claim — each a
//! fixed-size record signed with mudra NSIG1 over
//! `hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx)`, binding the chain id so a
//! pussy transaction never replays on bostrom.
//!
//! - [`tx`] — encodings, ids, identities (token, legacy owner, topic)
//! - [`Signer`] — a secp256k1 key as the chain sees it: neuron, pubkey,
//!   legacy account id; `from_spell` for the mnemonic path (mudra bridge)
//! - [`client`] — the HTTP surface of chain-wire §7: status, nonce,
//!   balance, submit, wait for inclusion
//!
//! The kernel (`chain::tx`, branch `feat/chain-kernel`) owns the node-side
//! validator; this module is reconciled against it when it lands and the
//! conformance vectors of chain-conformance §6 pin both until then.

pub mod client;
pub mod tx;
#[cfg(test)]
mod tests;

pub use client::{Client, Status, TxStatus};
pub use tx::{Kind, Tx};

use mudra::SigningKey;

/// Wallet errors: what went wrong, in the words a user can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Bytes that do not decode as a rule-set-1 transaction.
    Encoding(String),
    /// A kind rule the chain would reject (fee floor, p == q, valence).
    Rule(String),
    /// The key or its signature.
    Key(String),
    /// Transport or a node answer the client could not read.
    Http(String),
    /// The node rejected the transaction (HTTP 400 with the reason).
    Rejected(String),
    /// Inclusion did not happen within the wait budget.
    Timeout(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Encoding(m) => write!(f, "encoding: {m}"),
            Error::Rule(m) => write!(f, "rule: {m}"),
            Error::Key(m) => write!(f, "key: {m}"),
            Error::Http(m) => write!(f, "http: {m}"),
            Error::Rejected(m) => write!(f, "rejected: {m}"),
            Error::Timeout(m) => write!(f, "timeout: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// A signed transaction ready for `POST /tx`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTx {
    /// The chain id the statement and txid are bound to.
    pub chain: [u8; 32],
    /// Full wire bytes: `version ‖ kind ‖ body ‖ sig`.
    pub bytes: Vec<u8>,
    /// `hemera("soft3/chain/txid/v1\0" ‖ chain ‖ bytes)`.
    pub txid: [u8; 32],
    /// The decoded body, for display and receipts.
    pub tx: Tx,
}

impl SignedTx {
    pub fn txid_hex(&self) -> String {
        hex(&self.txid)
    }
}

/// A secp256k1 key as the chain identifies it.
///
/// `neuron = hemera(compressed pubkey)` is both mudra's native id and the
/// id genesis gives every snapshot holder whose pubkey was known, so a
/// bostrom or pussy holder signs with the key and mnemonic they already have
/// (chain.md §2.8). `account_id = ripemd160(sha256(pubkey))` is what the
/// claim tx carries for the address-only accounts.
#[derive(Clone)]
pub struct Signer {
    key: SigningKey,
    pub pubkey: [u8; 33],
    pub neuron: [u8; 32],
    pub account_id: [u8; 20],
}

impl Signer {
    pub fn from_key(key: SigningKey) -> Self {
        let pubkey = mudra::cosmos::compressed(key.verifying_key());
        let neuron = tx::neuron_of(&pubkey);
        let account_id = mudra::cosmos::account_id(&pubkey);
        Self { key, pubkey, neuron, account_id }
    }

    /// From raw 32 secret-key bytes.
    pub fn from_secret(secret: &[u8; 32]) -> Result<Self, Error> {
        SigningKey::from_bytes(secret.into())
            .map(Self::from_key)
            .map_err(|e| Error::Key(e.to_string()))
    }

    /// From a BIP-39 spell at the Cosmos path `m/44'/118'/0'/0/0` — the
    /// same derivation the old chains used, so the snapshot account is the
    /// one this key controls. Passphrase is `""` for a standard wallet.
    pub fn from_spell(spell: &str, passphrase: &str) -> Result<Self, Error> {
        mudra::spell::cosmos_key(spell.trim(), passphrase)
            .map(Self::from_key)
            .map_err(|e| Error::Key(e.to_string()))
    }

    /// Mint a fresh 24-word spell.
    pub fn generate_spell() -> Result<String, Error> {
        mudra::spell::generate().map_err(|e| Error::Key(e.to_string()))
    }

    /// The bech32 address under `hrp` (`pussy1…`, `bostrom1…`).
    pub fn address(&self, hrp: &str) -> Result<String, Error> {
        mudra::cosmos::address(&self.pubkey, hrp).map_err(|e| Error::Key(e.to_string()))
    }

    pub fn neuron_hex(&self) -> String {
        hex(&self.neuron)
    }

    /// The dormant owner genesis would have parked this key's balances
    /// under had the pubkey never appeared on the old chain.
    pub fn legacy_owner(&self, hrp: &str) -> [u8; 32] {
        tx::legacy_owner(hrp, &self.account_id)
    }

    /// Sign a transaction body for `chain`. The subject is `from` for
    /// transfer and cyberlink and `neuron_of(pubkey)` for claim; mudra
    /// refuses a subject this key does not own, so a transfer authored by
    /// another neuron never leaves this function signed.
    pub fn sign(&self, chain: &[u8; 32], tx: Tx) -> Result<SignedTx, Error> {
        tx.check_rules()?;
        let subject = match &tx {
            Tx::Transfer { from, .. } | Tx::Cyberlink { from, .. } => *from,
            Tx::Claim { .. } => self.neuron,
            Tx::Coinbase { .. } => return Err(Error::Rule("a coinbase is not signed".into())),
        };
        let mut bytes = tx.encode_unsigned();
        let statement = tx::statement(chain, &bytes);
        let sig = mudra::neuron::sign(&self.key, subject, statement)
            .map_err(|e| Error::Key(e.to_string()))?;
        bytes.extend_from_slice(&sig);
        let txid = tx::txid(chain, &bytes);
        Ok(SignedTx { chain: *chain, bytes, txid, tx })
    }

    /// A transfer of `amount` of `token` to `to`, at the fee floor unless
    /// `fee` says more (chain.md §2.3).
    pub fn transfer(
        &self,
        chain: &[u8; 32],
        nonce: u64,
        token: [u8; 32],
        amount: u64,
        to: [u8; 32],
        fee: Option<u64>,
    ) -> Result<SignedTx, Error> {
        let fee = fee.unwrap_or(Kind::Transfer.min_fee());
        self.sign(chain, Tx::Transfer { from: self.neuron, nonce, token, amount, to, fee })
    }

    /// A cyberlink `(p, q, valence)` by this neuron; `amount = 0` is the
    /// bare assertion and the common case (chain.md §2.4).
    #[allow(clippy::too_many_arguments)]
    pub fn cyberlink(
        &self,
        chain: &[u8; 32],
        nonce: u64,
        p: [u8; 32],
        q: [u8; 32],
        valence: i8,
        token: [u8; 32],
        amount: u64,
        fee: Option<u64>,
    ) -> Result<SignedTx, Error> {
        let fee = fee.unwrap_or(Kind::Cyberlink.min_fee());
        self.sign(
            chain,
            Tx::Cyberlink { from: self.neuron, nonce, token, amount, p, q, valence, fee },
        )
    }

    /// The claim that moves every balance of this key's dormant legacy
    /// owner to `neuron_of(pubkey)` (chain.md §2.8). The fee is paid by the
    /// dormant owner.
    pub fn claim(&self, chain: &[u8; 32], nonce: u64, fee: Option<u64>) -> Result<SignedTx, Error> {
        let fee = fee.unwrap_or(Kind::Claim.min_fee());
        self.sign(chain, Tx::Claim { account_id: self.account_id, nonce, fee })
    }
}

/// Decode and verify a wire transaction for `chain` the way the kernel's
/// step 7 does (signature and kind rules; nonce and balances are state).
/// Returns the body and, for claims, the signer's neuron.
pub fn verify(chain: &[u8; 32], bytes: &[u8]) -> Result<(Tx, Option<[u8; 32]>), Error> {
    let (tx, sig) = Tx::decode(bytes)?;
    tx.check_rules()?;
    let Some(sig) = sig else {
        return Ok((tx, None));
    };
    let unsigned = &bytes[..bytes.len() - tx::SIG_BYTES];
    let statement = tx::statement(chain, unsigned);
    let mut pubkey = [0u8; 33];
    pubkey.copy_from_slice(&sig[5..38]);
    let signer = tx::neuron_of(&pubkey);
    let subject = match &tx {
        Tx::Transfer { from, .. } | Tx::Cyberlink { from, .. } => *from,
        Tx::Claim { account_id, .. } => {
            if mudra::cosmos::account_id(&pubkey) != *account_id {
                return Err(Error::Key("claim account id is not derived by the signing key".into()));
            }
            signer
        }
        Tx::Coinbase { .. } => unreachable!("coinbase carries no signature"),
    };
    if !mudra::neuron::verify_statement(subject, statement, &sig) {
        return Err(Error::Key("NSIG1 signature does not verify for this chain and subject".into()));
    }
    Ok((tx, Some(signer)))
}

/// Lowercase hex of any bytes.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Exactly 32 bytes from 64 hex digits (an optional `0x` prefix allowed).
pub fn parse32(s: &str) -> Result<[u8; 32], Error> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Encoding(format!("expected 64 hex digits, got {s:?}")));
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("hex digits");
    }
    Ok(out)
}

/// The 32-byte card a user names: 64 hex digits as is, anything else as
/// `hemera(text)` — the particle of the label. The convention every soft3
/// route already uses.
pub fn card(text: &str) -> [u8; 32] {
    parse32(text).unwrap_or_else(|_| *hemera::hash(text.as_bytes()).as_bytes())
}

/// A token named by denom (`pussy`, `boot`) or by its 64-hex id.
pub fn token_arg(text: &str) -> [u8; 32] {
    parse32(text).unwrap_or_else(|_| tx::token(text))
}
