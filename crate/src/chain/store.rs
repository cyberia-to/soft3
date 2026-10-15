//! Chain store on fjall (§3.6): headers are permanent and carry their
//! chainwork and status, bodies and undo journals are keyed by block id,
//! the main chain is a height → id table. An in-memory index mirrors the
//! headers partition (168 B + 33 B per header: ~200 MB per million blocks)
//! so fork choice never reads disk.

use std::collections::HashMap;
use std::path::Path;

use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle, PersistMode};

use super::header::{BlockId, Header};
use super::state::Undo;
use super::u256::U256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    /// passed steps 1–5, no body
    Header = 0,
    /// body stored and stateless-checked (step 6), not yet applied
    Body = 1,
    /// applied at some point (steps 7–8 passed); may or may not be on the main chain
    Valid = 2,
    /// failed steps 6–8, or descends from such a block
    Invalid = 3,
}

impl Status {
    fn from_u8(b: u8) -> Option<Status> {
        Some(match b {
            0 => Status::Header,
            1 => Status::Body,
            2 => Status::Valid,
            3 => Status::Invalid,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub header: Header,
    pub chainwork: U256,
    pub status: Status,
    /// insertion order: the first-seen rule of §5.1 orders ties by it
    pub seq: u64,
}

pub struct Store {
    ks: Keyspace,
    headers: PartitionHandle,
    bodies: PartitionHandle,
    undo: PartitionHandle,
    main: PartitionHandle,
    index: HashMap<BlockId, Entry>,
    chain: Vec<BlockId>,
    next_seq: u64,
}

fn err(e: fjall::Error) -> String {
    e.to_string()
}

fn entry_bytes(e: &Entry) -> Vec<u8> {
    let mut out = e.header.encode().to_vec();
    out.extend_from_slice(&e.chainwork.to_le_bytes());
    out.push(e.status as u8);
    out.extend_from_slice(&e.seq.to_le_bytes());
    out
}

fn entry_from(b: &[u8]) -> Option<Entry> {
    if b.len() != 168 + 32 + 1 + 8 {
        return None;
    }
    Some(Entry {
        header: Header::decode_raw(&b[..168]).ok()?,
        chainwork: U256::from_le_bytes(b[168..200].try_into().ok()?),
        status: Status::from_u8(b[200])?,
        seq: u64::from_le_bytes(b[201..209].try_into().ok()?),
    })
}

impl Store {
    pub fn open(path: &Path) -> Result<Store, String> {
        let ks = Config::new(path).open().map_err(err)?;
        let part = |name: &str| ks.open_partition(name, PartitionCreateOptions::default()).map_err(err);
        let (headers, bodies, undo, main) = (part("headers")?, part("bodies")?, part("undo")?, part("main")?);
        let mut index = HashMap::new();
        let mut next_seq = 0u64;
        for kv in headers.iter() {
            let (k, v) = kv.map_err(err)?;
            let id: BlockId = k.as_ref().try_into().map_err(|_| "corrupt header key".to_string())?;
            let e = entry_from(&v).ok_or_else(|| "corrupt header entry".to_string())?;
            next_seq = next_seq.max(e.seq + 1);
            index.insert(id, e);
        }
        let mut chain = Vec::new();
        for kv in main.iter() {
            let (k, v) = kv.map_err(err)?;
            let h = u64::from_be_bytes(k.as_ref().try_into().map_err(|_| "corrupt height key".to_string())?);
            if h != chain.len() as u64 {
                return Err(format!("main chain has a gap at {h}"));
            }
            chain.push(v.as_ref().try_into().map_err(|_| "corrupt main entry".to_string())?);
        }
        Ok(Store { ks, headers, bodies, undo, main, index, chain, next_seq })
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    pub fn get(&self, id: &BlockId) -> Option<&Entry> {
        self.index.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&BlockId, &Entry)> {
        self.index.iter()
    }

    pub fn contains(&self, id: &BlockId) -> bool {
        self.index.contains_key(id)
    }

    pub fn put_header(&mut self, header: Header, chainwork: U256, status: Status) -> Result<(), String> {
        let id = header.id();
        let e = Entry { header, chainwork, status, seq: self.next_seq };
        self.next_seq += 1;
        self.headers.insert(id, entry_bytes(&e)).map_err(err)?;
        self.index.insert(id, e);
        Ok(())
    }

    pub fn set_status(&mut self, id: &BlockId, status: Status) -> Result<(), String> {
        let Some(e) = self.index.get_mut(id) else {
            return Err("unknown block".into());
        };
        e.status = status;
        let bytes = entry_bytes(e);
        self.headers.insert(*id, bytes).map_err(err)
    }

    pub fn put_body(&mut self, id: &BlockId, body: &[u8]) -> Result<(), String> {
        self.bodies.insert(*id, body).map_err(err)
    }

    pub fn body(&self, id: &BlockId) -> Option<Vec<u8>> {
        self.bodies.get(id).ok().flatten().map(|v| v.to_vec())
    }

    pub fn has_body(&self, id: &BlockId) -> bool {
        self.bodies.contains_key(id).unwrap_or(false)
    }

    pub fn put_undo(&mut self, id: &BlockId, undo: &Undo) -> Result<(), String> {
        self.undo.insert(*id, undo.encode()).map_err(err)
    }

    pub fn undo(&self, id: &BlockId) -> Option<Undo> {
        self.undo.get(id).ok().flatten().and_then(|v| Undo::decode(&v))
    }

    /// Main-chain id at `height`.
    pub fn main_at(&self, height: u64) -> Option<BlockId> {
        self.chain.get(height as usize).copied()
    }

    pub fn tip(&self) -> Option<(u64, BlockId)> {
        self.chain.last().map(|id| (self.chain.len() as u64 - 1, *id))
    }

    pub fn tip_height(&self) -> u64 {
        self.chain.len().saturating_sub(1) as u64
    }

    /// Append to the main chain; `id`'s height must be the next one.
    pub fn push_main(&mut self, id: BlockId) -> Result<(), String> {
        let h = self.chain.len() as u64;
        match self.index.get(&id) {
            Some(e) if e.header.height == h => {}
            _ => return Err(format!("block is not at height {h}")),
        }
        self.main.insert(h.to_be_bytes(), id).map_err(err)?;
        self.chain.push(id);
        Ok(())
    }

    pub fn pop_main(&mut self) -> Result<Option<BlockId>, String> {
        let Some(id) = self.chain.pop() else {
            return Ok(None);
        };
        self.main.remove((self.chain.len() as u64).to_be_bytes()).map_err(err)?;
        Ok(Some(id))
    }

    /// Walk `prev` links from `id` down to `height`.
    pub fn ancestor_at(&self, id: &BlockId, height: u64) -> Option<BlockId> {
        let mut cur = *id;
        loop {
            let e = self.index.get(&cur)?;
            if e.header.height == height {
                return Some(cur);
            }
            if e.header.height < height {
                return None;
            }
            // fast path: on the main chain the table answers directly
            if self.chain.get(e.header.height as usize) == Some(&cur) {
                return self.main_at(height);
            }
            cur = e.header.prev;
        }
    }

    /// Is `id` on the main chain?
    pub fn on_main(&self, id: &BlockId) -> bool {
        self.index.get(id).is_some_and(|e| self.chain.get(e.header.height as usize) == Some(id))
    }

    /// The main-chain ids from `from` to the tip, at most `count`.
    pub fn main_range(&self, from: u64, count: usize) -> Vec<BlockId> {
        self.chain.iter().skip(from as usize).take(count).copied().collect()
    }

    /// Durable write of everything so far.
    pub fn persist(&self) -> Result<(), String> {
        self.ks.persist(PersistMode::SyncAll).map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(label: &str) -> std::path::PathBuf {
        let mut d = std::env::temp_dir();
        d.push(format!("soft3-chain-store-{label}-{}-{}", std::process::id(), std::time::SystemTime::now().elapsed().map(|_| 0).unwrap_or(1)));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn header(height: u64, prev: BlockId, time: u64) -> Header {
        Header { version: 1, height, time, bits: 0x1e01_0000, nonce: [0; 16], prev, tx_root: [1; 32], state_root: [2; 32], chain: [3; 32] }
    }

    #[test]
    fn headers_bodies_undo_and_main_chain_survive_reopen() {
        let dir = temp("reopen");
        let (h0, h1) = (header(0, [0; 32], 0), header(1, [0; 32], 30));
        let h1 = Header { prev: h0.id(), ..h1 };
        {
            let mut s = Store::open(&dir).unwrap();
            assert!(s.is_empty());
            s.put_header(h0, U256::from_u64(1), Status::Valid).unwrap();
            s.push_main(h0.id()).unwrap();
            s.put_header(h1, U256::from_u64(3), Status::Body).unwrap();
            s.put_body(&h1.id(), b"body").unwrap();
            s.put_undo(&h1.id(), &Undo(vec![super::super::state::UndoEntry::Height { before: 0 }])).unwrap();
            assert!(s.push_main(h1.id()).is_ok());
            assert_eq!(s.push_main(h1.id()).ok(), None, "height 1 twice");
            s.set_status(&h1.id(), Status::Valid).unwrap();
            s.persist().unwrap();
        }
        let mut s = Store::open(&dir).unwrap();
        assert_eq!(s.tip(), Some((1, h1.id())));
        assert_eq!(s.get(&h1.id()).unwrap().status, Status::Valid);
        assert_eq!(s.get(&h1.id()).unwrap().chainwork, U256::from_u64(3));
        assert_eq!(s.body(&h1.id()).as_deref(), Some(&b"body"[..]));
        assert!(s.undo(&h1.id()).is_some());
        assert_eq!(s.ancestor_at(&h1.id(), 0), Some(h0.id()));
        assert_eq!(s.ancestor_at(&h1.id(), 2), None);
        assert!(s.on_main(&h1.id()));
        assert_eq!(s.main_range(0, 10), vec![h0.id(), h1.id()]);
        assert_eq!(s.pop_main().unwrap(), Some(h1.id()));
        assert!(!s.on_main(&h1.id()));
        assert_eq!(s.tip_height(), 0);
        std::fs::remove_dir_all(&dir).ok();
    }
}
