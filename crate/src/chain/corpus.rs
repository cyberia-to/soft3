//! The conformance corpus writer (proposals/proven-present.md row S0): every
//! block a node is given, in arrival order, with its native verdict and `R`
//! (the bbg state root after the call), plus a generator of invalid blocks.
//!
//! Format: JSON Lines, `<dir>/blocks.jsonl`, one record per line, append-only,
//! flushed per record. Fields:
//!
//! - `height`   the block's header height
//! - `id`       hex of the block id
//! - `block`    hex of [`Block::encode`] (`header[168] ‖ body`, the `B` frame payload)
//! - `verdict`  [`verdict_code`], or `"genesis"` for record 0
//! - `root`     hex of `R` (the tip's `state_root`) after the call when the verdict
//!              accepts; `null` for a rejection
//! - `mutation` `null` for a block as received/mined, else the generator's name
//!
//! Record 0 is block 0 with verdict `"genesis"` and `R(G)`.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::api::{Block, BlockVerdict, ChainApi, ChainError};
use super::apply::{BlockError, TxError};
use super::header::{self, Header, HeaderError};
use super::pow::valid_pow;
use super::state::Id;
use super::tree;
use super::tx::{self, Body, Tx};

/// The corpus file inside the corpus directory.
pub const FILE: &str = "blocks.jsonl";

/// Longest nonce search the generator runs for one block.
const GRIND_LIMIT: u128 = 1 << 20;

// ── verdict codes ───────────────────────────────────────────────────────

/// Stable snake_case code of an `accept_block` result. Every `match` here is
/// exhaustive on purpose: a new variant must fail the build, not be mis-coded.
pub fn verdict_code(r: &Result<BlockVerdict, ChainError>) -> String {
    match r {
        Ok(BlockVerdict::Extended) => "extended".into(),
        Ok(BlockVerdict::Reorganized { .. }) => "reorganized".into(),
        Ok(BlockVerdict::Stored) => "stored".into(),
        Ok(BlockVerdict::NeedBodies(_)) => "need_bodies".into(),
        Ok(BlockVerdict::Known) => "known".into(),
        Err(e) => format!("reject:{}", chain_error_code(e)),
    }
}

fn chain_error_code(e: &ChainError) -> String {
    match e {
        ChainError::Header(h) => format!("header:{}", header_error_code(h)),
        ChainError::UnknownParent(_) => "unknown_parent".into(),
        ChainError::InvalidParent(_) => "invalid_parent".into(),
        ChainError::Height { .. } => "height".into(),
        ChainError::Time { .. } => "time".into(),
        ChainError::FutureTime { .. } => "future_time".into(),
        ChainError::Bits { .. } => "bits".into(),
        ChainError::Pow => "pow".into(),
        ChainError::RuleSetUnknown(_) => "rule_set_unknown".into(),
        ChainError::DeepFork { .. } => "deep_fork".into(),
        ChainError::Checkpoint { .. } => "checkpoint".into(),
        ChainError::CheckpointRejected(_) => "checkpoint_rejected".into(),
        ChainError::Block(b) => format!("block:{}", block_error_code(b)),
        ChainError::Rejected(_) => "rejected".into(),
        ChainError::Genesis => "genesis".into(),
        ChainError::Store(_) => "store".into(),
    }
}

fn header_error_code(e: &HeaderError) -> &'static str {
    match e {
        HeaderError::Length(_) => "length",
        HeaderError::RuleSet(_) => "rule_set",
        HeaderError::WrongChain => "wrong_chain",
        HeaderError::Bits(_) => "bits",
    }
}

fn block_error_code(e: &BlockError) -> String {
    match e {
        BlockError::Body(_) => "body".into(),
        BlockError::Empty => "empty".into(),
        BlockError::TooManyTxs(_) => "too_many_txs".into(),
        BlockError::TooLarge(_) => "too_large".into(),
        BlockError::TxRoot => "tx_root".into(),
        BlockError::CoinbasePosition => "coinbase_position".into(),
        BlockError::CoinbaseHeight { .. } => "coinbase_height".into(),
        BlockError::DuplicateTx(_) => "duplicate_tx".into(),
        BlockError::CoinbaseAmount { .. } => "coinbase_amount".into(),
        BlockError::Tx { error, .. } => format!("tx:{}", tx_error_code(error)),
        BlockError::StateRoot { .. } => "state_root".into(),
        BlockError::Conservation { .. } => "conservation".into(),
        BlockError::Settlement { .. } => "settlement".into(),
        BlockError::StakeTotal { .. } => "stake_total".into(),
        BlockError::Overflow => "overflow".into(),
    }
}

fn tx_error_code(e: &TxError) -> &'static str {
    match e {
        TxError::Decode(_) => "decode",
        TxError::TooLarge(_) => "too_large",
        TxError::CoinbaseInBody => "coinbase_in_body",
        TxError::Signature => "signature",
        TxError::Nonce { .. } => "nonce",
        TxError::Fee { .. } => "fee",
        TxError::Insufficient { .. } => "insufficient",
        TxError::Immature { .. } => "immature",
        TxError::SelfLink => "self_link",
        TxError::StakeToken => "stake_token",
        TxError::Valence { .. } => "valence",
        TxError::NoCard => "no_card",
        TxError::Locked { .. } => "locked",
        TxError::UnlockAmount { .. } => "unlock_amount",
        TxError::ClaimKey => "claim_key",
        TxError::Overflow => "overflow",
    }
}

// ── the file ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct Record {
    height: u64,
    id: String,
    block: String,
    verdict: String,
    root: Option<String>,
    mutation: Option<String>,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.is_ascii() {
        return None;
    }
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()).collect()
}

/// Append-only writer of `<dir>/blocks.jsonl`.
pub struct CorpusWriter {
    file: File,
    path: PathBuf,
    records: usize,
}

impl CorpusWriter {
    /// Open (creating the directory and file if missing) for append.
    pub fn open(dir: &Path) -> io::Result<CorpusWriter> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(FILE);
        let records = match std::fs::read_to_string(&path) {
            Ok(text) => text.lines().filter(|l| !l.trim().is_empty()).count(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(e) => return Err(e),
        };
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(CorpusWriter { file, path, records })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Records in the file, including ones from earlier runs.
    pub fn len(&self) -> usize {
        self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records == 0
    }

    fn write(&mut self, rec: &Record) -> io::Result<()> {
        let mut line = serde_json::to_string(rec).map_err(io::Error::other)?;
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.flush()?;
        self.records += 1;
        Ok(())
    }

    /// Record 0: block 0 with verdict `"genesis"` and `R(G)`. Does nothing
    /// when the file already has records.
    pub fn record_genesis(&mut self, chain: &impl ChainApi) -> io::Result<()> {
        if !self.is_empty() {
            return Ok(());
        }
        let g = chain.header_at(0).and_then(|h| chain.block(&h.id())).ok_or_else(|| io::Error::other("chain has no block 0"))?;
        self.write(&Record {
            height: 0,
            id: hex(&g.id()),
            block: hex(&g.encode()),
            verdict: "genesis".into(),
            root: Some(hex(&g.header.state_root)),
            mutation: None,
        })
    }

    /// One record: `root` is `R` after the call; it is written only when the
    /// result accepts.
    pub fn record(&mut self, block: &Block, result: &Result<BlockVerdict, ChainError>, root: &Id, mutation: Option<&str>) -> io::Result<()> {
        self.write(&Record {
            height: block.header.height,
            id: hex(&block.id()),
            block: hex(&block.encode()),
            verdict: verdict_code(result),
            root: result.is_ok().then(|| hex(root)),
            mutation: mutation.map(str::to_string),
        })
    }
}

/// `accept_block`, then the record. Writes record 0 first if the file is
/// empty. The outer error is the file's; the inner is the chain's verdict.
pub fn record_accept<C: ChainApi>(
    chain: &mut C,
    writer: &mut CorpusWriter,
    block: &Block,
    now: u64,
    mutation: Option<&str>,
) -> io::Result<Result<BlockVerdict, ChainError>> {
    writer.record_genesis(&*chain)?;
    let result = chain.accept_block(block, now);
    let root = chain.tip().header.state_root;
    writer.record(block, &result, &root, mutation)?;
    Ok(result)
}

// ── replay ──────────────────────────────────────────────────────────────

/// The first record whose replay differs from the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMismatch {
    /// 1-based line in `blocks.jsonl` (0: the file could not be read)
    pub line: usize,
    pub reason: String,
}

impl std::fmt::Display for ReplayMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{FILE}:{}: {}", self.line, self.reason)
    }
}

impl std::error::Error for ReplayMismatch {}

/// Replay `<dir>/blocks.jsonl` on a fresh `chain`: record 0 must be the
/// chain's genesis and `R(G)`; every later block goes through `accept_block`
/// in file order and must reproduce the recorded verdict code and root.
/// Returns the number of records after genesis.
pub fn replay<C: ChainApi>(dir: &Path, chain: &mut C, now: u64) -> Result<usize, ReplayMismatch> {
    let text = std::fs::read_to_string(dir.join(FILE)).map_err(|e| ReplayMismatch { line: 0, reason: e.to_string() })?;
    let mut count = 0usize;
    let mut seen_genesis = false;
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let bad = |reason: String| ReplayMismatch { line, reason };
        let rec: Record = serde_json::from_str(raw).map_err(|e| bad(format!("not a record: {e}")))?;
        let bytes = unhex(&rec.block).ok_or_else(|| bad("block is not hex".into()))?;
        let block = Block::decode(&bytes).map_err(|e| bad(format!("block does not decode: {e}")))?;
        if hex(&block.id()) != rec.id || block.header.height != rec.height {
            return Err(bad("id/height do not match the block bytes".into()));
        }
        if !seen_genesis {
            seen_genesis = true;
            let g = chain.header_at(0).and_then(|h| chain.block(&h.id())).ok_or_else(|| bad("chain has no block 0".into()))?;
            if rec.verdict != "genesis" || rec.height != 0 {
                return Err(bad(format!("record 0 must be the genesis, found {:?} at height {}", rec.verdict, rec.height)));
            }
            if g.encode() != bytes {
                return Err(bad(format!("genesis block differs: file {}, chain {}", rec.id, hex(&g.id()))));
            }
            let want = hex(&g.header.state_root);
            if rec.root.as_deref() != Some(want.as_str()) {
                return Err(bad(format!("R(G) is {want}, file has {:?}", rec.root)));
            }
            continue;
        }
        let result = chain.accept_block(&block, now);
        let code = verdict_code(&result);
        if code != rec.verdict {
            return Err(bad(format!("height {}: verdict {code}, file has {}", rec.height, rec.verdict)));
        }
        let root = result.is_ok().then(|| hex(&chain.tip().header.state_root));
        if root != rec.root {
            return Err(bad(format!("height {}: R is {root:?}, file has {:?}", rec.height, rec.root)));
        }
        count += 1;
    }
    if !seen_genesis {
        return Err(ReplayMismatch { line: 0, reason: "empty corpus".into() });
    }
    Ok(count)
}

// ── the invalid-block generator ─────────────────────────────────────────

/// One mutated block and the verdict code it must draw.
#[derive(Clone, Debug)]
pub struct Invalid {
    pub name: &'static str,
    pub block: Block,
    pub expected: String,
}

/// First nonce (from 0) whose header does (`want = true`) or does not meet
/// its target; bounded, so a hard target yields `None` for `want = true`.
fn grind(mut h: Header, want: bool) -> Option<Header> {
    for n in 0..GRIND_LIMIT {
        h.nonce = n.to_le_bytes();
        if valid_pow(&h) == want {
            return Some(h);
        }
    }
    None
}

fn coinbase_bytes(height: u64, to: Id, amount: u64, extra: [u8; 32]) -> Vec<u8> {
    Tx { body: Body::Coinbase { height, to, amount, extra }, sig: vec![] }.encode()
}

/// For the valid next block of `chain`'s tip, one mutated block per
/// validation step (chain.md §5.5) that can be reached without a key. Each
/// fails at exactly its step: those targeting steps 6–8 get their nonce
/// re-ground so proof of work passes, and recompute the tx root unless that
/// is what they break. Deterministic. A mutation whose nonce search runs out
/// (hard target) is left out. Mutations needing a transaction (`bad_signature`,
/// `duplicate_tx`, `coinbase_position`) appear when `valid` carries one;
/// `replayed_tx` when the last 64 blocks of the main chain carry one.
pub fn invalid_blocks<C: ChainApi + ?Sized>(chain: &C, valid: &Block) -> Vec<Invalid> {
    let h = valid.header;
    let mut out: Vec<Invalid> = Vec::new();
    let mut add = |name: &'static str, header: Header, body: &[u8], expected: &str| {
        out.push(Invalid { name, block: Block { header, body: body.to_vec() }, expected: expected.into() });
    };

    // steps 1–5: header only (everything before PoW fails before PoW is read)
    add("wrong_chain", Header { chain: [0xEE; 32], ..h }, &valid.body, "reject:header:wrong_chain");
    add("rule_set", Header { version: (h.version & !0xffff) | 2, ..h }, &valid.body, "reject:header:rule_set");
    add("bits_zero", Header { bits: 0, ..h }, &valid.body, "reject:header:bits");
    add("second_genesis", Header { height: 0, ..h }, &valid.body, "reject:genesis");
    add("unknown_parent", Header { prev: [0xAB; 32], ..h }, &valid.body, "reject:unknown_parent");
    add("wrong_height", Header { height: h.height + 1, ..h }, &valid.body, "reject:height");
    add("time_not_after_mtp", Header { time: 0, ..h }, &valid.body, "reject:time");
    if let Ok(target) = header::decode_bits(h.bits) {
        let harder = target >> 1;
        if !harder.is_zero() {
            add("wrong_bits", Header { bits: header::encode_bits(harder), ..h }, &valid.body, "reject:bits");
        }
    }
    if let Some(bad) = grind(h, false) {
        add("bad_pow", bad, &valid.body, "reject:pow");
    }

    // steps 6–8: body and state rules, the nonce re-ground
    let mut sealed = |name: &'static str, header: Header, body: Vec<u8>, expected: &str| {
        if let Some(g) = grind(header, true) {
            add(name, g, &body, expected);
        }
    };
    let rooted = |raws: &[Vec<u8>]| Header { tx_root: tree::tx_root(raws), ..h };

    sealed("tx_root", Header { tx_root: flip(h.tx_root), ..h }, valid.body.clone(), "reject:block:tx_root");
    sealed("state_root", Header { state_root: flip(h.state_root), ..h }, valid.body.clone(), "reject:block:state_root");
    sealed("empty_body", h, tx::join_body(&[]), "reject:block:empty");
    if let Some(cut) = valid.body.len().checked_sub(1) {
        sealed("truncated_body", h, valid.body[..cut].to_vec(), "reject:block:body");
    }

    let raws = tx::split_body(&valid.body).unwrap_or_default();
    let cb = raws.first().and_then(|r| Tx::decode(r).ok()).and_then(|t| match t.body {
        Body::Coinbase { height, to, amount, extra } => Some((height, to, amount, extra)),
        _ => None,
    });
    if let Some((height, to, amount, extra)) = cb {
        let cb_raw = &raws[0];
        let with = |first: Vec<u8>| -> Vec<Vec<u8>> {
            let mut v = raws.clone();
            v[0] = first;
            v
        };
        let r = with(coinbase_bytes(height + 1, to, amount, extra));
        sealed("coinbase_height", rooted(&r), tx::join_body(&r), "reject:block:coinbase_height");
        let r = with(coinbase_bytes(height, to, amount + 1, extra));
        sealed("coinbase_amount", rooted(&r), tx::join_body(&r), "reject:block:coinbase_amount");
        let r = vec![cb_raw.clone(), cb_raw.clone()];
        sealed("coinbase_in_body", rooted(&r), tx::join_body(&r), "reject:block:tx:coinbase_in_body");

        if let Some(t) = raws.get(1) {
            let r = vec![cb_raw.clone(), t.clone(), t.clone()];
            sealed("duplicate_tx", rooted(&r), tx::join_body(&r), "reject:block:duplicate_tx");
            let mut bad = t.clone();
            let last = bad.len() - 1;
            bad[last] ^= 1;
            let r = with_tx(&raws, 1, bad);
            sealed("bad_signature", rooted(&r), tx::join_body(&r), "reject:block:tx:signature");
            let r = vec![t.clone(), cb_raw.clone()];
            sealed("coinbase_position", rooted(&r), tx::join_body(&r), "reject:block:coinbase_position");
        }

        // a transaction an earlier block spent: its nonce is behind the account's
        let tip = chain.tip().height;
        let old = (1..=tip).rev().take(64).find_map(|n| {
            let b = chain.blocks(n, 1).into_iter().next()?;
            tx::split_body(&b.body).ok()?.into_iter().skip(1).last()
        });
        let fees = raws.iter().skip(1).try_fold(0u64, |a, r| a.checked_add(Tx::decode(r).ok()?.fee()));
        if let (Some(old), Some(fees)) = (old, fees) {
            let old_fee = Tx::decode(&old).map(|t| t.fee()).unwrap_or(0);
            if let Some(amount) = amount.checked_sub(fees).and_then(|share| share.checked_add(old_fee)) {
                let r = vec![coinbase_bytes(height, to, amount, extra), old];
                sealed("replayed_tx", rooted(&r), tx::join_body(&r), "reject:block:tx:nonce");
            }
        }
    }
    out
}

fn with_tx(raws: &[Vec<u8>], at: usize, replacement: Vec<u8>) -> Vec<Vec<u8>> {
    let mut v = raws.to_vec();
    v[at] = replacement;
    v
}

fn flip(mut b: [u8; 32]) -> [u8; 32] {
    b[0] ^= 1;
    b
}
