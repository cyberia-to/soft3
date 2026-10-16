//! HTTP of a chain node — chain-wire.md §7 on the existing adapter: the
//! routes cyb and true-cyber read, the new chain routes, the mining
//! template, `POST /tx`, and `410 Gone` for the unsigned chaosnet ingress.
//! Dev chains add `/dev/*` switches for the local harness.

use std::sync::atomic::Ordering;

use serde::Deserialize;

use super::chain_node::ChainNode;
use super::http::{Error, Request, Response};
use crate::chain::net::codec::{self, hex, txid, unhex32, Header, Tx};
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

fn tx_json(tx: &Tx) -> serde_json::Value {
    match tx {
        Tx::Coinbase { height, to, amount, extra } => serde_json::json!({"kind": "coinbase", "height": height, "to": hex(to), "amount": amount, "extra": hex(extra)}),
        Tx::Transfer { from, nonce, token, amount, to, fee, .. } => serde_json::json!({"kind": "transfer", "from": hex(from), "nonce": nonce, "token": hex(token), "amount": amount, "to": hex(to), "fee": fee}),
        Tx::Cyberlink { from, nonce, token, amount, p, q, valence, fee, .. } => serde_json::json!({"kind": "cyberlink", "from": hex(from), "nonce": nonce, "token": hex(token), "amount": amount, "p": hex(p), "q": hex(q), "valence": valence, "fee": fee}),
        Tx::Claim { account_id, nonce, fee, .. } => serde_json::json!({"kind": "claim", "account_id": hex(account_id), "nonce": nonce, "fee": fee}),
    }
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
struct MineBody {
    on: bool,
    #[serde(default)]
    solo: Option<bool>,
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

fn body_hex(request: &Request) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(&request.body).map_err(|_| Error::bad("expected hex or JSON"))?.trim();
    let hex_str = if text.starts_with('{') {
        serde_json::from_str::<TxBody>(text).map_err(|e| Error::bad(e.to_string()))?.tx
    } else {
        text.to_owned()
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
            "soft3 · {} · chain+cybergraph+bbg\nmoniker {}\nheight {}\nGET /status /tip /header/H /block/H /blocks /tx/ID /nonce/N /balance/N /balances/N /mempool /peers /checkpoints /template?to=N /v2/history\nPOST /tx /submit\n",
            node.chain_name, node.moniker, node.chain.tip().height,
        ))),
        ("GET", "/tip") => {
            let tip = node.chain.tip();
            Ok(json(serde_json::json!({"height": tip.height, "id": hex(&tip.id), "chainwork": tip.chainwork.to_decimal()})))
        }
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
            if let Some((height, block)) = node.chain.tx_location(&id) {
                let (_, body) = node.chain.block(&block).ok_or_else(|| not_found("body_not_found", "body not held".into()))?;
                let bytes = node.chain.txs_of(&body).into_iter().find(|t| txid(&chain_id, t) == id).ok_or_else(|| not_found("tx_not_found", "index stale".into()))?;
                let tx = Tx::decode(&bytes).map_err(|e| Error::bad(e.to_string()))?;
                let mut v = tx_json(&tx);
                v["txid"] = hex(&id).into();
                v["height"] = height.into();
                v["block"] = hex(&block).into();
                v["hex"] = hex(&bytes).into();
                return Ok(json(v));
            }
            let pool = node.mempool.lock().expect("mempool");
            let entry = pool.get(&id).ok_or_else(|| not_found("tx_not_found", format!("no transaction {}", hex(&id))))?;
            let mut v = tx_json(&entry.tx);
            v["txid"] = hex(&id).into();
            v["height"] = "mempool".into();
            v["hex"] = hex(&entry.bytes).into();
            Ok(json(v))
        }
        ("GET", p) if p.starts_with("/nonce/") => {
            let neuron = unhex32(&p["/nonce/".len()..]).ok_or_else(|| Error::bad("neuron is 64 hex chars"))?;
            let next = node.mempool.lock().expect("mempool").next_nonce(node.chain.as_ref(), &neuron);
            Ok(json(serde_json::json!({"neuron": hex(&neuron), "nonce": next, "chain_nonce": node.chain.nonce(&neuron)})))
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
            let (header, body) = node.chain.template(&to, &txs, crate::chain::net::now_unix()).map_err(Error::bad)?;
            let target = codec::decode_bits(header.bits).map(|t| hex(&t.to_le_bytes())).unwrap_or_default();
            let mut templates = node.templates.lock().expect("templates");
            templates.retain(|(_, h, _)| h.prev == header.prev);
            templates.push((header.id(), header, body.clone()));
            templates.truncate(64);
            Ok(json(serde_json::json!({
                "chain": node.chain_name, "height": header.height, "prev": hex(&header.prev), "header": hex(&header.encode()),
                "bits": format!("{:08x}", header.bits), "target": target, "body": hex(&body), "expires": 0,
            })))
        }
        ("POST", "/submit") => {
            let text = std::str::from_utf8(&request.body).map_err(|_| Error::bad("hex header expected"))?.trim();
            let hex_str = if text.starts_with('{') { serde_json::from_str::<SubmitBody>(text).map_err(|e| Error::bad(e.to_string()))?.header } else { text.to_owned() };
            let bytes: Vec<u8> = (0..hex_str.len()).step_by(2).map(|i| u8::from_str_radix(hex_str.get(i..i + 2).unwrap_or("zz"), 16).map_err(|_| Error::bad("invalid hex"))).collect::<Result<_, _>>()?;
            let header = Header::decode(&bytes).map_err(|e| Error::bad(e.to_string()))?;
            match node.submit(header) {
                Ok(id) => Ok(json(serde_json::json!({"id": hex(&id), "height": header.height}))),
                Err("stale template") => Err(Error { status: "409 Conflict", code: "stale_template", message: "stale template".into() }),
                Err(e) => Err(Error { status: "422 Unprocessable Content", code: "header_rejected", message: e.into() }),
            }
        }
        ("POST", "/tx") => {
            let bytes = body_hex(&request)?;
            let id = node.mempool.lock().expect("mempool").admit(&bytes, node.chain.as_ref(), crate::chain::net::now_unix()).map_err(|e| Error { status: "400 Bad Request", code: "mempool_rejected", message: e.to_string() })?;
            node.runtime.block_on(node.net.publish_tx(&bytes));
            Ok(json(serde_json::json!({"txid": hex(&id)})))
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
        ("POST", "/dev/mine") if node.dev => {
            let body: MineBody = serde_json::from_slice(&request.body).map_err(|e| Error::bad(e.to_string()))?;
            node.mining.store(body.on, Ordering::Relaxed);
            if let Some(solo) = body.solo {
                node.solo.store(solo, Ordering::Relaxed);
            }
            Ok(json(serde_json::json!({"mining": body.on, "solo": node.solo.load(Ordering::Relaxed)})))
        }
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

