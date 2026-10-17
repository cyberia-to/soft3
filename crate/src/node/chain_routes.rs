//! HTTP of a chain node — chain-wire.md §7 on the existing adapter: the
//! routes cyb and true-cyber read, the new chain routes, the mining
//! template, `POST /tx`, and `410 Gone` for the unsigned chaosnet ingress.
//! Dev chains add `/dev/*` switches for the local harness.

use std::sync::atomic::Ordering;

use serde::Deserialize;

use super::chain_node::{ChainNode, SubmitError};
use super::http::{Error, Request, Response};
use crate::chain::net::codec::{self, hex, txid, unhex32, Header, Tx};
use crate::chain::net::mempool::AdmitError;
use crate::chain::net::peers::Direction;

fn json(value: serde_json::Value) -> Response {
    Response::bytes("application/json", format!("{value}\n").into_bytes())
}

fn not_found(code: &'static str, message: String) -> Error {
    Error { status: "404 Not Found", code, message }
}

fn gone() -> Error {
    Error {
        status: "410 Gone",
        code: "unsigned_ingress_removed",
        message: "the unsigned chaosnet ingress has no place on a public chain: sign a transaction and POST /tx (spacepussy-test keeps these routes)".into(),
    }
}

fn query(q: &str, name: &str) -> Result<Option<String>, Error> {
    let mut out = None;
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == name {
                if out.replace(v.to_owned()).is_some() {
                    return Err(Error::bad(format!("duplicate {name}")));
                }
            }
        }
    }
    Ok(out)
}

fn query_u64(q: &str, name: &str) -> Result<Option<u64>, Error> {
    query(q, name)?.map(|v| v.parse().map_err(|_| Error::bad(format!("invalid {name}")))).transpose()
}

fn id_or_height(node: &ChainNode, s: &str) -> Result<Header, Error> {
    if let Some(id) = unhex32(s) {
        return node.chain.header(&id).ok_or_else(|| not_found("block_not_found", format!("no block {s}")));
    }
    let height: u64 = s.parse().map_err(|_| Error::bad("height or 64-hex id expected"))?;
    node.chain.header_at(height).ok_or_else(|| not_found("block_not_found", format!("no block at height {height}")))
}

fn header_json(h: &Header) -> serde_json::Value {
    serde_json::json!({
        "id": hex(&h.id()), "hex": hex(&h.encode()), "version": h.version, "height": h.height, "time": h.time,
        "bits": format!("{:#010x}", h.bits), "nonce": hex(&h.nonce), "prev": hex(&h.prev), "tx_root": hex(&h.tx_root),
        "state_root": hex(&h.state_root), "chain": hex(&h.chain),
    })
}

fn kind_name(tx: &Tx) -> &'static str {
    match tx {
        Tx::Coinbase { .. } => "coinbase",
        Tx::Transfer { .. } => "transfer",
        Tx::Cyberlink { .. } => "cyberlink",
        Tx::Claim { .. } => "claim",
        Tx::Unlock { .. } => "unlock",
        Tx::Payout { .. } => "payout",
    }
}

/// The decoded fields of chain-wire §7 `GET /tx/{txid}`.
fn tx_fields(tx: &Tx) -> serde_json::Value {
    let mut v = match tx {
        Tx::Coinbase { height, to, amount, extra } => serde_json::json!({"height": height, "to": hex(to), "amount": amount, "extra": hex(extra)}),
        Tx::Transfer { from, nonce, token, amount, to, fee, .. } => serde_json::json!({"from": hex(from), "nonce": nonce, "token": hex(token), "amount": amount, "to": hex(to), "fee": fee}),
        Tx::Cyberlink { from, nonce, token, amount, p, q, valence, fee, .. } => serde_json::json!({"from": hex(from), "nonce": nonce, "token": hex(token), "amount": amount, "p": hex(p), "q": hex(q), "valence": valence, "fee": fee}),
        Tx::Claim { account_id, nonce, fee, .. } => serde_json::json!({"account_id": hex(account_id), "nonce": nonce, "fee": fee}),
        Tx::Unlock { from, nonce, p, q, amount, fee, .. } => serde_json::json!({"from": hex(from), "nonce": nonce, "p": hex(p), "q": hex(q), "amount": amount, "fee": fee}),
        Tx::Payout { from, nonce, p, q, payout, fee, .. } => serde_json::json!({"from": hex(from), "nonce": nonce, "p": hex(p), "q": hex(q), "payout": hex(payout), "fee": fee}),
    };
    if !tx.sig().is_empty() {
        v["sig"] = hex(tx.sig()).into();
    }
    v
}

fn error(status: &'static str, code: &'static str, message: impl Into<String>) -> Error {
    Error { status, code, message: message.into() }
}

fn block_json(node: &ChainNode, header: &Header, body: &[u8]) -> serde_json::Value {
    let chain_id = node.chain.chain_id();
    let txs = node.chain.txs_of(body);
    let fees: u64 = txs.iter().filter_map(|t| Tx::decode(t).ok()).map(|t| t.fee()).sum();
    let miner = txs.first().and_then(|t| Tx::decode(t).ok()).and_then(|t| match t { Tx::Coinbase { to, .. } => Some(hex(&to)), _ => None });
    let mut v = header_json(header);
    v["txids"] = txs.iter().map(|t| hex(&txid(&chain_id, t))).collect();
    v["txs"] = txs.len().into();
    v["size"] = (codec::HEADER_BYTES + body.len()).into();
    v["miner"] = miner.into();
    v["fees"] = fees.into();
    v
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TxBody {
    tx: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubmitBody {
    header: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IsolateBody {
    on: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DialBody {
    id: String,
    addrs: Vec<String>,
}

/// `POST /tx` body (chain-wire §7): `application/octet-stream` tx bytes, or
/// JSON `{"tx":"hex"}`, or bare hex text. A body that is not UTF-8, or is
/// UTF-8 but neither JSON nor hex, is the raw wire bytes — what
/// `soft3::wallet` (cyber, cyb) sends.
fn tx_body(body: &[u8]) -> Result<Vec<u8>, Error> {
    let Ok(text) = std::str::from_utf8(body) else { return Ok(body.to_vec()) };
    let text = text.trim();
    let hex_str = if text.starts_with('{') {
        serde_json::from_str::<TxBody>(text).map_err(|e| Error::bad(e.to_string()))?.tx
    } else if !text.is_empty() && text.bytes().all(|b| b.is_ascii_hexdigit()) {
        text.to_owned()
    } else {
        return Ok(body.to_vec());
    };
    (0..hex_str.len()).step_by(2).map(|i| u8::from_str_radix(hex_str.get(i..i + 2).ok_or_else(|| Error::bad("odd hex length"))?, 16).map_err(|_| Error::bad("invalid hex"))).collect()
}

pub(super) fn route(node: &ChainNode, request: Request) -> Result<Response, Error> {
    let path = if request.path == "/" { "/" } else { request.path.trim_end_matches('/') };
    let q = request.query.as_str();
    match (request.method.as_str(), path) {
        ("GET", "/health") => Ok(Response::text("ok\n")),
        ("GET", "/status") => Ok(Response::text(node.status_cybermark())),
        ("GET", "/root") => Ok(Response::text(format!("{}\n", hex(&node.chain.state_root())))),
        ("GET", "/stats") => {
            let (neurons, particles, axons) = node.chain.counts();
            Ok(Response::text(format!(
                "---\nparticle: stats\nheight: {}\nbbg-root: {}\nneurons: {}\nparticles: {}\naxons: {}\nmempool: {}\npeers: {}\n---\n",
                node.chain.tip().height, hex(&node.chain.state_root()), neurons, particles, axons,
                node.mempool.lock().expect("mempool").len(), node.net.peers().len(),
            )))
        }
        ("GET", "/") => Ok(Response::text(format!(
            "soft3 · {} · chain+cybergraph+bbg\nmoniker {}\nheight {}\nGET /status /tip /header/H /block/H /blocks /tx/ID /nonce/N /balance/N /balances/N /mempool /peers /checkpoints /template?to=N /v2/history /economy /epoch/E /epochs /focus /focus/P /card/C /cards/N\nPOST /tx /submit\n",
            node.chain_name, node.moniker, node.chain.tip().height,
        ))),
        // chain-wire §7: the kernel renders the answer
        ("GET", "/tip") => Ok(Response::bytes("application/json", format!("{}\n", node.kernel.tip_json()).into_bytes())),
        ("GET", p) if p.starts_with("/header/") => Ok(json(header_json(&id_or_height(node, &p["/header/".len()..])?))),
        ("GET", p) if p.starts_with("/block/") => {
            let header = id_or_height(node, &p["/block/".len()..])?;
            let (_, body) = node.chain.block(&header.id()).ok_or_else(|| not_found("body_not_found", "header known, body not held".into()))?;
            Ok(json(block_json(node, &header, &body)))
        }
        ("GET", "/blocks") => {
            let limit = query_u64(q, "limit")?.unwrap_or(50).clamp(1, 500);
            let after = query_u64(q, "after")?;
            let tip = node.chain.tip().height;
            let start = after.map(|a| a + 1).unwrap_or_else(|| tip.saturating_sub(limit - 1));
            let mut out = Vec::new();
            for h in start..=tip {
                if out.len() as u64 >= limit {
                    break;
                }
                if let Some((header, body)) = node.chain.block_at(h) {
                    let txs = node.chain.txs_of(&body);
                    let miner = txs.first().and_then(|t| Tx::decode(t).ok()).and_then(|t| match t { Tx::Coinbase { to, .. } => Some(hex(&to)), _ => None });
                    out.push(serde_json::json!({"height": h, "id": hex(&header.id()), "time": header.time, "txs": txs.len(), "size": codec::HEADER_BYTES + body.len(), "miner": miner}));
                }
            }
            Ok(json(serde_json::json!({"blocks": out, "tip": tip})))
        }
        ("GET", p) if p.starts_with("/tx/") => {
            let id = unhex32(&p["/tx/".len()..]).ok_or_else(|| Error::bad("txid is 64 hex chars"))?;
            let chain_id = node.chain.chain_id();
            if let Some((height, block, index)) = node.chain.tx_location(&id) {
                let (_, body) = node.chain.block(&block).ok_or_else(|| error("404 Not Found", "unknown", "unknown"))?;
                let bytes = node.chain.txs_of(&body).into_iter().find(|t| txid(&chain_id, t) == id).ok_or_else(|| error("404 Not Found", "unknown", "unknown"))?;
                let tx = Tx::decode(&bytes).map_err(|e| Error::bad(e.to_string()))?;
                return Ok(json(serde_json::json!({
                    "txid": hex(&id), "kind": kind_name(&tx), "tx": tx_fields(&tx), "height": height, "block": hex(&block), "index": index,
                })));
            }
            let pool = node.mempool.lock().expect("mempool");
            let entry = pool.get(&id).ok_or_else(|| error("404 Not Found", "unknown", "unknown"))?;
            Ok(json(serde_json::json!({
                "txid": hex(&id), "kind": kind_name(&entry.tx), "tx": tx_fields(&entry.tx), "height": "mempool", "block": null, "index": null,
            })))
        }
        // chain-economy §11
        ("GET", p) if p.starts_with("/epoch/") => {
            let e: u64 = p["/epoch/".len()..].parse().map_err(|_| Error::bad("epoch is a number"))?;
            node.kernel.economy(|c| c.epoch_json(e)).map(json).ok_or_else(|| not_found("epoch_not_settled", format!("epoch {e} is not settled on this chain")))
        }
        ("GET", "/epochs") => {
            let after = query(q, "after")?.map(|v| v.parse::<u64>()).transpose().map_err(|_| Error::bad("after is a number"))?.unwrap_or(0);
            let limit = query(q, "limit")?.map(|v| v.parse::<usize>()).transpose().map_err(|_| Error::bad("limit is a number"))?.unwrap_or(100);
            Ok(json(node.kernel.economy(|c| Some(c.epochs_json(after, limit))).unwrap_or_default()))
        }
        ("GET", "/focus") => {
            let top = query(q, "top")?.map(|v| v.parse::<usize>()).transpose().map_err(|_| Error::bad("top is a number"))?.unwrap_or(20);
            node.kernel.economy(|c| c.focus_top_json(top)).map(json).ok_or_else(|| not_found("focus_not_computed", "no settled epoch with φ* on this node yet".into()))
        }
        ("GET", p) if p.starts_with("/focus/") => {
            let particle = unhex32(&p["/focus/".len()..]).ok_or_else(|| Error::bad("particle is 64 hex chars"))?;
            node.kernel.economy(|c| c.focus_json(&particle)).map(json).ok_or_else(|| not_found("no_focus", "the particle has no focus at the last settled epoch".into()))
        }
        ("GET", p) if p.starts_with("/card/") => {
            let card = unhex32(&p["/card/".len()..]).ok_or_else(|| Error::bad("card is 64 hex chars"))?;
            node.kernel.economy(|c| c.card_json(&card)).map(json).ok_or_else(|| not_found("card_not_found", "no position with this id".into()))
        }
        ("GET", p) if p.starts_with("/cards/") => {
            let neuron = unhex32(&p["/cards/".len()..]).ok_or_else(|| Error::bad("neuron is 64 hex chars"))?;
            Ok(json(node.kernel.economy(|c| Some(c.cards_json(&neuron))).unwrap_or_default()))
        }
        ("GET", "/economy") => Ok(json(node.kernel.economy(|c| Some(c.economy_status())).unwrap_or_default())),
        ("GET", p) if p.starts_with("/nonce/") => {
            let neuron = unhex32(&p["/nonce/".len()..]).ok_or_else(|| Error::bad("not a 32-byte hex id"))?;
            let next = node.mempool.lock().expect("mempool").next_nonce(node.chain.as_ref(), &neuron);
            Ok(json(serde_json::json!({"neuron": hex(&neuron), "nonce": next, "confirmed": node.chain.nonce(&neuron)})))
        }
        ("GET", p) if p.starts_with("/balance/") => {
            let neuron = unhex32(&p["/balance/".len()..]).ok_or_else(|| Error::bad("neuron is 64 hex chars"))?;
            let token = match query(q, "token")? {
                Some(t) => unhex32(&t).ok_or_else(|| Error::bad("token is 64 hex chars"))?,
                None => node.chain.native_token(),
            };
            Ok(Response::text(format!(
                "---\nparticle: balance\nneuron: {}\ndenom: {}\ntoken: {}\nbalance: {}\nspendable: {}\nheight: {}\nsupply: {}\n---\n",
                hex(&neuron), node.chain_name, hex(&token), node.chain.balance(&neuron, &token), node.chain.spendable(&neuron, &token),
                node.chain.tip().height, node.chain.supply(&token),
            )))
        }
        ("GET", p) if p.starts_with("/balances/") => {
            let owner = unhex32(&p["/balances/".len()..]).ok_or_else(|| Error::bad("owner is 64 hex chars"))?;
            let balances: Vec<_> = node.chain.balances(&owner).into_iter().map(|(t, a)| serde_json::json!({"token": hex(&t), "amount": a})).collect();
            Ok(json(serde_json::json!({"owner": hex(&owner), "balances": balances, "height": node.chain.tip().height})))
        }
        ("GET", "/mempool") => {
            let pool = node.mempool.lock().expect("mempool");
            let ids: Vec<String> = pool.ids().iter().map(|id| hex(id)).collect();
            Ok(json(serde_json::json!({"count": pool.len(), "bytes": pool.bytes(), "txids": ids})))
        }
        ("GET", "/peers") => {
            let peers: Vec<_> = node.net.peers().into_iter().map(|p| serde_json::json!({
                "endpoint": hex(p.id.as_bytes()), "direction": if p.direction == Direction::Outbound { "outbound" } else { "inbound" },
                "best_height": p.best_height, "best_tip": hex(&p.best_tip), "score": p.score,
                "addrs": p.addrs.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            })).collect();
            Ok(json(serde_json::json!({"endpoint": hex(node.net.id().as_bytes()), "peers": peers, "outbound": node.net.outbound(), "isolated": node.net.is_isolated(), "book": node.net.book_len()})))
        }
        ("GET", "/checkpoints") => {
            let list: Vec<_> = node.net.checkpoints().iter().map(|c| serde_json::json!({"height": c.height, "block": hex(&c.block_id), "sig": hex(&c.sig)})).collect();
            Ok(json(serde_json::json!({"checkpoints": list})))
        }
        ("GET", "/template") => {
            let to = query(q, "to")?.and_then(|t| unhex32(&t)).ok_or_else(|| Error::bad("to=<64 hex neuron> required"))?;
            let txs = node.mempool.lock().expect("mempool").template_txs();
            let (header, body) = node.chain.template(&to, &txs, crate::chain::net::now_unix()).map_err(|e| error("503 Service Unavailable", "no_template", e))?;
            let mut templates = node.templates.lock().expect("templates");
            templates.retain(|(_, h, _)| h.prev == header.prev);
            templates.push((header.id(), header, body.clone()));
            if templates.len() > 64 {
                templates.remove(0);
            }
            node.last_template.store(crate::chain::net::now_unix(), Ordering::Relaxed);
            Ok(Response::bytes("application/json", format!("{}\n", node.chain.template_json(&header, &body, 0)).into_bytes()))
        }
        ("POST", "/submit") => {
            let text = std::str::from_utf8(&request.body).map_err(|_| Error::bad("hex header expected"))?.trim();
            let hex_str = if text.starts_with('{') { serde_json::from_str::<SubmitBody>(text).map_err(|e| Error::bad(e.to_string()))?.header } else { text.to_owned() };
            let bytes: Vec<u8> = (0..hex_str.len()).step_by(2).map(|i| u8::from_str_radix(hex_str.get(i..i + 2).unwrap_or("zz"), 16).map_err(|_| Error::bad("invalid hex"))).collect::<Result<_, _>>()?;
            let header = Header::decode(&bytes).map_err(|e| Error::bad(e.to_string()))?;
            match node.submit(header) {
                Ok(id) => Ok(json(serde_json::json!({"id": hex(&id), "height": header.height}))),
                Err(SubmitError::Stale) => Err(error("409 Conflict", "stale_template", "stale template")),
                Err(SubmitError::Rejected(e)) => Err(error("422 Unprocessable Content", "header_rejected", e)),
            }
        }
        ("POST", "/tx") => {
            let bytes = tx_body(&request.body)?;
            let tx = Tx::decode(&bytes).map_err(|e| error("400 Bad Request", "mempool_rejected", format!("decode: {e}")))?;
            let id = txid(&node.chain.chain_id(), &bytes);
            let admitted = node.mempool.lock().expect("mempool").admit(&bytes, node.chain.as_ref(), crate::chain::net::now_unix());
            match admitted {
                Ok(_) | Err(AdmitError::Known) => {}
                Err(e) => return Err(error("400 Bad Request", "mempool_rejected", e.to_string())),
            }
            let (account, nonce) = {
                let pool = node.mempool.lock().expect("mempool");
                pool.get(&id).map(|e| (e.from, e.nonce)).unwrap_or_else(|| (tx.subject().unwrap_or([0; 32]), tx.nonce().unwrap_or(0)))
            };
            node.runtime.block_on(node.net.publish_tx(&bytes));
            Ok(json(serde_json::json!({
                "txid": hex(&id), "kind": kind_name(&tx), "size": bytes.len(), "fee": tx.fee(), "account": hex(&account), "nonce": nonce,
            })))
        }
        ("GET", "/v2/history") => {
            let after = query_u64(q, "after")?;
            let limit = query_u64(q, "limit")?.unwrap_or(16);
            if !(1..=64).contains(&limit) {
                return Err(Error::bad("history limit must be 1..64"));
            }
            let chain_id = node.chain.chain_id();
            let tip = node.chain.tip().height;
            let (mut height, mut index) = match after { Some(p) => (p >> 20, (p & 0xfffff) + 1), None => (1, 0) };
            let mut entries = Vec::new();
            let mut next = after;
            'outer: while height <= tip {
                let Some((header, body)) = node.chain.block_at(height) else { break };
                let txs = node.chain.txs_of(&body);
                let block = hex(&header.id());
                let root = hex(&header.state_root);
                for (i, bytes) in txs.iter().enumerate().skip(index as usize) {
                    let position = (height << 20) | i as u64;
                    let kind = Tx::decode(bytes).map(|t| t.kind()).unwrap_or(255);
                    entries.push(serde_json::json!({
                        "receipt": {"position": position, "height": height, "block": block, "txid": hex(&txid(&chain_id, bytes)), "root": root},
                        "operation": hex(bytes), "kind": kind,
                    }));
                    next = Some(position);
                    if entries.len() as u64 >= limit {
                        break 'outer;
                    }
                }
                height += 1;
                index = 0;
            }
            Ok(json(serde_json::json!({"schema": "cyber/native-history/v1", "encoding": "chain-tx/v1-hex", "entries": entries, "next": next})))
        }
        ("POST", "/v1/link" | "/v1/pay" | "/v1/frame" | "/v2/frame" | "/v1/finalize") | ("GET", "/log") => Err(gone()),
        ("POST", "/dev/isolate") if node.dev => {
            let body: IsolateBody = serde_json::from_slice(&request.body).map_err(|e| Error::bad(e.to_string()))?;
            node.net.set_isolated(body.on);
            Ok(json(serde_json::json!({"isolated": body.on})))
        }
        ("POST", "/dev/dial") if node.dev => {
            let body: DialBody = serde_json::from_slice(&request.body).map_err(|e| Error::bad(e.to_string()))?;
            let id = unhex32(&body.id).and_then(|b| radio::EndpointId::from_bytes(&b).ok()).ok_or_else(|| Error::bad("id is a 64-hex endpoint id"))?;
            let addrs: Vec<radio::TransportAddr> = body.addrs.iter().filter_map(|a| a.parse().ok()).map(radio::TransportAddr::Ip).collect();
            let addr = radio::EndpointAddr::from_parts(id, addrs);
            node.runtime.block_on(node.net.dial(addr)).map_err(Error::bad)?;
            Ok(json(serde_json::json!({"dialed": body.id})))
        }
        _ => Err(not_found("not_found", "not found".into())),
    }
}


#[cfg(test)]
mod tx_body_tests {
    use super::tx_body;

    #[test]
    fn raw_hex_and_json_bodies_decode_to_the_same_bytes() {
        // a transfer-shaped body: kind byte, then bytes that are not UTF-8
        let raw: Vec<u8> = [1u8, 0, 0, 0].iter().copied().chain((0..=255u8).rev()).collect();
        assert!(std::str::from_utf8(&raw).is_err());
        let hex: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(tx_body(&raw).unwrap(), raw, "application/octet-stream");
        assert_eq!(tx_body(hex.as_bytes()).unwrap(), raw, "hex text");
        assert_eq!(tx_body(format!("{hex}\n").as_bytes()).unwrap(), raw, "hex text with newline");
        assert_eq!(tx_body(format!("{{\"tx\":\"{hex}\"}}").as_bytes()).unwrap(), raw, "JSON");
    }

    #[test]
    fn utf8_raw_bytes_that_are_not_hex_pass_through() {
        let raw = b"\x01\x00\x00\x00 plain ascii is not hex".to_vec();
        assert_eq!(tx_body(&raw).unwrap(), raw);
    }

    #[test]
    fn malformed_json_and_odd_hex_are_rejected() {
        assert!(tx_body(b"{\"tx\": 5}").is_err());
        assert!(tx_body(b"abc").is_err());
    }
}
