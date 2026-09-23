use cybergraph::native::{Event, Operation, Receipt};
use foculus::CyberFrame;
use serde::Deserialize;

use super::http::{Error, Request, Response};
use super::{hash32, hex, key32, now_unix, Node};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    neuron: String,
    from: String,
    to: String,
    #[serde(default = "default_token")]
    token: String,
    #[serde(default = "default_amount")]
    amount: u64,
    #[serde(default)]
    valence: i8,
    request_id: Option<String>,
}

fn default_token() -> String {
    "0".into()
}
fn default_amount() -> u64 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pay {
    neuron: String,
    to: String,
    amount: u64,
    request_id: Option<String>,
}

fn identity(header: Option<&str>, body: Option<&str>) -> Result<Option<[u8; 32]>, Error> {
    if header.is_some() && body.is_some() && header != body {
        return Err(Error::bad("Idempotency-Key and request_id must match"));
    }
    let Some(identity) = header.or(body) else {
        return Ok(None);
    };
    if identity.is_empty()
        || identity.len() > 128
        || !identity.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(Error::bad(
            "request identity must contain 1..128 visible ASCII bytes",
        ));
    }
    // Receipts return the raw key as 64 hex digits, allowing server-assigned
    // identities to be retained and reused by clients.
    if identity.len() == 64 && identity.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(Some(key32(identity)));
    }
    let mut input = b"soft3/native-request/v1\0".to_vec();
    input.extend_from_slice(identity.as_bytes());
    Ok(Some(hash32(&input)))
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    fn ok(r: Result<Option<[u8; 32]>, Error>) -> Option<[u8; 32]> {
        match r {
            Ok(v) => v,
            Err(e) => panic!("expected Ok, got error: {}", e.message),
        }
    }

    fn err_message(r: Result<Option<[u8; 32]>, Error>) -> String {
        match r {
            Err(e) => e.message,
            Ok(_) => panic!("expected an error, got Ok"),
        }
    }

    #[test]
    fn absent_header_and_body_yields_no_identity() {
        assert_eq!(ok(identity(None, None)), None);
    }

    #[test]
    fn header_alone_is_accepted() {
        assert!(ok(identity(Some("req-1"), None)).is_some());
    }

    #[test]
    fn body_alone_is_accepted() {
        assert!(ok(identity(None, Some("req-1"))).is_some());
    }

    #[test]
    fn matching_header_and_body_are_accepted() {
        assert!(ok(identity(Some("req-1"), Some("req-1"))).is_some());
    }

    #[test]
    fn mismatched_header_and_body_are_rejected() {
        let message = err_message(identity(Some("req-1"), Some("req-2")));
        assert_eq!(message, "Idempotency-Key and request_id must match");
    }

    #[test]
    fn empty_identity_is_rejected() {
        let message = err_message(identity(Some(""), None));
        assert!(message.contains("1..128 visible ASCII bytes"));
    }

    #[test]
    fn identity_over_128_bytes_is_rejected() {
        let long = "a".repeat(129);
        let message = err_message(identity(Some(&long), None));
        assert!(message.contains("1..128 visible ASCII bytes"));
    }

    #[test]
    fn identity_at_128_bytes_is_accepted() {
        let boundary = "a".repeat(128);
        assert!(ok(identity(Some(&boundary), None)).is_some());
    }

    #[test]
    fn non_ascii_graphic_identity_is_rejected() {
        assert!(err_message(identity(Some("req 1"), None)).contains("1..128 visible ASCII bytes"));
        assert!(err_message(identity(Some("req\t1"), None)).contains("1..128 visible ASCII bytes"));
        assert!(err_message(identity(Some("naïve"), None)).contains("1..128 visible ASCII bytes"));
    }

    #[test]
    fn sixty_four_hex_digits_pass_through_as_the_raw_key() {
        let hex_id = "ab".repeat(32);
        assert_eq!(hex_id.len(), 64);
        let id = ok(identity(Some(&hex_id), None)).unwrap();
        assert_eq!(id, key32(&hex_id));
    }

    #[test]
    fn sixty_four_char_non_hex_identity_is_hashed_not_treated_as_a_key() {
        let non_hex = "z".repeat(64);
        let id = ok(identity(Some(&non_hex), None)).unwrap();
        assert_ne!(id, key32(&non_hex));
        let mut input = b"soft3/native-request/v1\0".to_vec();
        input.extend_from_slice(non_hex.as_bytes());
        assert_eq!(id, hash32(&input));
    }

    #[test]
    fn arbitrary_identity_is_domain_separated_hashed() {
        let raw = "client-chosen-id";
        let id = ok(identity(Some(raw), None)).unwrap();
        let mut input = b"soft3/native-request/v1\0".to_vec();
        input.extend_from_slice(raw.as_bytes());
        assert_eq!(id, hash32(&input));
    }

    #[test]
    fn distinct_identities_hash_to_distinct_keys() {
        let a = ok(identity(Some("client-a"), None)).unwrap();
        let b = ok(identity(Some("client-b"), None)).unwrap();
        assert_ne!(a, b);
    }
}

pub(super) fn submit(node: &mut Node, request: &Request, path: &str) -> Result<Response, Error> {
    let header = request.idempotency_key.as_deref();
    let (request_id, operation, json) = match path {
        "/v1/link" => {
            let body: Link =
                serde_json::from_slice(&request.body).map_err(|e| Error::bad(e.to_string()))?;
            (
                identity(header, body.request_id.as_deref())?,
                Operation::Link {
                    neuron: key32(&body.neuron),
                    from: key32(&body.from),
                    to: key32(&body.to),
                    token: key32(&body.token),
                    amount: body.amount,
                    valence: body.valence,
                },
                true,
            )
        }
        "/v1/pay" => {
            let body: Pay =
                serde_json::from_slice(&request.body).map_err(|e| Error::bad(e.to_string()))?;
            (
                identity(header, body.request_id.as_deref())?,
                Operation::Pay {
                    from: key32(&body.neuron),
                    to: key32(&body.to),
                    amount: body.amount,
                },
                true,
            )
        }
        "/v1/frame" => {
            let frames = foculus::frames::decode_events_strict(&request.body)
                .map_err(|e| Error::bad(e.to_string()))?;
            if frames.is_empty() || frames.len() > 64 {
                return Err(Error::bad("native batch requires 1..64 events"));
            }
            let events = frames
                .into_iter()
                .map(|frame| match frame {
                    CyberFrame::Signal(signal) => Event::Signal(signal),
                    CyberFrame::Intent(intent) => Event::Intent(intent),
                })
                .collect();
            (identity(header, None)?, Operation::Events(events), false)
        }
        "/v2/frame" => {
            let signal = foculus::signal_codec::decode_signal(&request.body)
                .map_err(|e| Error::bad(e.to_string()))?;
            (
                identity(header, None)?,
                Operation::Events(vec![Event::Signal(signal)]),
                false,
            )
        }
        _ => return Err(Error::bad("unsupported submission route")),
    };
    let receipt = node.native.accept(request_id, operation, now_unix())?;
    Ok(if json {
        json_receipt(&receipt)
    } else {
        frame_receipt(&receipt)
    })
}

fn json_receipt(receipt: &Receipt) -> Response {
    let value = receipt_value(receipt);
    Response::bytes("application/json", format!("{value}\n").into_bytes())
}

pub(super) fn receipt_value(receipt: &Receipt) -> serde_json::Value {
    let mut value = serde_json::json!({
        "ok": true, "request_id": hex(&receipt.request_id), "position": receipt.position,
        "height": receipt.height, "root": hex(&receipt.root), "signals": receipt.signals,
        "supply": receipt.supply, "weight": receipt.weight, "timestamp": receipt.timestamp,
        "applied": receipt.applied,
    });
    if let Some(balance) = receipt.balance {
        value["balance"] = balance.into();
    }
    value
}

fn frame_receipt(receipt: &Receipt) -> Response {
    Response::text(format!(
        "---\nparticle: receipt\nrequest-id: {}\nposition: {}\napplied: {}\nrejected: 0\nheight: {}\nbbg-root: {}\nsignals: {}\nsupply: {}\nweight: {}\ntimestamp: {}\n---\n",
        hex(&receipt.request_id), receipt.position, receipt.applied, receipt.height,
        hex(&receipt.root), receipt.signals, receipt.supply, receipt.weight, receipt.timestamp,
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    /// A fresh `Node` in a scratch home directory under the OS temp dir,
    /// uniquely named per call so parallel tests never collide.
    struct TestNode {
        #[allow(dead_code)]
        home: PathBuf,
        node: Node,
    }

    fn test_node() -> TestNode {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::temp_dir().join(format!(
            "soft3-requests-test-{}-{nanos}-{id}",
            std::process::id()
        ));
        let node = Node::open(home.clone(), "test-moniker".into()).expect("node opens");
        TestNode { home, node }
    }

    fn post(body: &[u8]) -> Request {
        Request {
            method: "POST".into(),
            path: String::new(),
            query: String::new(),
            idempotency_key: None,
            body: body.to_vec(),
        }
    }

    fn post_with_key(key: &str, body: &[u8]) -> Request {
        Request {
            method: "POST".into(),
            path: String::new(),
            query: String::new(),
            idempotency_key: Some(key.into()),
            body: body.to_vec(),
        }
    }

    #[test]
    fn link_dispatches_and_returns_a_receipt() {
        let mut t = test_node();
        let request = post(br#"{"neuron":"1","from":"2","to":"3"}"#);
        let response = submit(&mut t.node, &request, "/v1/link").expect("accepted");
        let value = response.json();
        assert_eq!(value["applied"], 1);
        assert_eq!(value["request_id"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn link_defaults_token_and_amount_when_omitted() {
        let mut t = test_node();
        let request = post(br#"{"neuron":"1","from":"2","to":"3"}"#);
        let response = submit(&mut t.node, &request, "/v1/link").expect("accepted");
        // token defaults to "0", amount to 1 (default_token/default_amount);
        // acceptance alone (no deserialize error) proves both defaults filled in.
        assert_eq!(response.json()["applied"], 1);
    }

    #[test]
    fn link_rejects_an_unknown_field() {
        let mut t = test_node();
        let request = post(br#"{"neuron":"1","from":"2","to":"3","bogus":true}"#);
        let error = submit(&mut t.node, &request, "/v1/link").unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
    }

    #[test]
    fn pay_rejects_zero_amount() {
        let mut t = test_node();
        let request = post(br#"{"neuron":"1","to":"2","amount":0}"#);
        let error = submit(&mut t.node, &request, "/v1/pay").unwrap_err();
        assert!(
            error.message.contains("zero payment or insufficient funds"),
            "{}",
            error.message
        );
    }

    #[test]
    fn pay_rejects_an_unfunded_sender() {
        let mut t = test_node();
        let request = post(br#"{"neuron":"1","to":"2","amount":5}"#);
        let error = submit(&mut t.node, &request, "/v1/pay").unwrap_err();
        assert!(
            error.message.contains("zero payment or insufficient funds"),
            "{}",
            error.message
        );
    }

    #[test]
    fn frame_v1_rejects_an_empty_batch() {
        let mut t = test_node();
        let request = post(b"");
        let error = submit(&mut t.node, &request, "/v1/frame").unwrap_err();
        assert!(
            error.message.contains("1..64 events"),
            "{}",
            error.message
        );
    }

    #[test]
    fn frame_v1_rejects_unparseable_bytes() {
        let mut t = test_node();
        let request = post(b"not a tade frame");
        let error = submit(&mut t.node, &request, "/v1/frame").unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
    }

    #[test]
    fn frame_v2_rejects_unparseable_bytes() {
        let mut t = test_node();
        let request = post(b"not a signal");
        let error = submit(&mut t.node, &request, "/v2/frame").unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
    }

    #[test]
    fn unsupported_route_is_rejected() {
        let mut t = test_node();
        let request = post(b"{}");
        let error = submit(&mut t.node, &request, "/v1/nope").unwrap_err();
        assert_eq!(error.message, "unsupported submission route");
    }

    #[test]
    fn idempotency_key_becomes_the_receipts_request_id() {
        let mut t = test_node();
        let request = post_with_key("my-client-key", br#"{"neuron":"1","from":"2","to":"3"}"#);
        let response = submit(&mut t.node, &request, "/v1/link").expect("accepted");
        let expected = identity(Some("my-client-key"), None).unwrap().unwrap();
        assert_eq!(response.json()["request_id"], hex(&expected));
    }

    #[test]
    fn a_64_hex_idempotency_key_passes_through_as_the_request_id() {
        let mut t = test_node();
        let key = "a".repeat(64);
        let request = post_with_key(&key, br#"{"neuron":"1","from":"2","to":"3"}"#);
        let response = submit(&mut t.node, &request, "/v1/link").expect("accepted");
        assert_eq!(response.json()["request_id"], key);
    }
}
