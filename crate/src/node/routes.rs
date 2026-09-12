use std::sync::Mutex;

use super::http::{Error, Request, Response};
use super::{hex, key32, Node};

pub(super) fn route(node: &Mutex<Node>, request: Request) -> Result<Response, Error> {
    let mut node = node
        .lock()
        .map_err(|_| Error::unavailable("node state requires recovery"))?;
    if node.failed || !node.native.healthy() {
        return Err(Error::unavailable("storage requires recovery"));
    }
    let result = route_ready(&mut node, request);
    if result
        .as_ref()
        .is_err_and(|error| error.code == "storage_unavailable")
    {
        node.failed = true;
    }
    result
}

fn route_ready(node: &mut Node, request: Request) -> Result<Response, Error> {
    let path = if request.path == "/" {
        "/"
    } else {
        request.path.trim_end_matches('/')
    };
    match (request.method.as_str(), path) {
        ("GET", "/health") => Ok(Response::text("ok\n")),
        ("GET", "/status") => Ok(Response::text(node.status_cybermark())),
        ("GET", "/root") => Ok(Response::text(format!("{}\n", node.root_hex()))),
        ("GET", "/stats") => {
            let stats = node.native.graph().bbg.statistics();
            Ok(Response::text(format!(
                "---\nparticle: stats\nheight: {}\nbbg-root: {}\nsignals: {}\nneurons: {}\nparticles: {}\naxons: {}\nnode-count: {}\nmax-degree: {}\n---\n",
                node.height(), node.root_hex(), node.signal_count(), node.neuron_count(),
                node.particle_count(), node.axon_count(), stats.node_count, stats.max_degree,
            )))
        }
        ("GET", "/") => Ok(Response::text(format!(
            "soft3 · spacepussy-test · cybergraph+bbg\nmoniker {}\nheight {}\nsignals {}\nGET /status /stats /root\nPOST /v1/link /v1/pay /v1/frame /v2/frame\n",
            node.moniker, node.height(), node.signal_count(),
        ))),
        ("GET", path) if path.starts_with("/balance/") => {
            let neuron = key32(&path["/balance/".len()..]);
            Ok(Response::text(format!(
                "---\nparticle: balance\nneuron: {}\ndenom: {}\nbalance: {}\nheight: {}\nsupply: {}\n---\n",
                hex(&neuron), node.network.denom(), node.balance(&neuron), node.height(), node.pi_supply(),
            )))
        }
        ("GET", "/log") => {
            let from = parameter(&request.query, "from")?.unwrap_or(0);
            let from = usize::try_from(from).map_err(|_| Error::bad("offset exceeds platform limit"))?;
            // This bound exceeds the largest supported legacy frame. The
            // coordinator returns complete frames and validates the cursor.
            let bytes = node.native.wire_log(from, 8 * 1024 * 1024)?;
            Ok(Response::bytes("application/octet-stream", bytes))
        }
        ("GET", "/blocks") => {
            let limit = parameter(&request.query, "limit")?.unwrap_or(50).clamp(1, 500) as usize;
            let before = parameter(&request.query, "before")?;
            let mut text = String::from("---\nparticle: blocks\n---\n");
            for block in node.native.blocks(before, limit)? {
                text.push_str(&format!("{} {} {} {}\n", block.height, block.timestamp, block.supply, block.weight));
            }
            Ok(Response::text(text))
        }
        ("GET", "/v2/history") => {
            let after = parameter(&request.query, "after")?;
            let limit = parameter(&request.query, "limit")?.unwrap_or(16);
            if !(1..=64).contains(&limit) { return Err(Error::bad("history limit must be 1..64")); }
            let rows = node.native.history(after, limit as usize)?;
            let next = rows.last().map(|(receipt, _)| receipt.position);
            let entries: Vec<_> = rows.into_iter().map(|(receipt, operation)| serde_json::json!({
                "receipt": super::requests::receipt_value(&receipt), "operation": hex(&operation),
            })).collect();
            Ok(Response::bytes("application/json", serde_json::json!({
                "schema": "cyber/native-history/v1", "encoding": "CGOP-v1-hex",
                "entries": entries, "next": next,
            }).to_string().into_bytes()))
        }
        ("GET", path) if path.starts_with("/block/") => {
            let height = path["/block/".len()..].parse::<u64>().map_err(|_| Error::bad("invalid height"))?;
            let block = node.native.block(height)?.ok_or_else(|| Error {
                status: "404 Not Found", code: "block_not_found", message: format!("no block at height {height}"),
            })?;
            let mut text = format!(
                "---\nparticle: block\nheight: {}\ntime: {}\nsupply: {}\nweight: {}\nneuron: {}\nbbg-root: {}\n---\n",
                block.height, block.timestamp, block.supply, block.weight, hex(&block.signal.neuron), hex(&block.root),
            );
            for link in &block.signal.links {
                text.push_str(&format!("link {} -> {} amount {} valence {}\n", hex(&link.from), hex(&link.to), link.amount, link.valence));
            }
            for (to, amount) in &block.signal.delta_pi {
                text.push_str(&format!("pay -> {} amount {}\n", hex(to), amount));
            }
            Ok(Response::text(text))
        }
        ("POST", "/v1/link" | "/v1/pay" | "/v1/frame" | "/v2/frame") => {
            super::requests::submit(node, &request, path)
        }
        ("POST", "/v1/finalize") => Err(Error {
            status: "410 Gone", code: "standalone_finalize_removed",
            message: "finalize is part of every accepted signal: one signal, one block".into(),
        }),
        _ => Err(Error { status: "404 Not Found", code: "not_found", message: "not found".into() }),
    }
}

fn parameter(query: &str, name: &str) -> Result<Option<u64>, Error> {
    let mut result = None;
    for parameter in query.split('&') {
        if let Some((key, value)) = parameter.split_once('=') {
            if key == name {
                if result.is_some() {
                    return Err(Error::bad(format!("duplicate {name}")));
                }
                result = Some(
                    value
                        .parse()
                        .map_err(|_| Error::bad(format!("invalid {name}")))?,
                );
            }
        }
    }
    Ok(result)
}
