//! The genesis file: one tade stream, dialect `urn:cyberia:chain-genesis:1`
//! (`specs/chain.md` §4.1). Its hemera particle is the chain id.
//!
//! ```text
//! (*, k)  "urn:cyberia:chain-genesis:1"
//! (~, p)  params      version u32 ‖ name ‖ hrp ‖ denom ‖ time u64 ‖ bits u32 ‖ subsidy u64 ‖ checkpoint_key [33] ‖ checkpoint_sunset u64
//!                     ‖ pool: format u32 ‖ depth u8 ‖ anchor_window u32 ‖ tags [32]   (the frozen pool formats, chain.md §11.9)
//! ($, b)  balance     owner [32] ‖ token [32] ‖ amount u64              (sorted by (owner, token), unique, amount > 0)
//! (#, l)  link        neuron [32] ‖ p [32] ‖ q [32] ‖ height u64         (source order, duplicates kept)
//! ```
//!
//! Strings are `len: u8 ‖ utf8`; integers little-endian; a frame is
//! `0x1F ‖ sigil ‖ render ‖ LEB128(len) ‖ data` exactly as `tade::Chunk::encode`.

use std::io;

use super::Id;
use crate::chain::pool::{POOL_BYTES, Pool};

pub const DIALECT: &str = "urn:cyberia:chain-genesis:1";
pub const VERSION: u32 = 1;
pub const GENESIS_BITS: u32 = 0x1e01_0000;
pub const CHECKPOINT_SUNSET: u64 = 525_600;
/// `BLOCKS_PER_YEAR / 1 %` — `SUBSIDY = floor(S0 / SUBSIDY_DIVISOR)` (chain.md §3.5).
pub const SUBSIDY_DIVISOR: u128 = 105_120_000;

pub const SIGIL_PARAMS: u8 = tade::sigil::SIG; // '~'
pub const SIGIL_BALANCE: u8 = tade::sigil::BUC; // '$'
pub const SIGIL_LINK: u8 = tade::sigil::HAX; // '#'
pub const RENDER_PARAMS: u8 = b'p';
pub const RENDER_BALANCE: u8 = b'b';
pub const RENDER_LINK: u8 = b'l';

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    pub version: u32,
    pub name: String,
    pub hrp: String,
    pub denom: String,
    pub time: u64,
    pub bits: u32,
    pub subsidy: u64,
    pub checkpoint_key: [u8; 33],
    pub checkpoint_sunset: u64,
    /// The pool's frozen formats (rule set 2, off at launch): the chain id
    /// commits to them through this record.
    pub pool: Pool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance {
    pub owner: Id,
    pub token: Id,
    pub amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    pub neuron: Id,
    pub p: Id,
    pub q: Id,
    pub height: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisFile {
    pub params: Params,
    pub balances: Vec<Balance>,
    pub links: Vec<Link>,
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

fn put_str(out: &mut Vec<u8>, s: &str) -> io::Result<()> {
    let len = u8::try_from(s.len()).map_err(|_| invalid(format!("string over 255 bytes: {s:?}")))?;
    out.push(len);
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

impl Params {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut d = Vec::with_capacity(96);
        d.extend_from_slice(&self.version.to_le_bytes());
        put_str(&mut d, &self.name)?;
        put_str(&mut d, &self.hrp)?;
        put_str(&mut d, &self.denom)?;
        d.extend_from_slice(&self.time.to_le_bytes());
        d.extend_from_slice(&self.bits.to_le_bytes());
        d.extend_from_slice(&self.subsidy.to_le_bytes());
        d.extend_from_slice(&self.checkpoint_key);
        d.extend_from_slice(&self.checkpoint_sunset.to_le_bytes());
        d.extend_from_slice(&self.pool.encode());
        Ok(d)
    }

    pub fn decode(d: &[u8]) -> io::Result<Self> {
        let mut r = Reader(d);
        let version = r.u32()?;
        let name = r.str()?;
        let hrp = r.str()?;
        let denom = r.str()?;
        let time = r.u64()?;
        let bits = r.u32()?;
        let subsidy = r.u64()?;
        let checkpoint_key = r.array::<33>()?;
        let checkpoint_sunset = r.u64()?;
        let pool = Pool::decode(r.take(POOL_BYTES)?)?;
        r.end()?;
        Ok(Self { version, name, hrp, denom, time, bits, subsidy, checkpoint_key, checkpoint_sunset, pool })
    }
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < n {
            return Err(invalid("truncated frame"));
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        Ok(self.take(N)?.try_into().unwrap())
    }
    fn str(&mut self) -> io::Result<String> {
        let len = self.take(1)?[0] as usize;
        let s = self.take(len)?;
        String::from_utf8(s.to_vec()).map_err(|_| invalid("string not utf-8"))
    }
    fn end(&self) -> io::Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid(format!("{} trailing bytes in frame", self.0.len())))
        }
    }
}

fn frame(sigil: u8, render: u8, data: Vec<u8>) -> Vec<u8> {
    tade::Chunk::new(sigil, render, tade::bytes::Bytes::from(data)).encode()
}

impl GenesisFile {
    /// Canonical bytes. Checks the canonical order before writing.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        self.check()?;
        let mut out = Vec::with_capacity(64 + 76 * self.balances.len() + 108 * self.links.len());
        out.extend(frame(b'*', b'k', DIALECT.as_bytes().to_vec()));
        out.extend(frame(SIGIL_PARAMS, RENDER_PARAMS, self.params.encode()?));
        for b in &self.balances {
            let mut d = Vec::with_capacity(72);
            d.extend_from_slice(&b.owner);
            d.extend_from_slice(&b.token);
            d.extend_from_slice(&b.amount.to_le_bytes());
            out.extend(frame(SIGIL_BALANCE, RENDER_BALANCE, d));
        }
        for l in &self.links {
            let mut d = Vec::with_capacity(104);
            d.extend_from_slice(&l.neuron);
            d.extend_from_slice(&l.p);
            d.extend_from_slice(&l.q);
            d.extend_from_slice(&l.height.to_le_bytes());
            out.extend(frame(SIGIL_LINK, RENDER_LINK, d));
        }
        Ok(out)
    }

    /// Strict decoder: the dialect frame first, exactly one `p`, then `b`
    /// frames in canonical order, then `l` frames, nothing else, no
    /// trailing bytes, no unknown frames.
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut pos = 0usize;
        let mut frames = Vec::new();
        while pos < bytes.len() {
            if bytes[pos] != tade::MARKER {
                return Err(invalid(format!("byte {pos}: expected frame marker")));
            }
            if bytes.len() < pos + 3 {
                return Err(invalid("truncated frame header"));
            }
            let sigil = bytes[pos + 1];
            let render = bytes[pos + 2];
            let (len, next) = tade::decode_varint(bytes, pos + 3).ok_or_else(|| invalid("bad frame length"))?;
            let len = usize::try_from(len).map_err(|_| invalid("frame too long"))?;
            let end = next.checked_add(len).filter(|e| *e <= bytes.len()).ok_or_else(|| invalid("truncated frame"))?;
            frames.push((sigil, render, &bytes[next..end]));
            pos = end;
        }
        let mut it = frames.into_iter();
        match it.next() {
            Some((b'*', b'k', d)) if d == DIALECT.as_bytes() => {}
            _ => return Err(invalid(format!("first frame is not (*, k) {DIALECT}"))),
        }
        let params = match it.next() {
            Some((SIGIL_PARAMS, RENDER_PARAMS, d)) => Params::decode(d)?,
            _ => return Err(invalid("second frame is not (~, p)")),
        };
        let mut balances = Vec::new();
        let mut links = Vec::new();
        for (sigil, render, d) in it {
            match (sigil, render) {
                (SIGIL_BALANCE, RENDER_BALANCE) => {
                    if !links.is_empty() {
                        return Err(invalid("balance frame after a link frame"));
                    }
                    if d.len() != 72 {
                        return Err(invalid(format!("balance frame of {} bytes", d.len())));
                    }
                    balances.push(Balance {
                        owner: d[..32].try_into().unwrap(),
                        token: d[32..64].try_into().unwrap(),
                        amount: u64::from_le_bytes(d[64..72].try_into().unwrap()),
                    });
                }
                (SIGIL_LINK, RENDER_LINK) => {
                    if d.len() != 104 {
                        return Err(invalid(format!("link frame of {} bytes", d.len())));
                    }
                    links.push(Link {
                        neuron: d[..32].try_into().unwrap(),
                        p: d[32..64].try_into().unwrap(),
                        q: d[64..96].try_into().unwrap(),
                        height: u64::from_le_bytes(d[96..104].try_into().unwrap()),
                    });
                }
                (s, r) => return Err(invalid(format!("unexpected frame ({}, {})", s as char, r as char))),
            }
        }
        let file = Self { params, balances, links };
        file.check()?;
        Ok(file)
    }

    /// The canonical-form rules of §4.1.
    pub fn check(&self) -> io::Result<()> {
        if self.params.version != VERSION {
            return Err(invalid(format!("params version {}", self.params.version)));
        }
        self.params.pool.check()?;
        for w in self.balances.windows(2) {
            let a = (&w[0].owner, &w[0].token);
            let b = (&w[1].owner, &w[1].token);
            if a >= b {
                return Err(invalid("balances not strictly sorted by (owner, token)"));
            }
        }
        if self.balances.iter().any(|b| b.amount == 0) {
            return Err(invalid("zero balance frame"));
        }
        Ok(())
    }

    /// Genesis supply of the native coin: Σ `b` of `token(denom)`.
    pub fn native_supply(&self) -> u128 {
        let native = super::token_of(&self.params.denom);
        self.balances.iter().filter(|b| b.token == native).map(|b| u128::from(b.amount)).sum()
    }
}

/// `SUBSIDY = floor(S0 / 105,120,000)`.
pub fn subsidy_of(s0: u128) -> io::Result<u64> {
    u64::try_from(s0 / SUBSIDY_DIVISOR).map_err(|_| invalid("subsidy does not fit u64"))
}

/// The 168-byte block-0 header of chain.md §1.1/§4.3 and its id.
pub fn block0_header(time: u64, state_root: &Id, chain: &Id) -> [u8; 168] {
    let mut h = [0u8; 168];
    h[0..4].copy_from_slice(&VERSION.to_le_bytes());
    // height 0, time
    h[12..20].copy_from_slice(&time.to_le_bytes());
    h[20..24].copy_from_slice(&GENESIS_BITS.to_le_bytes());
    // nonce 0, prev 0
    h[72..104].copy_from_slice(chain); // tx_root = chain for genesis
    h[104..136].copy_from_slice(state_root);
    h[136..168].copy_from_slice(chain);
    h
}

/// `block_id = hemera(header_bytes)`.
pub fn block_id(header: &[u8; 168]) -> Id {
    *hemera::hash(header).as_bytes()
}

/// Generic header encoder (for the conformance vector test).
#[cfg(test)]
pub fn header_bytes(
    version: u32,
    height: u64,
    time: u64,
    bits: u32,
    nonce: [u8; 16],
    prev: &Id,
    tx_root: &Id,
    state_root: &Id,
    chain: &Id,
) -> [u8; 168] {
    let mut h = [0u8; 168];
    h[0..4].copy_from_slice(&version.to_le_bytes());
    h[4..12].copy_from_slice(&height.to_le_bytes());
    h[12..20].copy_from_slice(&time.to_le_bytes());
    h[20..24].copy_from_slice(&bits.to_le_bytes());
    h[24..40].copy_from_slice(&nonce);
    h[40..72].copy_from_slice(prev);
    h[72..104].copy_from_slice(tx_root);
    h[104..136].copy_from_slice(state_root);
    h[136..168].copy_from_slice(chain);
    h
}
