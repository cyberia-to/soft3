//! The chain dialect over tade — chain-wire.md §2–3. Every stream starts
//! with the dialect declaration `(*, k) urn:cyberia:chain:1`; every other
//! frame is `(#, letter)` with the letter of the spec's tables. Integers are
//! little-endian, lists count-prefixed, sizes bounded before allocation.

use super::codec::{Header, Id, DecodeError, HEADER_BYTES, MAX_BLOCK_BYTES, SIG_BYTES};

pub const DIALECT: &str = "urn:cyberia:chain:1";
pub const SIGIL: u8 = b'#';
pub const CONTROL: u8 = b'*';
pub const MAX_HEADERS_PER_RESPONSE: usize = 2000;
pub const MAX_RANGE: u16 = 128;
pub const MAX_BLOCKS_BY_ID: usize = 128;
pub const MAX_TXS_PER_REQUEST: usize = 256;
pub const MAX_GOSSIP_BYTES: usize = 4096;
pub const MAX_RESPONSE_BYTES: usize = MAX_BLOCK_BYTES + 16;
pub const MAX_REQUEST_BYTES: usize = 16 + 256 * 32;
pub const STATUS_BYTES: usize = 32 + 2 + 8 + 32 + 32 + 32;
pub const CHECKPOINT_BYTES: usize = 8 + 32 + SIG_BYTES;

pub const ERR_UNKNOWN: u8 = 1;
pub const ERR_PRUNED: u8 = 2;
pub const ERR_LIMIT: u8 = 3;
pub const ERR_WRONG_CHAIN: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Status {
    pub chain: Id,
    pub rule_set: u16,
    pub height: u64,
    pub tip: Id,
    pub chainwork: [u8; 32],
    pub genesis: Id,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Checkpoint {
    pub height: u64,
    pub block_id: Id,
    pub sig: Vec<u8>,
}

impl Checkpoint {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.height.to_le_bytes().to_vec();
        out.extend_from_slice(&self.block_id);
        out.extend_from_slice(&self.sig);
        out
    }

    pub fn decode(b: &[u8]) -> Result<Self, FrameError> {
        if b.len() != CHECKPOINT_BYTES {
            return Err(FrameError::Length);
        }
        Ok(Checkpoint {
            height: u64::from_le_bytes(b[0..8].try_into().expect("8")),
            block_id: b[8..40].try_into().expect("32"),
            sig: b[40..].to_vec(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Frame {
    Dialect(String),
    Status(Status),
    GetHeaders { locator: Vec<Id>, stop: Id },
    Headers(Vec<Header>),
    GetRange { from: u64, count: u16 },
    GetBlocks(Vec<Id>),
    Block { header: Header, body: Vec<u8> },
    GetTxs(Vec<Id>),
    Tx(Vec<u8>),
    GetCheckpoints { from_height: u64 },
    Checkpoint(Checkpoint),
    End,
    Error { code: u8, reason: String },
    HeaderGossip(Header),
    Shadow(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    Marker,
    Varint,
    Truncated,
    Oversize(usize),
    Trailing,
    UnknownType(u8, u8),
    Length,
    Count,
    Utf8,
    Body(DecodeError),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Marker => write!(f, "frame marker missing"),
            FrameError::Varint => write!(f, "malformed size varint"),
            FrameError::Truncated => write!(f, "truncated frame"),
            FrameError::Oversize(n) => write!(f, "frame of {n} bytes over size"),
            FrameError::Trailing => write!(f, "bytes after frame"),
            FrameError::UnknownType(s, r) => write!(f, "unknown frame type ({s:#04x},{r:#04x})"),
            FrameError::Length => write!(f, "payload length"),
            FrameError::Count => write!(f, "count out of range"),
            FrameError::Utf8 => write!(f, "invalid utf-8"),
            FrameError::Body(e) => write!(f, "{e}"),
        }
    }
}

fn ids(b: &[u8], count: usize, max: usize) -> Result<Vec<Id>, FrameError> {
    if count > max || b.len() != count * 32 {
        return Err(FrameError::Count);
    }
    Ok(b.chunks_exact(32).map(|c| c.try_into().expect("32")).collect())
}

impl Frame {
    pub fn letter(&self) -> u8 {
        match self {
            Frame::Dialect(_) => b'k',
            Frame::Status(_) => b's',
            Frame::GetHeaders { .. } => b'g',
            Frame::Headers(_) => b'h',
            Frame::GetRange { .. } => b'r',
            Frame::GetBlocks(_) => b'b',
            Frame::Block { .. } => b'B',
            Frame::GetTxs(_) => b't',
            Frame::Tx(_) => b'T',
            Frame::GetCheckpoints { .. } => b'k',
            Frame::Checkpoint(_) => b'C',
            Frame::End => b'e',
            Frame::Error { .. } => b'x',
            Frame::HeaderGossip(_) => b'H',
            Frame::Shadow(_) => b'X',
        }
    }

    pub fn is_request(&self) -> bool {
        matches!(
            self,
            Frame::Status(_) | Frame::GetHeaders { .. } | Frame::GetRange { .. } | Frame::GetBlocks(_) | Frame::GetTxs(_) | Frame::GetCheckpoints { .. }
        )
    }

    pub fn payload(&self) -> Vec<u8> {
        match self {
            Frame::Dialect(urn) => urn.as_bytes().to_vec(),
            Frame::Status(s) => {
                let mut out = s.chain.to_vec();
                out.extend_from_slice(&s.rule_set.to_le_bytes());
                out.extend_from_slice(&s.height.to_le_bytes());
                out.extend_from_slice(&s.tip);
                out.extend_from_slice(&s.chainwork);
                out.extend_from_slice(&s.genesis);
                out
            }
            Frame::GetHeaders { locator, stop } => {
                let mut out = vec![locator.len() as u8];
                for id in locator {
                    out.extend_from_slice(id);
                }
                out.extend_from_slice(stop);
                out
            }
            Frame::Headers(headers) => {
                let mut out = (headers.len() as u16).to_le_bytes().to_vec();
                for h in headers {
                    out.extend_from_slice(&h.encode());
                }
                out
            }
            Frame::GetRange { from, count } => {
                let mut out = from.to_le_bytes().to_vec();
                out.extend_from_slice(&count.to_le_bytes());
                out
            }
            Frame::GetBlocks(ids) => {
                let mut out = vec![ids.len() as u8];
                for id in ids {
                    out.extend_from_slice(id);
                }
                out
            }
            Frame::Block { header, body } => {
                let mut out = header.encode().to_vec();
                out.extend_from_slice(body);
                out
            }
            Frame::GetTxs(ids) => {
                let mut out = (ids.len() as u16).to_le_bytes().to_vec();
                for id in ids {
                    out.extend_from_slice(id);
                }
                out
            }
            Frame::Tx(bytes) | Frame::Shadow(bytes) => bytes.clone(),
            Frame::GetCheckpoints { from_height } => from_height.to_le_bytes().to_vec(),
            Frame::Checkpoint(c) => c.encode(),
            Frame::End => Vec::new(),
            Frame::Error { code, reason } => {
                let mut out = vec![*code];
                out.extend_from_slice(reason.as_bytes());
                out
            }
            Frame::HeaderGossip(h) => h.encode().to_vec(),
        }
    }

    /// Tade bytes: `0x1F sigil render varint payload`.
    pub fn encode(&self) -> Vec<u8> {
        let sigil = if matches!(self, Frame::Dialect(_)) { CONTROL } else { SIGIL };
        tade::Chunk::new(sigil, self.letter(), bytes::Bytes::from(self.payload())).encode()
    }

    /// Interpret one tade chunk in this dialect.
    pub fn from_parts(sigil: u8, render: u8, b: &[u8]) -> Result<Frame, FrameError> {
        if sigil == CONTROL {
            return match render {
                b'k' => Ok(Frame::Dialect(String::from_utf8(b.to_vec()).map_err(|_| FrameError::Utf8)?)),
                _ => Err(FrameError::UnknownType(sigil, render)),
            };
        }
        if sigil != SIGIL {
            return Err(FrameError::UnknownType(sigil, render));
        }
        let u64_at = |at: usize| u64::from_le_bytes(b[at..at + 8].try_into().expect("8"));
        match render {
            b's' => {
                if b.len() != STATUS_BYTES {
                    return Err(FrameError::Length);
                }
                Ok(Frame::Status(Status {
                    chain: b[0..32].try_into().expect("32"),
                    rule_set: u16::from_le_bytes([b[32], b[33]]),
                    height: u64_at(34),
                    tip: b[42..74].try_into().expect("32"),
                    chainwork: b[74..106].try_into().expect("32"),
                    genesis: b[106..138].try_into().expect("32"),
                }))
            }
            b'g' => {
                if b.is_empty() {
                    return Err(FrameError::Length);
                }
                let count = b[0] as usize;
                if count == 0 || b.len() != 1 + count * 32 + 32 {
                    return Err(FrameError::Count);
                }
                let locator = ids(&b[1..1 + count * 32], count, 255)?;
                Ok(Frame::GetHeaders { locator, stop: b[1 + count * 32..].try_into().expect("32") })
            }
            b'h' => {
                if b.len() < 2 {
                    return Err(FrameError::Length);
                }
                let count = u16::from_le_bytes([b[0], b[1]]) as usize;
                if count > MAX_HEADERS_PER_RESPONSE || b.len() != 2 + count * HEADER_BYTES {
                    return Err(FrameError::Count);
                }
                let headers = b[2..]
                    .chunks_exact(HEADER_BYTES)
                    .map(|c| Header::decode(c).map_err(FrameError::Body))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Frame::Headers(headers))
            }
            b'r' => {
                if b.len() != 10 {
                    return Err(FrameError::Length);
                }
                Ok(Frame::GetRange { from: u64_at(0), count: u16::from_le_bytes([b[8], b[9]]) })
            }
            b'b' => {
                if b.is_empty() {
                    return Err(FrameError::Length);
                }
                Ok(Frame::GetBlocks(ids(&b[1..], b[0] as usize, MAX_BLOCKS_BY_ID)?))
            }
            b'B' => {
                if b.len() < HEADER_BYTES || b.len() > MAX_BLOCK_BYTES {
                    return Err(FrameError::Length);
                }
                let header = Header::decode(&b[..HEADER_BYTES]).map_err(FrameError::Body)?;
                Ok(Frame::Block { header, body: b[HEADER_BYTES..].to_vec() })
            }
            b't' => {
                if b.len() < 2 {
                    return Err(FrameError::Length);
                }
                let count = u16::from_le_bytes([b[0], b[1]]) as usize;
                Ok(Frame::GetTxs(ids(&b[2..], count, MAX_TXS_PER_REQUEST)?))
            }
            b'T' => {
                if b.is_empty() || b.len() > super::codec::MAX_TX_BYTES {
                    return Err(FrameError::Length);
                }
                Ok(Frame::Tx(b.to_vec()))
            }
            b'k' => {
                if b.len() != 8 {
                    return Err(FrameError::Length);
                }
                Ok(Frame::GetCheckpoints { from_height: u64_at(0) })
            }
            b'C' => Ok(Frame::Checkpoint(Checkpoint::decode(b)?)),
            b'e' => {
                if !b.is_empty() {
                    return Err(FrameError::Length);
                }
                Ok(Frame::End)
            }
            b'x' => {
                if b.is_empty() || b.len() > 256 {
                    return Err(FrameError::Length);
                }
                let reason = std::str::from_utf8(&b[1..]).map_err(|_| FrameError::Utf8)?.to_owned();
                Ok(Frame::Error { code: b[0], reason })
            }
            b'H' => Ok(Frame::HeaderGossip(Header::decode(b).map_err(FrameError::Body)?)),
            b'X' => {
                if b.len() > MAX_GOSSIP_BYTES {
                    return Err(FrameError::Length);
                }
                Ok(Frame::Shadow(b.to_vec()))
            }
            _ => Err(FrameError::UnknownType(sigil, render)),
        }
    }

    /// Exactly one frame, nothing before or after it, payload ≤ `max`.
    pub fn decode(bytes: &[u8], max: usize) -> Result<Frame, FrameError> {
        let (sigil, render, payload, used) = split(bytes, max)?;
        if used != bytes.len() {
            return Err(FrameError::Trailing);
        }
        Frame::from_parts(sigil, render, payload)
    }
}

/// Parse the tade envelope at the start of `bytes`: `(sigil, render, payload, bytes used)`.
pub fn split(bytes: &[u8], max: usize) -> Result<(u8, u8, &[u8], usize), FrameError> {
    if bytes.is_empty() || bytes[0] != tade::MARKER {
        return Err(FrameError::Marker);
    }
    if bytes.len() < 4 {
        return Err(FrameError::Truncated);
    }
    let (len, start) = varint(&bytes[3..])?;
    if len > max as u64 {
        return Err(FrameError::Oversize(len as usize));
    }
    let start = 3 + start;
    let end = start + len as usize;
    if end > bytes.len() {
        return Err(FrameError::Truncated);
    }
    Ok((bytes[1], bytes[2], &bytes[start..end], end))
}

/// LEB128 with the tade bound (10 bytes); returns `(value, bytes used)`.
pub fn varint(bytes: &[u8]) -> Result<(u64, usize), FrameError> {
    let mut value = 0u64;
    for (i, byte) in bytes.iter().enumerate().take(10) {
        if i == 9 && *byte > 1 {
            return Err(FrameError::Varint);
        }
        value |= ((*byte & 0x7f) as u64) << (7 * i);
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
    }
    if bytes.len() < 10 { Err(FrameError::Truncated) } else { Err(FrameError::Varint) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Header {
        Header {
            version: 1, height: 1, time: 1762349000, bits: 0x1d100000, nonce: [0; 16],
            prev: [0x11; 32], tx_root: [0x22; 32], state_root: [0x33; 32], chain: [0x44; 32],
        }
    }

    fn all() -> Vec<Frame> {
        vec![
            Frame::Dialect(DIALECT.into()),
            Frame::Status(Status { chain: [1; 32], rule_set: 1, height: 7, tip: [2; 32], chainwork: [3; 32], genesis: [4; 32] }),
            Frame::GetHeaders { locator: vec![[5; 32], [6; 32]], stop: [0; 32] },
            Frame::Headers(vec![header(), header()]),
            Frame::Headers(vec![]),
            Frame::GetRange { from: 10, count: 128 },
            Frame::GetBlocks(vec![[7; 32]]),
            Frame::Block { header: header(), body: vec![1, 0, 0, 0] },
            Frame::GetTxs(vec![[8; 32]; 3]),
            Frame::Tx(vec![1, 0, 9]),
            Frame::GetCheckpoints { from_height: 120 },
            Frame::Checkpoint(Checkpoint { height: 120, block_id: [9; 32], sig: vec![1; 102] }),
            Frame::End,
            Frame::Error { code: 3, reason: "limit".into() },
            Frame::HeaderGossip(header()),
            Frame::Shadow(vec![0; 10]),
        ]
    }

    #[test]
    fn round_trip_through_tade_reference_decoder() {
        for frame in all() {
            let bytes = frame.encode();
            let chunks = tade::Reader::read_all(&bytes[..]).unwrap();
            assert_eq!(chunks.len(), 1, "{frame:?}");
            let decoded = Frame::from_parts(chunks[0].sigil, chunks[0].render, &chunks[0].payload).unwrap();
            assert_eq!(decoded, frame);
            assert_eq!(Frame::decode(&bytes, MAX_RESPONSE_BYTES).unwrap(), frame);
        }
    }

    #[test]
    fn status_is_138_bytes() {
        let frame = &all()[1];
        assert_eq!(frame.payload().len(), 138);
        assert_eq!(STATUS_BYTES, 138);
    }

    #[test]
    fn strictness() {
        let mut bytes = Frame::End.encode();
        bytes.push(0);
        assert_eq!(Frame::decode(&bytes, 10), Err(FrameError::Trailing));
        assert_eq!(Frame::decode(b"", 10), Err(FrameError::Marker));
        assert_eq!(Frame::decode(b"abc", 10), Err(FrameError::Marker));
        let big = Frame::Tx(vec![0; 100]).encode();
        assert_eq!(Frame::decode(&big, 50), Err(FrameError::Oversize(100)));
        assert!(matches!(Frame::decode(&big[..20], 200), Err(FrameError::Truncated)));
        assert!(matches!(Frame::decode(&[0x1f, b'#', b'z', 0], 10), Err(FrameError::UnknownType(..))));
        assert!(matches!(Frame::decode(&[0x1f, b'*', b'c', 0], 10), Err(FrameError::UnknownType(..))));
        let too_many = Frame::GetRange { from: 0, count: 129 }.encode();
        assert!(matches!(Frame::decode(&too_many, 100), Ok(Frame::GetRange { count: 129, .. })));
        let wrong_sigil = tade::Chunk::new(b'~', b'e', bytes::Bytes::new()).encode();
        assert!(matches!(Frame::decode(&wrong_sigil, 10), Err(FrameError::UnknownType(..))));
    }

    #[test]
    fn varint_bounds() {
        assert_eq!(varint(&[0]).unwrap(), (0, 1));
        assert_eq!(varint(&[0xac, 0x02]).unwrap(), (300, 2));
        assert_eq!(varint(&[0x80; 10]), Err(FrameError::Varint));
        assert_eq!(varint(&[0x80; 3]), Err(FrameError::Truncated));
        assert_eq!(varint(&[0xff; 9]).map(|_| ()), Err(FrameError::Truncated));
    }

    #[test]
    fn decoder_survives_random_and_mutated_bytes() {
        let mut x = 0x2545f4914f6cdd1du64;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let seeds: Vec<Vec<u8>> = all().iter().map(Frame::encode).collect();
        for i in 0..40_000 {
            let mut bytes = if i % 2 == 0 {
                (0..(next() % 400) as usize).map(|_| next() as u8).collect::<Vec<u8>>()
            } else {
                seeds[(next() as usize) % seeds.len()].clone()
            };
            if !bytes.is_empty() {
                for _ in 0..(next() % 4) {
                    let at = (next() as usize) % bytes.len();
                    bytes[at] = next() as u8;
                }
                if next() % 5 == 0 {
                    bytes.truncate((next() as usize) % bytes.len());
                }
            }
            let _ = Frame::decode(&bytes, MAX_RESPONSE_BYTES);
            let _ = Frame::decode(&bytes, MAX_GOSSIP_BYTES);
            let _ = split(&bytes, 64);
        }
    }
}
