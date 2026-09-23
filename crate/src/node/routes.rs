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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    /// A fresh `Node` in a scratch home directory under the OS temp dir,
    /// uniquely named per call so parallel tests never collide.
    struct TestNode {
        node: Node,
        _home: TestHome,
    }

    struct TestHome(PathBuf);

    impl Drop for TestHome {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn test_node() -> TestNode {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::temp_dir().join(format!(
            "soft3-routes-test-{}-{nanos}-{id}",
            std::process::id()
        ));
        let home = TestHome(home);
        let node = Node::open(home.0.clone(), "test-moniker".into()).expect("node opens");
        TestNode { node, _home: home }
    }

    fn get(path: &str, query: &str) -> Request {
        Request {
            method: "GET".into(),
            path: path.into(),
            query: query.into(),
            idempotency_key: None,
            body: Vec::new(),
        }
    }

    fn post(path: &str, body: &[u8]) -> Request {
        Request {
            method: "POST".into(),
            path: path.into(),
            query: String::new(),
            idempotency_key: None,
            body: body.to_vec(),
        }
    }

    fn text(body: &[u8]) -> &str {
        std::str::from_utf8(body).expect("utf-8 body")
    }

    #[test]
    fn health_returns_ok() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/health", "")).expect("ok");
        let (status, content_type, body) = response.parts();
        assert_eq!(status, "200 OK");
        assert_eq!(content_type, "text/plain; charset=utf-8");
        assert_eq!(body, b"ok\n");
    }

    #[test]
    fn status_reports_chain_and_moniker() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/status", "")).expect("ok");
        let (_, _, body) = response.parts();
        let body = text(body);
        assert!(body.contains("chain: spacepussy-test"), "{body}");
        assert!(body.contains("moniker: test-moniker"), "{body}");
        assert!(body.contains("denom: testpussy"), "{body}");
    }

    #[test]
    fn root_matches_node_root_hex() {
        let mut t = test_node();
        let expected = format!("{}\n", t.node.root_hex());
        let response = route_ready(&mut t.node, get("/root", "")).expect("ok");
        let (_, _, body) = response.parts();
        assert_eq!(text(body), expected);
    }

    #[test]
    fn stats_reports_particle_stats() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/stats", "")).expect("ok");
        let (_, _, body) = response.parts();
        assert!(text(body).contains("particle: stats"));
    }

    #[test]
    fn index_lists_moniker_and_routes() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/", "")).expect("ok");
        let (_, _, body) = response.parts();
        let body = text(body);
        assert!(body.contains("moniker test-moniker"), "{body}");
        assert!(body.contains("POST /v1/link"), "{body}");
    }

    #[test]
    fn trailing_slashes_are_trimmed_before_dispatch() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/health///", "")).expect("ok");
        let (_, _, body) = response.parts();
        assert_eq!(body, b"ok\n");
    }

    #[test]
    fn balance_reports_for_hex_neuron() {
        let mut t = test_node();
        let neuron = "0".repeat(64);
        let response =
            route_ready(&mut t.node, get(&format!("/balance/{neuron}"), "")).expect("ok");
        let (_, _, body) = response.parts();
        let body = text(body);
        assert!(body.contains("particle: balance"), "{body}");
        assert!(body.contains(&format!("neuron: {neuron}")), "{body}");
    }

    #[test]
    fn log_defaults_to_offset_zero() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/log", "")).expect("ok");
        let (status, content_type, _) = response.parts();
        assert_eq!(status, "200 OK");
        assert_eq!(content_type, "application/octet-stream");
    }

    #[test]
    fn blocks_limit_is_clamped_not_rejected() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/blocks", "limit=99999")).expect("ok");
        assert_eq!(response.parts().0, "200 OK");
        let response = route_ready(&mut t.node, get("/blocks", "limit=0")).expect("ok");
        assert_eq!(response.parts().0, "200 OK");
    }

    #[test]
    fn history_limit_out_of_range_is_rejected() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, get("/v2/history", "limit=65")).unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
        assert!(error.message.contains("1..64"), "{}", error.message);
    }

    #[test]
    fn history_limit_in_range_is_accepted() {
        let mut t = test_node();
        let response = route_ready(&mut t.node, get("/v2/history", "limit=1")).expect("ok");
        let (status, content_type, body) = response.parts();
        assert_eq!(status, "200 OK");
        assert_eq!(content_type, "application/json");
        assert!(text(body).contains("cyber/native-history/v1"));
    }

    #[test]
    fn block_not_found_is_404() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, get("/block/999999", "")).unwrap_err();
        assert_eq!(error.status, "404 Not Found");
        assert_eq!(error.code, "block_not_found");
    }

    #[test]
    fn block_invalid_height_is_400() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, get("/block/not-a-number", "")).unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
    }

    #[test]
    fn unknown_get_path_is_404() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, get("/nope", "")).unwrap_err();
        assert_eq!(error.status, "404 Not Found");
        assert_eq!(error.code, "not_found");
    }

    #[test]
    fn post_finalize_is_gone() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, post("/v1/finalize", b"{}")).unwrap_err();
        assert_eq!(error.status, "410 Gone");
        assert_eq!(error.code, "standalone_finalize_removed");
    }

    #[test]
    fn post_link_dispatches_into_submit() {
        let mut t = test_node();
        // Malformed JSON never reaches a valid signal; a rejection here still
        // proves dispatch reached `requests::submit` rather than falling
        // through to the unmatched-route 404 below.
        let error = route_ready(&mut t.node, post("/v1/link", b"not json")).unwrap_err();
        assert_ne!(error.code, "not_found");
    }

    #[test]
    fn unknown_post_path_is_404() {
        let mut t = test_node();
        let error = route_ready(&mut t.node, post("/nope", b"{}")).unwrap_err();
        assert_eq!(error.status, "404 Not Found");
        assert_eq!(error.code, "not_found");
    }

    #[test]
    fn route_locks_and_delegates_to_route_ready() {
        let t = test_node();
        let lock = Mutex::new(t.node);
        let response = route(&lock, get("/health", "")).expect("ok");
        assert_eq!(response.parts().0, "200 OK");
    }
}

#[cfg(test)]
mod parameter_tests {
    use super::*;

    fn ok(r: Result<Option<u64>, Error>) -> Option<u64> {
        match r {
            Ok(v) => v,
            Err(e) => panic!("expected Ok, got error: {}", e.message),
        }
    }

    fn err_message(r: Result<Option<u64>, Error>) -> String {
        match r {
            Err(e) => e.message,
            Ok(v) => panic!("expected an error, got Ok({v:?})"),
        }
    }

    #[test]
    fn empty_query_yields_none() {
        assert_eq!(ok(parameter("", "limit")), None);
    }

    #[test]
    fn absent_key_yields_none() {
        assert_eq!(ok(parameter("from=1&before=2", "limit")), None);
    }

    #[test]
    fn present_key_is_parsed() {
        assert_eq!(ok(parameter("limit=50", "limit")), Some(50));
    }

    #[test]
    fn key_among_others_is_found() {
        assert_eq!(ok(parameter("from=1&limit=50&before=2", "limit")), Some(50));
    }

    #[test]
    fn duplicate_key_is_rejected() {
        let message = err_message(parameter("limit=1&limit=2", "limit"));
        assert_eq!(message, "duplicate limit");
    }

    #[test]
    fn non_numeric_value_is_rejected() {
        let message = err_message(parameter("limit=abc", "limit"));
        assert_eq!(message, "invalid limit");
    }

    #[test]
    fn empty_value_is_rejected() {
        let message = err_message(parameter("limit=", "limit"));
        assert_eq!(message, "invalid limit");
    }

    #[test]
    fn negative_value_is_rejected() {
        let message = err_message(parameter("limit=-1", "limit"));
        assert_eq!(message, "invalid limit");
    }

    #[test]
    fn key_without_equals_is_ignored() {
        assert_eq!(ok(parameter("flag&limit=7", "limit")), Some(7));
    }

    #[test]
    fn unrelated_key_with_same_prefix_does_not_match() {
        assert_eq!(ok(parameter("limits=99", "limit")), None);
    }

    #[test]
    fn zero_is_a_valid_value() {
        assert_eq!(ok(parameter("from=0", "from")), Some(0));
    }
}
