//! A minimal chain behind [`ChainView`] so the wire runs end to end before
//! the kernel lands: real header syntax, PoW, MTP, heaviest chain with the
//! reorg bound, real transaction rules (signatures, nonces, balances, fees,
//! coinbase exactness and maturity, claims) over an account map — but a
//! constant target instead of ASERT and a hemera digest of the account map
//! instead of the bbg root. It is the local harness's chain, never a launch
//! chain; `--chain pussy|bostrom` refuse it. Blocks persist to
//! `$home/<chain>/blocks.tade` and replay on open.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use tokio::sync::broadcast;

use super::chain_view::{BlockError, BlockVerdict, ChainEvent, ChainView, HeaderError, HeaderVerdict, Tip};
use super::codec::{self, decode_bits, decode_body, hash, hash_tagged, token, tx_root, txid, Header, Id, Tx, U256, COINBASE_MATURITY, MAX_FUTURE_DRIFT, MAX_REORG, MIN_FEE_PER_BYTE, MTP_WINDOW, POW_LIMIT_BITS, RULE_SET};
use super::frames::{Frame, SIGIL};

pub const GENESIS_DIALECT: &str = "urn:cyberia:chain-genesis:1";
/// Target 2^252: sixteen expected hashes per block, the harness's cadence.
pub const DEV_BITS: u32 = 0x2010_0000;

// ── genesis file (chain.md §4.1) ────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub name: String,
    pub hrp: String,
    pub denom: String,
    pub time: u64,
    pub bits: u32,
    pub subsidy: u64,
    pub checkpoint_key: [u8; 33],
    pub checkpoint_sunset: u64,
    pub balances: Vec<(Id, Id, u64)>,
    pub links: Vec<(Id, Id, Id, u64)>,
}

fn string(out: &mut Vec<u8>, s: &str) {
    out.push(s.len().min(255) as u8);
    out.extend_from_slice(&s.as_bytes()[..s.len().min(255)]);
}

fn read_string(b: &[u8], at: &mut usize) -> Result<String, String> {
    let len = *b.get(*at).ok_or("params truncated")? as usize;
    let s = b.get(*at + 1..*at + 1 + len).ok_or("params truncated")?;
    *at += 1 + len;
    String::from_utf8(s.to_vec()).map_err(|_| "params utf-8".to_string())
}

fn read_u64(b: &[u8], at: &mut usize) -> Result<u64, String> {
    let s = b.get(*at..*at + 8).ok_or("params truncated")?;
    *at += 8;
    Ok(u64::from_le_bytes(s.try_into().expect("8")))
}

impl Genesis {
    pub fn encode(&self) -> Vec<u8> {
        // chain.md §4.1: `(~, p)` params, `($, b)` balance, `(#, l)` link
        let frame = |sigil: u8, render: u8, payload: Vec<u8>| tade::Chunk::new(sigil, render, bytes::Bytes::from(payload)).encode();
        let mut out = Frame::Dialect(GENESIS_DIALECT.into()).encode();
        let mut p = 1u32.to_le_bytes().to_vec();
        string(&mut p, &self.name);
        string(&mut p, &self.hrp);
        string(&mut p, &self.denom);
        p.extend_from_slice(&self.time.to_le_bytes());
        p.extend_from_slice(&self.bits.to_le_bytes());
        p.extend_from_slice(&self.subsidy.to_le_bytes());
        p.extend_from_slice(&self.checkpoint_key);
        p.extend_from_slice(&self.checkpoint_sunset.to_le_bytes());
        out.extend(frame(tade::sigil::SIG, b'p', p));
        for (owner, tok, amount) in &self.balances {
            let mut b = owner.to_vec();
            b.extend_from_slice(tok);
            b.extend_from_slice(&amount.to_le_bytes());
            out.extend(frame(tade::sigil::BUC, b'b', b));
        }
        for (neuron, p, q, height) in &self.links {
            let mut l = neuron.to_vec();
            l.extend_from_slice(p);
            l.extend_from_slice(q);
            l.extend_from_slice(&height.to_le_bytes());
            out.extend(frame(SIGIL, b'l', l));
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Genesis, String> {
        let chunks = tade::Reader::read_all(bytes).map_err(|e| e.to_string())?;
        let mut it = chunks.iter();
        match it.next() {
            Some(c) if c.sigil == b'*' && c.render == b'k' && c.payload.as_ref() == GENESIS_DIALECT.as_bytes() => {}
            _ => return Err("genesis: first frame is not the chain-genesis dialect".into()),
        }
        let p = it.next().filter(|c| c.sigil == tade::sigil::SIG && c.render == b'p').ok_or("genesis: params frame missing")?;
        let b = &p.payload;
        let mut at = 4;
        if b.len() < 4 || u32::from_le_bytes(b[0..4].try_into().expect("4")) != 1 {
            return Err("genesis: params version".into());
        }
        let name = read_string(b, &mut at)?;
        let hrp = read_string(b, &mut at)?;
        let denom = read_string(b, &mut at)?;
        let time = read_u64(b, &mut at)?;
        let bits = u32::from_le_bytes(b.get(at..at + 4).ok_or("params truncated")?.try_into().expect("4"));
        at += 4;
        let subsidy = read_u64(b, &mut at)?;
        let checkpoint_key: [u8; 33] = b.get(at..at + 33).ok_or("params truncated")?.try_into().expect("33");
        at += 33;
        let checkpoint_sunset = read_u64(b, &mut at)?;
        if at != b.len() {
            return Err("genesis: params trailing bytes".into());
        }
        let mut genesis = Genesis { name, hrp, denom, time, bits, subsidy, checkpoint_key, checkpoint_sunset, balances: vec![], links: vec![] };
        let mut seen_link = false;
        for c in it {
            match (c.sigil, c.render) {
                (tade::sigil::BUC, b'b') if !seen_link && c.payload.len() == 72 => {
                    let entry = (c.payload[0..32].try_into().expect("32"), c.payload[32..64].try_into().expect("32"), u64::from_le_bytes(c.payload[64..72].try_into().expect("8")));
                    if entry.2 == 0 || genesis.balances.last().is_some_and(|last| (last.0, last.1) >= (entry.0, entry.1)) {
                        return Err("genesis: balances not canonical".into());
                    }
                    genesis.balances.push(entry);
                }
                (SIGIL, b'l') if c.payload.len() == 104 => {
                    seen_link = true;
                    genesis.links.push((
                        c.payload[0..32].try_into().expect("32"), c.payload[32..64].try_into().expect("32"),
                        c.payload[64..96].try_into().expect("32"), u64::from_le_bytes(c.payload[96..104].try_into().expect("8")),
                    ));
                }
                _ => return Err("genesis: malformed frame".into()),
            }
        }
        if genesis.name.is_empty() || genesis.denom.is_empty() || decode_bits(genesis.bits).is_none() {
            return Err("genesis: params out of range".into());
        }
        Ok(genesis)
    }
}

/// A dev genesis: `name`, funded owners, the harness target.
pub fn dev_genesis(name: &str, time: u64, funded: &[(Id, u64)]) -> Genesis {
    let native = token(name);
    let mut balances: Vec<(Id, Id, u64)> = funded.iter().map(|(o, a)| (*o, native, *a)).collect();
    balances.sort();
    balances.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    let total: u64 = balances.iter().map(|b| b.2).sum();
    Genesis {
        name: name.into(), hrp: name.into(), denom: name.into(), time, bits: DEV_BITS,
        subsidy: (total / 105_120_000).max(1_000), checkpoint_key: [0; 33], checkpoint_sunset: 525_600, balances, links: vec![],
    }
}

pub fn legacy_owner(hrp: &str, account_id: &[u8; 20]) -> Id {
    hash_tagged(b"soft3/chain/legacy/v1\0", &[hrp.as_bytes(), account_id])
}

// ── state ───────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct State {
    balances: BTreeMap<(Id, Id), u64>,
    nonces: BTreeMap<Id, u64>,
    coins: BTreeMap<Id, u64>,
    /// height → coinbase credits maturing at `height + COINBASE_MATURITY`.
    immature: BTreeMap<u64, Vec<(Id, u64)>>,
    particles: BTreeSet<Id>,
    axons: BTreeSet<(Id, Id)>,
}

impl State {
    fn root(&self) -> Id {
        let mut bytes = b"soft3/chain/devroot/v1\0".to_vec();
        for ((o, t), a) in &self.balances {
            bytes.extend_from_slice(o);
            bytes.extend_from_slice(t);
            bytes.extend_from_slice(&a.to_le_bytes());
        }
        for (n, v) in &self.nonces {
            bytes.extend_from_slice(n);
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        for (t, s) in &self.coins {
            bytes.extend_from_slice(t);
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        for (h, credits) in &self.immature {
            bytes.extend_from_slice(&h.to_le_bytes());
            for (o, a) in credits {
                bytes.extend_from_slice(o);
                bytes.extend_from_slice(&a.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(self.particles.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.axons.len() as u64).to_le_bytes());
        hash(&bytes)
    }

    fn immature_of(&self, owner: &Id, height: u64) -> u64 {
        self.immature.iter().filter(|(h, _)| **h + COINBASE_MATURITY > height)
            .flat_map(|(_, c)| c.iter()).filter(|(o, _)| o == owner).map(|(_, a)| *a).sum()
    }

    fn spendable(&self, owner: &Id, tok: &Id, native: &Id, height: u64) -> u64 {
        let bal = self.balances.get(&(*owner, *tok)).copied().unwrap_or(0);
        if tok == native { bal.saturating_sub(self.immature_of(owner, height)) } else { bal }
    }

    fn credit(&mut self, owner: Id, tok: Id, amount: u64) {
        *self.balances.entry((owner, tok)).or_insert(0) += amount;
    }

    fn debit(&mut self, owner: &Id, tok: &Id, native: &Id, amount: u64, height: u64) -> Result<(), BlockError> {
        if self.spendable(owner, tok, native, height) < amount {
            return Err(BlockError::Tx("insufficient balance".into()));
        }
        let entry = self.balances.get_mut(&(*owner, *tok)).ok_or_else(|| BlockError::Tx("insufficient balance".into()))?;
        *entry -= amount;
        if *entry == 0 {
            self.balances.remove(&(*owner, *tok));
        }
        Ok(())
    }

    /// Step 7 for one signed transaction (kinds 1–3) at `height`.
    fn apply_tx(&mut self, bytes: &[u8], tx: &Tx, height: u64, chain: &Id, native: &Id, hrp: &str) -> Result<(), BlockError> {
        if matches!(tx, Tx::Coinbase { .. }) {
            return Err(BlockError::Body("second coinbase".into()));
        }
        if tx.fee() < bytes.len() as u64 * MIN_FEE_PER_BYTE {
            return Err(BlockError::Tx("fee below floor".into()));
        }
        if !tx.verify(chain) {
            return Err(BlockError::Tx("bad signature".into()));
        }
        let from = tx.subject().expect("signed subject");
        if let Tx::Claim { account_id, .. } = tx {
            let pubkey: [u8; 33] = tx.sig()[5..38].try_into().expect("33");
            if mudra::cosmos::account_id(&pubkey) != *account_id {
                return Err(BlockError::Tx("claim account".into()));
            }
        }
        if tx.nonce() != Some(self.nonces.get(&from).copied().unwrap_or(0)) {
            return Err(BlockError::Tx("nonce".into()));
        }
        match tx {
            Tx::Transfer { token: tok, amount, to, fee, .. } => {
                let native_debit = if tok == native { fee.saturating_add(*amount) } else { *fee };
                self.debit(&from, native, native, native_debit, height)?;
                if tok != native {
                    self.debit(&from, tok, native, *amount, height)?;
                }
                self.credit(*to, *tok, *amount);
            }
            Tx::Cyberlink { token: tok, amount, p, q, fee, .. } => {
                if p == q {
                    return Err(BlockError::Tx("p == q".into()));
                }
                let native_debit = if tok == native { fee.saturating_add(*amount) } else { *fee };
                self.debit(&from, native, native, native_debit, height)?;
                if tok != native {
                    self.debit(&from, tok, native, *amount, height)?;
                }
                if *amount > 0 {
                    self.credit(hash_tagged(b"", &[p, q]), *tok, *amount);
                }
                self.particles.insert(*p);
                self.particles.insert(*q);
                self.axons.insert((*p, *q));
            }
            Tx::Claim { account_id, fee, .. } => {
                let dormant = legacy_owner(hrp, account_id);
                self.debit(&dormant, native, native, *fee, height)?;
                let moved: Vec<((Id, Id), u64)> = self.balances.range((dormant, [0u8; 32])..=(dormant, [0xff; 32])).map(|(k, v)| (*k, *v)).collect();
                for ((_, tok), amount) in moved {
                    self.balances.remove(&(dormant, tok));
                    self.credit(from, tok, amount);
                }
            }
            Tx::Coinbase { .. } => unreachable!("filtered above"),
        }
        self.balances.retain(|_, v| *v > 0);
        *self.nonces.entry(from).or_insert(0) += 1;
        Ok(())
    }

    /// Steps 7–8 of chain.md §5.5 for one block's transactions, coinbase first.
    fn apply(&mut self, height: u64, txs: &[Vec<u8>], chain: &Id, native: &Id, subsidy: u64, hrp: &str) -> Result<(), BlockError> {
        let parsed: Vec<Tx> = txs.iter().map(|t| Tx::decode(t).map_err(|e| BlockError::Tx(e.to_string()))).collect::<Result<_, _>>()?;
        let fees: u64 = parsed.iter().map(Tx::fee).fold(0u64, |a, b| a.saturating_add(b));
        let Some(Tx::Coinbase { height: cb_height, to, amount, .. }) = parsed.first() else {
            return Err(BlockError::Body("coinbase first".into()));
        };
        if *cb_height != height || *amount != subsidy.saturating_add(fees) {
            return Err(BlockError::Tx("coinbase amount or height".into()));
        }
        if parsed[1..].iter().any(|t| matches!(t, Tx::Coinbase { .. })) {
            return Err(BlockError::Body("second coinbase".into()));
        }
        self.credit(*to, *native, *amount);
        *self.coins.entry(*native).or_insert(0) += subsidy;
        self.immature.entry(height).or_default().push((*to, *amount));
        for (bytes, tx) in txs[1..].iter().zip(&parsed[1..]) {
            self.apply_tx(bytes, tx, height, chain, native, hrp)?;
        }
        self.immature.retain(|h, _| *h + COINBASE_MATURITY > height);
        Ok(())
    }
}

// ── the chain ───────────────────────────────────────────────────────────────

struct Entry {
    header: Header,
    chainwork: U256,
    invalid: bool,
    has_body: bool,
}

struct Inner {
    genesis: Genesis,
    chain_id: Id,
    genesis_id: Id,
    native: Id,
    pow_limit: U256,
    headers: HashMap<Id, Entry>,
    bodies: HashMap<Id, Vec<u8>>,
    main: Vec<Id>,
    state: State,
    tip: Tip,
    best: Tip,
    txindex: HashMap<Id, (u64, Id)>,
    log: Option<File>,
}

pub struct DevChain {
    inner: Mutex<Inner>,
    events: broadcast::Sender<ChainEvent>,
}

impl DevChain {
    /// Open from a genesis file; replays `blocks.tade` under `home` when present.
    pub fn open(genesis_bytes: &[u8], home: Option<&Path>) -> Result<DevChain, String> {
        let genesis = Genesis::decode(genesis_bytes)?;
        let chain_id = hash(genesis_bytes);
        let native = token(&genesis.denom);
        let mut state = State::default();
        for (owner, tok, amount) in &genesis.balances {
            state.credit(*owner, *tok, *amount);
            *state.coins.entry(*tok).or_insert(0) += amount;
            state.nonces.entry(*owner).or_insert(0);
        }
        for (neuron, p, q, _) in &genesis.links {
            state.particles.insert(*p);
            state.particles.insert(*q);
            state.axons.insert((*p, *q));
            state.nonces.entry(*neuron).or_insert(0);
        }
        let header = Header {
            version: RULE_SET as u32, height: 0, time: genesis.time, bits: genesis.bits, nonce: [0; 16],
            prev: [0; 32], tx_root: chain_id, state_root: state.root(), chain: chain_id,
        };
        let genesis_id = header.id();
        let tip = Tip { height: 0, id: genesis_id, chainwork: U256::ZERO };
        let genesis_target = decode_bits(genesis.bits).ok_or("genesis bits")?;
        let pow_limit = genesis_target.max(decode_bits(POW_LIMIT_BITS).expect("limit"));
        let mut headers = HashMap::new();
        headers.insert(genesis_id, Entry { header, chainwork: U256::ZERO, invalid: false, has_body: true });
        let mut bodies = HashMap::new();
        bodies.insert(genesis_id, Vec::new());
        let chain = DevChain {
            inner: Mutex::new(Inner { genesis, chain_id, genesis_id, native, pow_limit, headers, bodies, main: vec![genesis_id], state, tip, best: tip, txindex: HashMap::new(), log: None }),
            events: broadcast::channel(256).0,
        };
        if let Some(home) = home {
            std::fs::create_dir_all(home).map_err(|e| e.to_string())?;
            let path = home.join("blocks.tade");
            if let Ok(bytes) = std::fs::read(&path) {
                chain.replay(&bytes);
            }
            let file = OpenOptions::new().create(true).append(true).open(&path).map_err(|e| e.to_string())?;
            chain.inner.lock().expect("chain").log = Some(file);
        }
        Ok(chain)
    }

    fn replay(&self, bytes: &[u8]) {
        let mut at = 0;
        while at < bytes.len() {
            let Ok((sigil, render, payload, used)) = super::frames::split(&bytes[at..], super::frames::MAX_RESPONSE_BYTES) else { break };
            at += used;
            match Frame::from_parts(sigil, render, payload) {
                Ok(Frame::HeaderGossip(h)) => {
                    let _ = self.accept_header(&h, h.time.saturating_add(MAX_FUTURE_DRIFT));
                }
                Ok(Frame::Block { header, body }) => {
                    let _ = self.accept_header(&header, header.time.saturating_add(MAX_FUTURE_DRIFT));
                    let _ = self.accept_block(&header, &body);
                }
                _ => {}
            }
        }
    }

    fn log(inner: &mut Inner, frame: &Frame) {
        if let Some(file) = inner.log.as_mut() {
            let _ = file.write_all(&frame.encode());
        }
    }

    pub fn genesis(&self) -> Genesis {
        self.inner.lock().expect("chain").genesis.clone()
    }

}

fn mtp(inner: &Inner, mut id: Id) -> u64 {
    let mut times = Vec::with_capacity(MTP_WINDOW);
    while times.len() < MTP_WINDOW {
        let Some(e) = inner.headers.get(&id) else { break };
        times.push(e.header.time);
        if e.header.height == 0 {
            break;
        }
        id = e.header.prev;
    }
    times.sort_unstable();
    times[(times.len() - 1) / 2]
}

/// The path from `id` back to the first main-chain ancestor: `(fork height, ids newest-first)`.
fn path_to_main(inner: &Inner, mut id: Id) -> Option<(u64, Vec<Id>)> {
    let mut path = Vec::new();
    loop {
        let e = inner.headers.get(&id)?;
        let h = e.header.height as usize;
        if inner.main.get(h) == Some(&id) {
            return Some((e.header.height, path));
        }
        path.push(id);
        id = e.header.prev;
    }
}

impl Inner {
    /// Rebuild the state along `main[..=fork]` then `path` (newest-first), checking roots.
    fn rebuild(&self, fork: u64, path: &[Id]) -> Result<State, (Id, BlockError)> {
        let mut state = State::default();
        let genesis_entry = &self.headers[&self.genesis_id];
        for (owner, tok, amount) in &self.genesis.balances {
            state.credit(*owner, *tok, *amount);
            *state.coins.entry(*tok).or_insert(0) += amount;
            state.nonces.entry(*owner).or_insert(0);
        }
        for (neuron, p, q, _) in &self.genesis.links {
            state.particles.insert(*p);
            state.particles.insert(*q);
            state.axons.insert((*p, *q));
            state.nonces.entry(*neuron).or_insert(0);
        }
        debug_assert_eq!(state.root(), genesis_entry.header.state_root);
        let ids: Vec<Id> = self.main[1..=fork as usize].iter().copied().chain(path.iter().rev().copied()).collect();
        for id in ids {
            let e = &self.headers[&id];
            let body = self.bodies.get(&id).ok_or((id, BlockError::MissingParent))?;
            let txs = decode_body(body).map_err(|err| (id, BlockError::Body(err.to_string())))?;
            state.apply(e.header.height, &txs, &self.chain_id, &self.native, self.genesis.subsidy, &self.genesis.hrp).map_err(|err| (id, err))?;
            if state.root() != e.header.state_root {
                return Err((id, BlockError::StateRoot));
            }
        }
        Ok(state)
    }
}

impl DevChain {
    /// Make `header` (with its body held) the tip: incremental apply when it
    /// extends the tip, else a rebuild along `path` from `fork`; marks the
    /// header invalid on a body/state failure, never on a missing parent.
    fn adopt(&self, inner: &mut Inner, header: &Header, fork: u64, path: Vec<Id>) -> Result<BlockVerdict, BlockError> {
        let id = header.id();
        let chainwork = inner.headers[&id].chainwork;
        if inner.tip.height.saturating_sub(fork) > MAX_REORG {
            return Ok(BlockVerdict::DeepFork);
        }
        let new_state = if header.prev == inner.tip.id {
            let body = inner.bodies[&id].clone();
            let txs = decode_body(&body).map_err(|e| BlockError::Body(e.to_string()))?;
            let mut state = inner.state.clone();
            let result = state.apply(header.height, &txs, &inner.chain_id, &inner.native, inner.genesis.subsidy, &inner.genesis.hrp)
                .and_then(|()| if state.root() == header.state_root { Ok(()) } else { Err(BlockError::StateRoot) });
            match result {
                Ok(()) => state,
                Err(e) => {
                    inner.headers.get_mut(&id).expect("entry").invalid = true;
                    return Err(e);
                }
            }
        } else {
            match inner.rebuild(fork, &path) {
                Ok(state) => state,
                Err((_, BlockError::MissingParent)) => return Err(BlockError::MissingParent),
                Err((bad, e)) => {
                    if bad == id || path.contains(&bad) {
                        inner.headers.get_mut(&bad).expect("entry").invalid = true;
                        if bad != id {
                            inner.headers.get_mut(&id).expect("entry").invalid = true;
                        }
                    }
                    return Err(e);
                }
            }
        };
        let reorged_out: Vec<Vec<u8>> = inner.main[fork as usize + 1..].iter().filter_map(|old| inner.bodies.get(old).cloned()).collect();
        let chain_id = inner.chain_id;
        for body in &reorged_out {
            for t in decode_body(body).unwrap_or_default() {
                inner.txindex.remove(&txid(&chain_id, &t));
            }
        }
        inner.main.truncate(fork as usize + 1);
        for new in path.iter().rev() {
            inner.main.push(*new);
            let h = inner.headers[new].header.height;
            let body = inner.bodies[new].clone();
            for t in decode_body(&body).unwrap_or_default() {
                inner.txindex.insert(txid(&chain_id, &t), (h, *new));
            }
        }
        inner.state = new_state;
        inner.tip = Tip { height: header.height, id, chainwork };
        if inner.best.chainwork < chainwork {
            inner.best = inner.tip;
        }
        let _ = self.events.send(ChainEvent::NewTip { height: header.height, id, reorged_out });
        Ok(BlockVerdict::NewTip)
    }
}

impl ChainView for DevChain {
    fn chain_id(&self) -> Id { self.inner.lock().expect("chain").chain_id }
    fn genesis_id(&self) -> Id { self.inner.lock().expect("chain").genesis_id }
    fn chain_name(&self) -> String { self.inner.lock().expect("chain").genesis.name.clone() }
    fn hrp(&self) -> String { self.inner.lock().expect("chain").genesis.hrp.clone() }
    fn native_token(&self) -> Id { self.inner.lock().expect("chain").native }
    fn subsidy(&self) -> u64 { self.inner.lock().expect("chain").genesis.subsidy }
    fn genesis_time(&self) -> u64 { self.inner.lock().expect("chain").genesis.time }
    fn checkpoint_key(&self) -> Option<[u8; 33]> {
        let key = self.inner.lock().expect("chain").genesis.checkpoint_key;
        (key != [0; 33]).then_some(key)
    }
    fn tip(&self) -> Tip { self.inner.lock().expect("chain").tip }
    fn best_header(&self) -> Tip { self.inner.lock().expect("chain").best }
    fn header(&self, id: &Id) -> Option<Header> { self.inner.lock().expect("chain").headers.get(id).map(|e| e.header) }
    fn header_at(&self, height: u64) -> Option<Header> {
        let inner = self.inner.lock().expect("chain");
        inner.main.get(height as usize).and_then(|id| inner.headers.get(id)).map(|e| e.header)
    }
    fn chainwork_of(&self, id: &Id) -> Option<U256> { self.inner.lock().expect("chain").headers.get(id).map(|e| e.chainwork) }
    fn has_body(&self, id: &Id) -> bool { self.inner.lock().expect("chain").bodies.contains_key(id) }
    fn block(&self, id: &Id) -> Option<(Header, Vec<u8>)> {
        let inner = self.inner.lock().expect("chain");
        Some((inner.headers.get(id)?.header, inner.bodies.get(id)?.clone()))
    }
    fn block_at(&self, height: u64) -> Option<(Header, Vec<u8>)> {
        let id = *self.inner.lock().expect("chain").main.get(height as usize)?;
        self.block(&id)
    }
    fn height_of(&self, id: &Id) -> Option<u64> {
        let inner = self.inner.lock().expect("chain");
        let h = inner.headers.get(id)?.header.height;
        (inner.main.get(h as usize) == Some(id)).then_some(h)
    }

    fn locator(&self) -> Vec<Id> {
        let inner = self.inner.lock().expect("chain");
        locator_heights(inner.tip.height).into_iter().map(|h| inner.main[h as usize]).collect()
    }

    fn headers_from_locator(&self, locator: &[Id], stop: &Id, max: usize) -> Vec<Header> {
        let inner = self.inner.lock().expect("chain");
        let mut start = None;
        for id in locator {
            if let Some(e) = inner.headers.get(id) {
                if inner.main.get(e.header.height as usize) == Some(id) {
                    start = Some(e.header.height + 1);
                    break;
                }
            }
        }
        let Some(start) = start else { return Vec::new() };
        let mut out = Vec::new();
        for id in inner.main.iter().skip(start as usize).take(max) {
            out.push(inner.headers[id].header);
            if id == stop {
                break;
            }
        }
        out
    }

    fn accept_header(&self, header: &Header, now: u64) -> Result<HeaderVerdict, HeaderError> {
        let mut inner = self.inner.lock().expect("chain");
        if header.rule_set() != RULE_SET {
            return Err(HeaderError::UnknownRuleSet);
        }
        if header.chain != inner.chain_id {
            return Err(HeaderError::WrongChain);
        }
        let target = decode_bits(header.bits).filter(|t| *t <= inner.pow_limit).ok_or(HeaderError::BadBits)?;
        let id = header.id();
        if inner.headers.contains_key(&id) {
            return Ok(HeaderVerdict::Known);
        }
        if header.height == 0 {
            return Err(HeaderError::Syntax("second genesis".into()));
        }
        let Some(parent) = inner.headers.get(&header.prev) else { return Ok(HeaderVerdict::Orphan) };
        if parent.invalid {
            return Err(HeaderError::InvalidParent);
        }
        if header.height != parent.header.height + 1 {
            return Err(HeaderError::BadParent);
        }
        if header.time <= mtp(&inner, header.prev) {
            return Err(HeaderError::BadTime);
        }
        if header.time > now.saturating_add(MAX_FUTURE_DRIFT) {
            return Ok(HeaderVerdict::Held);
        }
        if header.bits != inner.genesis.bits {
            return Err(HeaderError::BadTarget);
        }
        if U256::from_le_bytes(&id) >= target {
            return Err(HeaderError::BadPow);
        }
        let chainwork = parent.chainwork.saturating_add(codec::work(target));
        inner.headers.insert(id, Entry { header: *header, chainwork, invalid: false, has_body: false });
        if chainwork > inner.best.chainwork {
            inner.best = Tip { height: header.height, id, chainwork };
        }
        Self::log(&mut inner, &Frame::HeaderGossip(*header));
        Ok(HeaderVerdict::Accepted)
    }

    fn accept_block(&self, header: &Header, body: &[u8]) -> Result<BlockVerdict, BlockError> {
        let mut inner = self.inner.lock().expect("chain");
        let id = header.id();
        let entry = inner.headers.get(&id).ok_or(BlockError::UnknownHeader)?;
        if entry.has_body {
            return Ok(BlockVerdict::Known);
        }
        if entry.invalid {
            return Err(BlockError::InvalidParent);
        }
        let chainwork = entry.chainwork;
        let txs = decode_body(body).map_err(|e| BlockError::Body(e.to_string()))?;
        if tx_root(&txs) != header.tx_root {
            return Err(BlockError::Body("tx_root".into()));
        }
        if txs[0].get(1) != Some(&0) || txs[1..].iter().any(|t| t.get(1) == Some(&0)) {
            return Err(BlockError::Body("coinbase position".into()));
        }
        let mut seen = BTreeSet::new();
        if !txs.iter().all(|t| seen.insert(txid(&inner.chain_id, t))) {
            return Err(BlockError::Body("duplicate txid".into()));
        }
        inner.bodies.insert(id, body.to_vec());
        inner.headers.get_mut(&id).expect("entry").has_body = true;
        Self::log(&mut inner, &Frame::Block { header: *header, body: body.to_vec() });
        if chainwork <= inner.tip.chainwork {
            // a side-chain body: maybe the best header chain is now complete
            let best = inner.best;
            if best.chainwork > inner.tip.chainwork && best.id != id {
                if let Some((fork, path)) = path_to_main(&inner, best.id) {
                    if path.iter().all(|p| inner.bodies.contains_key(p)) {
                        let best_header = inner.headers[&best.id].header;
                        return match self.adopt(&mut inner, &best_header, fork, path) {
                            Ok(v) => Ok(v),
                            Err(_) => Ok(BlockVerdict::Stored),
                        };
                    }
                }
            }
            return Ok(BlockVerdict::Stored);
        }
        let (fork, path) = path_to_main(&inner, id).ok_or(BlockError::UnknownHeader)?;
        self.adopt(&mut inner, header, fork, path)
    }

    fn missing_bodies(&self, max: usize) -> Vec<(u64, Id)> {
        let inner = self.inner.lock().expect("chain");
        if inner.best.chainwork <= inner.tip.chainwork {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut id = inner.best.id;
        loop {
            let Some(e) = inner.headers.get(&id) else { break };
            if e.has_body || inner.main.get(e.header.height as usize) == Some(&id) {
                break;
            }
            out.push((e.header.height, id));
            id = e.header.prev;
        }
        out.reverse();
        out.truncate(max);
        out
    }

    fn nonce(&self, neuron: &Id) -> u64 { self.inner.lock().expect("chain").state.nonces.get(neuron).copied().unwrap_or(0) }
    fn balance(&self, owner: &Id, tok: &Id) -> u64 { self.inner.lock().expect("chain").state.balances.get(&(*owner, *tok)).copied().unwrap_or(0) }
    fn spendable(&self, owner: &Id, tok: &Id) -> u64 {
        let inner = self.inner.lock().expect("chain");
        inner.state.spendable(owner, tok, &inner.native, inner.tip.height)
    }
    fn balances(&self, owner: &Id) -> Vec<(Id, u64)> {
        self.inner.lock().expect("chain").state.balances.range((*owner, [0u8; 32])..=(*owner, [0xff; 32])).map(|((_, t), a)| (*t, *a)).collect()
    }
    fn tx_location(&self, id: &Id) -> Option<(u64, Id)> { self.inner.lock().expect("chain").txindex.get(id).copied() }
    fn supply(&self, tok: &Id) -> u64 { self.inner.lock().expect("chain").state.coins.get(tok).copied().unwrap_or(0) }
    fn conserved(&self) -> bool {
        let inner = self.inner.lock().expect("chain");
        let genesis_native: u64 = inner.genesis.balances.iter().filter(|b| b.1 == inner.native).map(|b| b.2).sum();
        let held: u64 = inner.state.balances.iter().filter(|((_, t), _)| *t == inner.native).map(|(_, a)| *a).sum();
        held == genesis_native + inner.tip.height * inner.genesis.subsidy
    }
    fn counts(&self) -> (u64, u64, u64) {
        let inner = self.inner.lock().expect("chain");
        (inner.state.nonces.len() as u64, inner.state.particles.len() as u64, inner.state.axons.len() as u64)
    }
    fn state_root(&self) -> Id { self.inner.lock().expect("chain").state.root() }

    fn template(&self, to: &Id, txs: &[Vec<u8>], now: u64) -> Result<(Header, Vec<u8>), String> {
        let inner = self.inner.lock().expect("chain");
        let height = inner.tip.height + 1;
        let mut running = inner.state.clone();
        let mut chosen: Vec<Vec<u8>> = Vec::new();
        let mut fees = 0u64;
        let mut seen = BTreeSet::new();
        for bytes in txs {
            let Ok(tx) = Tx::decode(bytes) else { continue };
            if !seen.insert(txid(&inner.chain_id, bytes)) {
                continue;
            }
            let mut trial = running.clone();
            if trial.apply_tx(bytes, &tx, height, &inner.chain_id, &inner.native, &inner.genesis.hrp).is_ok() {
                running = trial;
                fees = fees.saturating_add(tx.fee());
                chosen.push(bytes.clone());
            }
        }
        let mut body = vec![Tx::Coinbase { height, to: *to, amount: inner.genesis.subsidy + fees, extra: [0; 32] }.encode()];
        body.extend(chosen);
        let mut final_state = inner.state.clone();
        final_state.apply(height, &body, &inner.chain_id, &inner.native, inner.genesis.subsidy, &inner.genesis.hrp).map_err(|e| e.to_string())?;
        let header = Header {
            version: RULE_SET as u32, height, time: now.max(mtp(&inner, inner.tip.id) + 1), bits: inner.genesis.bits, nonce: [0; 16],
            prev: inner.tip.id, tx_root: tx_root(&body), state_root: final_state.root(), chain: inner.chain_id,
        };
        Ok((header, codec::encode_body(&body)))
    }

    fn events(&self) -> broadcast::Receiver<ChainEvent> { self.events.subscribe() }
}

/// chain-wire §3: the last 10 heights then doubling spacing back to genesis.
pub fn locator_heights(tip: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut h = tip;
    let mut step = 1u64;
    loop {
        out.push(h);
        if h == 0 {
            break;
        }
        if out.len() >= 10 {
            step *= 2;
        }
        h = h.saturating_sub(step);
    }
    out
}

/// A CPU nonce search for the harness: returns the solved header.
pub fn mine(header: Header, max_hashes: u64) -> Option<Header> {
    mine_from(header, 0, max_hashes)
}

/// Nonces `from .. from + count`.
pub fn mine_from(mut header: Header, from: u64, count: u64) -> Option<Header> {
    let target = decode_bits(header.bits)?;
    for n in from..from.saturating_add(count) {
        header.nonce = (n as u128).to_le_bytes();
        if U256::from_le_bytes(&header.id()) < target {
            return Some(header);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seed: u8) -> (mudra::SigningKey, Id) {
        let key = mudra::SigningKey::from_bytes((&[seed; 32]).into()).unwrap();
        let id = mudra::claim::neuron_of(&mudra::cosmos::compressed(key.verifying_key()));
        (key, id)
    }

    fn chain() -> (DevChain, Genesis) {
        let (_, alice) = key(1);
        let genesis = dev_genesis("dev", 1_000_000, &[(alice, 1_000_000_000)]);
        (DevChain::open(&genesis.encode(), None).unwrap(), genesis)
    }

    fn mine_next(chain: &DevChain, to: &Id, txs: &[Vec<u8>], now: u64) -> (Header, Vec<u8>) {
        let (header, body) = chain.template(to, txs, now).unwrap();
        (mine(header, 1 << 20).unwrap(), body)
    }

    #[test]
    fn genesis_round_trip() {
        let (_, g) = chain();
        assert_eq!(Genesis::decode(&g.encode()).unwrap(), g);
        assert!(Genesis::decode(b"junk").is_err());
    }

    #[test]
    fn locator_vector() {
        assert_eq!(locator_heights(1000), vec![1000, 999, 998, 997, 996, 995, 994, 993, 992, 991, 989, 985, 977, 961, 929, 865, 737, 481, 0]);
    }

    #[test]
    fn mine_transfer_and_reorg() {
        let (chain, _) = chain();
        let (alice_key, alice) = key(1);
        let (_, bob) = key(2);
        let miner = [9u8; 32];
        let now = chain.genesis_time() + 10;
        for i in 1..=3 {
            let (h, b) = mine_next(&chain, &miner, &[], now + i * 30);
            assert_eq!(chain.accept_header(&h, now + 1000).unwrap(), HeaderVerdict::Accepted);
            assert_eq!(chain.accept_block(&h, &b).unwrap(), BlockVerdict::NewTip);
        }
        assert_eq!(chain.tip().height, 3);
        assert!(chain.conserved());
        let tx = Tx::Transfer { from: alice, nonce: 0, token: chain.native_token(), amount: 5, to: bob, fee: 224, sig: vec![] };
        let signed = codec::sign_tx(&chain.chain_id(), &tx, &alice_key).unwrap().encode();
        let (h4, b4) = mine_next(&chain, &miner, &[signed.clone()], now + 200);
        chain.accept_header(&h4, now + 1000).unwrap();
        assert_eq!(chain.accept_block(&h4, &b4).unwrap(), BlockVerdict::NewTip);
        assert_eq!(chain.balance(&bob, &chain.native_token()), 5);
        assert_eq!(chain.nonce(&alice), 1);
        assert!(chain.tx_location(&txid(&chain.chain_id(), &signed)).is_some());
        assert!(chain.conserved());
        // immature coinbase cannot be spent
        let spend = Tx::Transfer { from: miner, nonce: 0, token: chain.native_token(), amount: 1, to: bob, fee: 224, sig: vec![0; 102] };
        let (h, b) = chain.template(&miner, &[spend.encode()], now + 300).unwrap();
        assert_eq!(decode_body(&b).unwrap().len(), 1, "unsigned/immature tx dropped");
        let _ = h;
        // a competing fork from height 2 with three blocks outweighs the main chain of four
        let fork_base = chain.header_at(2).unwrap();
        let mut prev = fork_base;
        let mut forks = Vec::new();
        let mut state_probe = DevChain::open(&chain.genesis().encode(), None).unwrap();
        for i in 1..=2 {
            let (h, b) = chain.block_at(i).unwrap();
            state_probe.accept_header(&h, now + 1000).unwrap();
            state_probe.accept_block(&h, &b).unwrap();
        }
        for i in 0..3 {
            let (h, b) = mine_next(&state_probe, &bob, &[], now + 400 + i * 30);
            state_probe.accept_header(&h, now + 1000).unwrap();
            state_probe.accept_block(&h, &b).unwrap();
            forks.push((h, b));
            prev = h;
        }
        let _ = prev;
        for (i, (h, b)) in forks.iter().enumerate() {
            assert_eq!(chain.accept_header(h, now + 1000).unwrap(), HeaderVerdict::Accepted);
            let verdict = chain.accept_block(h, b).unwrap();
            if i < 2 { assert_eq!(verdict, BlockVerdict::Stored) } else { assert_eq!(verdict, BlockVerdict::NewTip) }
        }
        assert_eq!(chain.tip().height, 5);
        assert_eq!(chain.balance(&bob, &chain.native_token()), 3 * chain.subsidy(), "reorged-out transfer is gone; bob keeps his fork coinbases");
        assert_eq!(chain.nonce(&alice), 0);
        assert!(chain.conserved());
        state_probe = DevChain::open(&chain.genesis().encode(), None).unwrap();
        drop(state_probe);
    }

    #[test]
    fn header_rules() {
        let (chain, _) = chain();
        let now = chain.genesis_time() + 10;
        let (h, b) = mine_next(&chain, &[1; 32], &[], now);
        let mut wrong_chain = h;
        wrong_chain.chain = [1; 32];
        assert_eq!(chain.accept_header(&wrong_chain, now), Err(HeaderError::WrongChain));
        let mut bad_pow = h;
        bad_pow.nonce = [0xff; 16];
        if !bad_pow.valid_pow() {
            assert_eq!(chain.accept_header(&bad_pow, now), Err(HeaderError::BadPow));
        }
        let mut orphan = h;
        orphan.prev = [7; 32];
        assert_eq!(chain.accept_header(&orphan, now), Ok(HeaderVerdict::Orphan));
        assert_eq!(chain.accept_header(&h, h.time - MAX_FUTURE_DRIFT - 1), Ok(HeaderVerdict::Held));
        assert_eq!(chain.accept_header(&h, now), Ok(HeaderVerdict::Accepted));
        assert_eq!(chain.accept_header(&h, now), Ok(HeaderVerdict::Known));
        let mut bad_body = b.clone();
        bad_body[4] ^= 1;
        assert!(chain.accept_block(&h, &bad_body).is_err());
        assert_eq!(chain.accept_block(&h, &b).unwrap(), BlockVerdict::NewTip);
        assert_eq!(chain.accept_block(&h, &b).unwrap(), BlockVerdict::Known);
    }

    #[test]
    fn persists_and_replays() {
        let (_, alice) = key(1);
        let genesis = dev_genesis("dev", 1_000_000, &[(alice, 1_000_000)]).encode();
        let home = std::env::temp_dir().join(format!("devchain-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        {
            let chain = DevChain::open(&genesis, Some(&home)).unwrap();
            for i in 1..=4 {
                let (h, b) = mine_next(&chain, &[1; 32], &[], 1_000_000 + i * 30);
                chain.accept_header(&h, u64::MAX / 2).unwrap();
                chain.accept_block(&h, &b).unwrap();
            }
            assert_eq!(chain.tip().height, 4);
        }
        let chain = DevChain::open(&genesis, Some(&home)).unwrap();
        assert_eq!(chain.tip().height, 4);
        assert!(chain.conserved());
        std::fs::remove_dir_all(&home).ok();
    }
}
