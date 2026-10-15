//! Genesis: the tade stream of §4.1, its application (§4.1) and block 0 (§4.3).
//!
//! A genesis file is one tade stream, dialect `urn:cyberia:chain-genesis:1`:
//! the `(*, k)` declaration, exactly one `p` frame, then `b` frames sorted by
//! `(owner, token)`, then `l` frames in source order, nothing else. Frames
//! are tade chunks (`0x1f ‖ type ‖ render ‖ varint(len) ‖ payload`) with the
//! type byte in the sigil slot and render `b` (binary) for `p`/`b`/`l`. The
//! decoder is strict: no stray bytes, no re-declaration, no unknown frame.
//! Its particle (`hemera(bytes)`) is the chain id.
//!
//! The pipeline that derives the real files from the burial snapshots is
//! `soft3 genesis build` (specs/genesis.md, soft3#183); this module is the
//! consensus-side reader and applier that pipeline's output must satisfy.

use super::header::{Header, decode_bits, hash32};
use super::params::{GENESIS_DIALECT, subsidy};
use super::sigs::token;
use super::state::{Id, StateBackend, StateError};

const MARKER: u8 = 0x1f;
const RENDER_TOKEN: u8 = b'k';
const RENDER_BINARY: u8 = b'b';

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
    pub owner: Id,
    pub token: Id,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub neuron: Id,
    pub p: Id,
    pub q: Id,
    pub height: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub params: Params,
    pub balances: Vec<Balance>,
    pub links: Vec<Link>,
    /// the file bytes; `hemera(bytes)` is the chain id
    pub bytes: Vec<u8>,
    pub chain_id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisError {
    Frame(usize),
    Dialect,
    Params(&'static str),
    Order(usize),
    Balance(usize, &'static str),
    Trailing,
    Supply,
    Apply(StateError),
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenesisError::Frame(at) => write!(f, "malformed frame at byte {at}"),
            GenesisError::Dialect => write!(f, "first frame is not the {GENESIS_DIALECT} declaration"),
            GenesisError::Params(w) => write!(f, "params frame: {w}"),
            GenesisError::Order(i) => write!(f, "frame {i} out of canonical order"),
            GenesisError::Balance(i, w) => write!(f, "balance frame {i}: {w}"),
            GenesisError::Trailing => write!(f, "bytes after the last frame"),
            GenesisError::Supply => write!(f, "subsidy field disagrees with floor(S0 / 105,120,000)"),
            GenesisError::Apply(e) => write!(f, "apply: {e}"),
        }
    }
}

fn frame(out: &mut Vec<u8>, kind: u8, render: u8, payload: &[u8]) {
    out.push(MARKER);
    out.push(kind);
    out.push(render);
    tade::encode_varint(payload.len() as u64, out);
    out.extend_from_slice(payload);
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.push(s.len() as u8);
    out.extend_from_slice(s.as_bytes());
}

impl Params {
    pub fn encode(&self) -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(&self.version.to_le_bytes());
        put_str(&mut p, &self.name);
        put_str(&mut p, &self.hrp);
        put_str(&mut p, &self.denom);
        p.extend_from_slice(&self.time.to_le_bytes());
        p.extend_from_slice(&self.bits.to_le_bytes());
        p.extend_from_slice(&self.subsidy.to_le_bytes());
        p.extend_from_slice(&self.checkpoint_key);
        p.extend_from_slice(&self.checkpoint_sunset.to_le_bytes());
        p
    }

    fn decode(b: &[u8]) -> Result<Params, GenesisError> {
        let mut at = 0usize;
        let take = |at: &mut usize, n: usize| -> Result<&[u8], GenesisError> {
            let s = b.get(*at..*at + n).ok_or(GenesisError::Params("truncated"))?;
            *at += n;
            Ok(s)
        };
        let version = u32::from_le_bytes(take(&mut at, 4)?.try_into().expect("4"));
        if version != 1 {
            return Err(GenesisError::Params("version"));
        }
        let string = |at: &mut usize| -> Result<String, GenesisError> {
            let n = take(at, 1)?[0] as usize;
            let s = take(at, n)?;
            String::from_utf8(s.to_vec()).map_err(|_| GenesisError::Params("utf8"))
        };
        let name = string(&mut at)?;
        let hrp = string(&mut at)?;
        let denom = string(&mut at)?;
        if name.is_empty() || hrp.is_empty() || denom.is_empty() {
            return Err(GenesisError::Params("empty string"));
        }
        let time = u64::from_le_bytes(take(&mut at, 8)?.try_into().expect("8"));
        let bits = u32::from_le_bytes(take(&mut at, 4)?.try_into().expect("4"));
        decode_bits(bits).map_err(|_| GenesisError::Params("bits"))?;
        let subsidy = u64::from_le_bytes(take(&mut at, 8)?.try_into().expect("8"));
        let checkpoint_key: [u8; 33] = take(&mut at, 33)?.try_into().expect("33");
        let checkpoint_sunset = u64::from_le_bytes(take(&mut at, 8)?.try_into().expect("8"));
        if at != b.len() {
            return Err(GenesisError::Params("trailing bytes"));
        }
        Ok(Params { version, name, hrp, denom, time, bits, subsidy, checkpoint_key, checkpoint_sunset })
    }

    /// all-zero key means no checkpoints (§4.1)
    pub fn checkpoints_enabled(&self) -> bool {
        self.checkpoint_key != [0; 33]
    }
}

impl Genesis {
    /// Build the canonical bytes: balances are sorted and deduplicated here;
    /// links keep the given order. `subsidy` is written from §3.5.
    pub fn encode(mut params: Params, mut balances: Vec<Balance>, links: &[Link]) -> Vec<u8> {
        balances.sort_by(|a, b| (a.owner, a.token).cmp(&(b.owner, b.token)));
        balances.dedup_by(|a, b| a.owner == b.owner && a.token == b.token);
        balances.retain(|b| b.amount > 0);
        let native = token(&params.denom);
        let s0 = balances.iter().filter(|b| b.token == native).map(|b| b.amount).fold(0u64, |a, x| a.saturating_add(x));
        params.subsidy = subsidy(s0);
        let mut out = Vec::new();
        frame(&mut out, b'*', RENDER_TOKEN, GENESIS_DIALECT.as_bytes());
        frame(&mut out, b'p', RENDER_BINARY, &params.encode());
        for b in &balances {
            let mut p = Vec::with_capacity(72);
            p.extend_from_slice(&b.owner);
            p.extend_from_slice(&b.token);
            p.extend_from_slice(&b.amount.to_le_bytes());
            frame(&mut out, b'b', RENDER_BINARY, &p);
        }
        for l in links {
            let mut p = Vec::with_capacity(104);
            p.extend_from_slice(&l.neuron);
            p.extend_from_slice(&l.p);
            p.extend_from_slice(&l.q);
            p.extend_from_slice(&l.height.to_le_bytes());
            frame(&mut out, b'l', RENDER_BINARY, &p);
        }
        out
    }

    /// Strict decode of a genesis file.
    pub fn decode(bytes: &[u8]) -> Result<Genesis, GenesisError> {
        let mut at = 0usize;
        let mut frames: Vec<(u8, u8, &[u8])> = Vec::new();
        while at < bytes.len() {
            let start = at;
            if bytes.len() < at + 4 || bytes[at] != MARKER {
                return Err(GenesisError::Frame(start));
            }
            let (kind, render) = (bytes[at + 1], bytes[at + 2]);
            let (len, next) = tade::decode_varint(bytes, at + 3).ok_or(GenesisError::Frame(start))?;
            let end = next.checked_add(len as usize).filter(|e| *e <= bytes.len()).ok_or(GenesisError::Frame(start))?;
            frames.push((kind, render, &bytes[next..end]));
            at = end;
        }
        let Some((b'*', RENDER_TOKEN, dialect)) = frames.first() else {
            return Err(GenesisError::Dialect);
        };
        if *dialect != GENESIS_DIALECT.as_bytes() {
            return Err(GenesisError::Dialect);
        }
        let Some((b'p', RENDER_BINARY, params)) = frames.get(1) else {
            return Err(GenesisError::Params("missing"));
        };
        let params = Params::decode(params)?;
        let mut balances = Vec::new();
        let mut links = Vec::new();
        let mut phase = 0u8;
        for (i, (kind, render, payload)) in frames.iter().enumerate().skip(2) {
            match (*kind, *render, payload.len()) {
                (b'b', RENDER_BINARY, 72) if phase == 0 => {
                    let b = Balance {
                        owner: payload[..32].try_into().expect("32"),
                        token: payload[32..64].try_into().expect("32"),
                        amount: u64::from_le_bytes(payload[64..].try_into().expect("8")),
                    };
                    if b.amount == 0 {
                        return Err(GenesisError::Balance(i, "zero amount"));
                    }
                    if let Some(prev) = balances.last() {
                        let (prev, cur): (&Balance, &Balance) = (prev, &b);
                        if (cur.owner, cur.token) <= (prev.owner, prev.token) {
                            return Err(GenesisError::Balance(i, "not sorted or duplicate"));
                        }
                    }
                    balances.push(b);
                }
                (b'l', RENDER_BINARY, 104) => {
                    phase = 1;
                    links.push(Link {
                        neuron: payload[..32].try_into().expect("32"),
                        p: payload[32..64].try_into().expect("32"),
                        q: payload[64..96].try_into().expect("32"),
                        height: u64::from_le_bytes(payload[96..].try_into().expect("8")),
                    });
                }
                _ => return Err(GenesisError::Order(i)),
            }
        }
        let g = Genesis { params, balances, links, bytes: bytes.to_vec(), chain_id: hash32(bytes) };
        if g.params.subsidy != subsidy(g.s0()) {
            return Err(GenesisError::Supply);
        }
        Ok(g)
    }

    pub fn native_token(&self) -> Id {
        token(&self.params.denom)
    }

    /// `S0`: the genesis supply of the native coin.
    pub fn s0(&self) -> u64 {
        let native = self.native_token();
        self.balances.iter().filter(|b| b.token == native).map(|b| b.amount).fold(0u64, |a, x| a.saturating_add(x))
    }

    /// The ASERT anchor (§1.5).
    pub fn anchor(&self) -> super::daa::Anchor {
        super::daa::Anchor { target: decode_bits(self.params.bits).expect("checked on decode"), time: self.params.time }
    }

    /// Apply to an empty state (§4.1): balances and coin totals, every link
    /// as a bbg insert with `token = τ_native`, amount 0, valence 0 at the
    /// chain's height 0 (the old height rides in the record for indexers),
    /// a neuron record with nonce 0 for every owner and author.
    pub fn apply<S: StateBackend>(&self, s: &mut S) -> Result<(), GenesisError> {
        let native = self.native_token();
        for b in &self.balances {
            s.ensure_neuron(&b.owner);
            s.credit(&b.owner, &b.token, b.amount).map_err(GenesisError::Apply)?;
            s.mint(&b.token, b.amount).map_err(GenesisError::Apply)?;
        }
        for l in &self.links {
            s.ensure_neuron(&l.neuron);
            s.link(&l.neuron, &l.p, &l.q, &native, 0, 0, l.height).map_err(GenesisError::Apply)?;
        }
        s.set_height(0);
        Ok(())
    }

    /// Block 0 (§4.3).
    pub fn block0(&self, state_root: Id) -> Header {
        Header {
            version: 1,
            height: 0,
            time: self.params.time,
            bits: self.params.bits,
            nonce: [0; 16],
            prev: [0; 32],
            tx_root: self.chain_id,
            state_root,
            chain: self.chain_id,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use super::super::bbg_state::BbgBacked;
    use super::super::params::GENESIS_BITS;

    /// The conformance net of chain-conformance §8: one balance, no links.
    pub(crate) fn conformance_params() -> Params {
        Params {
            version: 1,
            name: "conformance".into(),
            hrp: "pussy".into(),
            denom: "pussy".into(),
            time: 0,
            bits: GENESIS_BITS,
            subsidy: 0,
            checkpoint_key: [0; 33],
            checkpoint_sunset: 0,
        }
    }

    pub(crate) fn conformance_genesis() -> Genesis {
        let bytes = Genesis::encode(
            conformance_params(),
            vec![Balance { owner: [0x66; 32], token: token("pussy"), amount: 1_000_000 }],
            &[],
        );
        Genesis::decode(&bytes).unwrap()
    }

    #[test]
    fn conformance_net_round_trips_and_applies() {
        let g = conformance_genesis();
        assert_eq!(g.s0(), 1_000_000);
        assert_eq!(g.params.subsidy, 0, "1,000,000 / 105,120,000 floors to 0");
        assert_eq!(g.chain_id, hash32(&g.bytes));
        let mut s = BbgBacked::new();
        g.apply(&mut s).unwrap();
        assert_eq!(s.balance(&[0x66; 32], &token("pussy")), 1_000_000);
        assert_eq!(s.supply(&token("pussy")), 1_000_000);
        assert_eq!(s.nonce(&[0x66; 32]), 0);
        assert!(s.has_neuron(&[0x66; 32]));
        let b0 = g.block0(s.root());
        assert_eq!(b0.tx_root, g.chain_id);
        assert_eq!(b0.height, 0);
        assert_eq!(Genesis::decode(&g.bytes).unwrap(), g);
    }

    #[test]
    fn strict_decoder_rejects_every_deviation() {
        let g = conformance_genesis();
        let mut p = g.params.clone();
        p.subsidy = 1;
        let bytes = Genesis::encode(p.clone(), g.balances.clone(), &[]);
        assert!(Genesis::decode(&bytes).is_ok(), "encode recomputes the subsidy");
        let mut tampered = bytes.clone();
        tampered.push(0);
        assert_eq!(Genesis::decode(&tampered), Err(GenesisError::Frame(bytes.len())));
        let mut no_dialect = bytes.clone();
        no_dialect[1] = b'#';
        assert_eq!(Genesis::decode(&no_dialect), Err(GenesisError::Dialect));
        // unsorted balances: hand-build two b frames in the wrong order
        let mut dup = Vec::new();
        frame(&mut dup, b'*', RENDER_TOKEN, GENESIS_DIALECT.as_bytes());
        frame(&mut dup, b'p', RENDER_BINARY, &g.params.encode());
        for owner in [[0x77u8; 32], [0x66; 32]] {
            let mut pl = owner.to_vec();
            pl.extend_from_slice(&token("pussy"));
            pl.extend_from_slice(&500_000u64.to_le_bytes());
            frame(&mut dup, b'b', RENDER_BINARY, &pl);
        }
        assert_eq!(Genesis::decode(&dup), Err(GenesisError::Balance(3, "not sorted or duplicate")));
        // a balance after a link is out of order
        let mut late = Genesis::encode(g.params.clone(), g.balances.clone(), &[Link { neuron: [1; 32], p: [2; 32], q: [3; 32], height: 5 }]);
        let b_frame = {
            let mut pl = [0x99u8; 32].to_vec();
            pl.extend_from_slice(&token("pussy"));
            pl.extend_from_slice(&1u64.to_le_bytes());
            let mut f = Vec::new();
            frame(&mut f, b'b', RENDER_BINARY, &pl);
            f
        };
        late.extend_from_slice(&b_frame);
        assert_eq!(Genesis::decode(&late), Err(GenesisError::Order(4)));
        // wrong subsidy on the wire
        let mut wrong = Vec::new();
        frame(&mut wrong, b'*', RENDER_TOKEN, GENESIS_DIALECT.as_bytes());
        frame(&mut wrong, b'p', RENDER_BINARY, &p.encode());
        assert_eq!(Genesis::decode(&wrong), Err(GenesisError::Supply));
    }

    #[test]
    fn links_apply_with_zero_amount_and_authors_get_records() {
        let links = vec![
            Link { neuron: [1; 32], p: [2; 32], q: [3; 32], height: 5 },
            Link { neuron: [1; 32], p: [2; 32], q: [3; 32], height: 6 },
        ];
        let bytes = Genesis::encode(conformance_params(), vec![Balance { owner: [0x66; 32], token: token("pussy"), amount: 7 }], &links);
        let g = Genesis::decode(&bytes).unwrap();
        assert_eq!(g.links, links, "duplicates kept");
        let mut s = BbgBacked::new();
        g.apply(&mut s).unwrap();
        assert!(s.has_neuron(&[1; 32]));
        assert_eq!(s.axon_count(), 1);
        assert_eq!(s.token_sum(&token("pussy")), 7);
    }
}
