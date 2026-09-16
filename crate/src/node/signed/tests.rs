use super::*;
use neuron_model::{action::ActionRequest, identity::ActionContext};
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "soft3-signed-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn open(&self) -> Node {
        Node::open(self.0.clone(), "signed-test".into()).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn key() -> mudra::SigningKey {
    mudra::SigningKey::from_bytes((&[7; 32]).into()).unwrap()
}
fn subject() -> Particle {
    mudra::claim::neuron_of(&mudra::cosmos::compressed(key().verifying_key()))
}
fn signed(action: ActionRequest) -> SignedAction {
    let proof = mudra::neuron::sign_statement(
        &key(),
        subject(),
        hash32(&statement_bytes(&action).unwrap()),
    )
    .unwrap();
    SignedAction::new(action, proof).unwrap()
}
fn signal(node: &Node) -> SignedAction {
    let subject = subject();
    let signal = cybergraph::Signal {
        neuron: subject,
        network: node.network_id,
        links: vec![cybergraph::CyberlinkRecord {
            neuron: subject,
            from: hash32(b"zheng"),
            to: hash32(b"pussy"),
            token: [0; 32],
            amount: 100,
            valence: 1,
            height: 17,
        }],
        delta_pi: vec![],
        box_moves: vec![],
        step: 0,
        prev: [0; 32],
        height: 17,
        proof: Some(
            foculus::prove_pay(&foculus::PayStatement {
                content_id: [11; 32],
                total_out: 1,
                leg_count: 1,
            })
            .unwrap(),
        ),
    };
    signed(ActionRequest {
        request: [1; 32],
        attachment: [2; 32],
        context: ActionContext {
            subject: SubjectRef::Native(subject),
            network: NetworkRef::Native(node.network_id),
            binding_revision: 0,
            prog: None,
            invocation: None,
            policy: [3; 32],
            grant: [4; 32],
        },
        kind: "native/signal".into(),
        payload: foculus::signal_codec::encode_signal(&signal).unwrap(),
    })
}
fn post(envelope: &SignedAction) -> Request {
    Request {
        method: "POST".into(),
        path: "/v3/action".into(),
        query: String::new(),
        idempotency_key: Some(hex(&envelope.action.request)),
        body: envelope.encode().unwrap(),
    }
}
fn route(node: &Mutex<Node>, method: &str, path: &str, body: Vec<u8>) -> Result<Response, Error> {
    super::super::routes::route(
        node,
        Request {
            method: method.into(),
            path: path.into(),
            query: String::new(),
            idempotency_key: None,
            body,
        },
    )
}
#[test]
fn activation_blocks_old_writers_and_survives_restart() {
    let dir = Directory::new();
    let mut node = dir.open();
    assert_eq!(capabilities(&node).unwrap().json()["authenticated"], false);
    let envelope = signal(&node);
    assert_eq!(
        submit(&mut node, &post(&envelope)).unwrap_err().status,
        "401 Unauthorized"
    );
    let network = node.enable_authentication().unwrap();
    drop(node);
    let node = Mutex::new(dir.open());
    let caps = route(&node, "GET", "/capabilities", vec![]).unwrap().json();
    assert_eq!(caps["profile"], PROFILE);
    assert_eq!(caps["network"], hex(&network));
    for path in ["/v1/link", "/v1/pay", "/v1/frame", "/v2/frame"] {
        assert_eq!(
            route(&node, "POST", path, vec![]).unwrap_err().status,
            "401 Unauthorized"
        );
    }
    assert_eq!(node.lock().unwrap().height(), 0);
}
#[test]
fn interrupted_activation_retires_genesis_before_readiness_and_missing_target_fails() {
    let dir = Directory::new();
    let node = dir.open();
    let original = std::fs::read(dir.0.join("genesis.json")).unwrap();
    node.native.database().require_reader_generation(ReaderGeneration::AuthenticatedV1).unwrap();
    drop(node); // Crash between BBG promotion and filesystem retirement.
    let node = dir.open();
    assert!(node.authenticated().unwrap());
    assert_eq!(std::fs::read(dir.0.join("genesis.json/source")).unwrap(), original);
    drop(node);
    std::fs::rename(dir.0.join("bbg"), dir.0.join("retained-bbg")).unwrap();
    assert!(Node::open(dir.0.clone(), "missing-target".into()).is_err());
    assert!(!dir.0.join("bbg").exists());
}
#[test]
fn exact_signal_and_payment_receipts_survive_lost_response_and_restart() {
    let dir = Directory::new();
    let mut node = dir.open();
    node.enable_authentication().unwrap();
    let envelope = signal(&node);
    let receipt = submit(&mut node, &post(&envelope)).unwrap().json();
    assert_eq!(receipt["state"], "accepted");
    assert_eq!(node.balance(&subject()), 100);
    assert_eq!(
        foculus::signal_codec::encode_signal(&node.native.block(1).unwrap().unwrap().signal)
            .unwrap(),
        envelope.action.payload
    );
    assert_eq!(submit(&mut node, &post(&envelope)).unwrap().json(), receipt);
    let mut pay = envelope.action.clone();
    pay.request = [9; 32];
    pay.kind = "native/pay".into();
    pay.payload = serde_json::to_vec(&Pay {
        to: [5; 32],
        amount: 30,
    })
    .unwrap();
    let payment = signed(pay.clone());
    let paid = submit(&mut node, &post(&payment)).unwrap().json();
    assert_eq!(
        (
            node.balance(&subject()),
            node.balance(&[5; 32]),
            node.height()
        ),
        (70, 30, 2)
    );
    pay.payload = serde_json::to_vec(&Pay {
        to: [5; 32],
        amount: 31,
    })
    .unwrap();
    assert_eq!(
        submit(&mut node, &post(&signed(pay))).unwrap_err().status,
        "409 Conflict"
    );
    drop(node);
    let mut node = dir.open();
    assert_eq!(submit(&mut node, &post(&payment)).unwrap().json(), paid);
    let path = format!(
        "/v3/receipt/{}/{}",
        hex(&subject()),
        hex(&payment.action.request)
    );
    assert_eq!(lookup(&node, &path).unwrap().json(), paid);
    assert_eq!(node.height(), 2);
    let page=history(&node,None,4).unwrap().json();
    assert_eq!(page["network"],hex(&node.network_id));
    assert_eq!(page["next"],2);
    assert_eq!(page["entries"][0]["signals"][0],hex(&envelope.action.payload));
    let payment_signal=node.native.block(2).unwrap().unwrap().signal;
    assert_eq!(page["entries"][1]["signals"][0],hex(&foculus::signal_codec::encode_signal(&payment_signal).unwrap()));
    assert!(history(&node,Some(2),4).unwrap().json()["entries"].as_array().unwrap().is_empty());
    assert_eq!(
        lookup(
            &node,
            &format!(
                "/v3/receipt/{}/{}",
                hex(&[6; 32]),
                hex(&payment.action.request)
            )
        )
        .unwrap_err()
        .status,
        "404 Not Found"
    );
}
#[test]
fn prepared_journal_recovers_without_a_second_native_acceptance() {
    let dir = Directory::new();
    let node = dir.open();
    node.enable_authentication().unwrap();
    let envelope = signal(&node);
    let (request, _) = operation(&node, &envelope).unwrap();
    pin(&node, request, &envelope.encode().unwrap()).unwrap();
    let path = format!(
        "/v3/receipt/{}/{}",
        hex(&subject()),
        hex(&envelope.action.request)
    );
    drop(node);
    let mut node = dir.open();
    assert_eq!(lookup(&node, &path).unwrap().json()["state"], "prepared");
    let accepted = submit(&mut node, &post(&envelope)).unwrap().json();
    assert_eq!(lookup(&node, &path).unwrap().json(), accepted);
    assert_eq!(node.height(), 1);
}
#[test]
fn signed_signal_never_acknowledges_a_skipped_payment_leg() {
    let dir=Directory::new();let mut node=dir.open();node.enable_authentication().unwrap();
    let mut action=signal(&node).action;
    let mut s=foculus::signal_codec::decode_signal(&action.payload).unwrap();
    s.links.clear();s.delta_pi.push(([9;32],1));
    action.payload=foculus::signal_codec::encode_signal(&s).unwrap();
    let envelope=signed(action);
    assert_eq!(submit(&mut node,&post(&envelope)).unwrap_err().status,"400 Bad Request");
    assert_eq!(node.height(),0);
    assert_eq!(node.balance(&[9;32]),0);
    let path=format!("/v3/receipt/{}/{}",hex(&subject()),hex(&envelope.action.request));
    assert_eq!(lookup(&node,&path).unwrap().json()["state"],"prepared");
}
#[test]
fn subject_network_payload_signature_and_canonical_encoding_are_enforced() {
    let dir = Directory::new();
    let mut node = dir.open();
    node.enable_authentication().unwrap();
    let envelope = signal(&node);
    let mut wrong = envelope.clone();
    wrong.action.context.network = NetworkRef::Native([99; 32]);
    assert_eq!(
        submit(&mut node, &post(&signed(wrong.action)))
            .unwrap_err()
            .status,
        "400 Bad Request"
    );
    let mut wrong = envelope.clone();
    wrong.action.context.subject = SubjectRef::Native([99; 32]);
    assert_eq!(
        submit(&mut node, &post(&wrong)).unwrap_err().status,
        "401 Unauthorized"
    );
    let mut wrong = envelope.clone();
    wrong.evidence[90] ^= 1;
    assert_eq!(
        submit(&mut node, &post(&wrong)).unwrap_err().status,
        "401 Unauthorized"
    );
    let mut wrong = envelope.action.clone();
    let mut s = foculus::signal_codec::decode_signal(&wrong.payload).unwrap();
    s.network = [88; 32];
    wrong.payload = foculus::signal_codec::encode_signal(&s).unwrap();
    assert_eq!(
        submit(&mut node, &post(&signed(wrong))).unwrap_err().status,
        "400 Bad Request"
    );
    let mut wrong = post(&envelope);
    wrong.idempotency_key = Some(hex(&[2; 32]));
    assert_eq!(
        submit(&mut node, &wrong).unwrap_err().status,
        "400 Bad Request"
    );
    let mut wrong = post(&envelope);
    wrong.body.push(b' ');
    assert_eq!(
        submit(&mut node, &wrong).unwrap_err().status,
        "400 Bad Request"
    );
    let mut wrong = envelope.action;
    wrong.kind = "native/pay".into();
    wrong.payload =
        serde_json::to_vec(&serde_json::json!({"to":vec![5;32],"amount":1})).unwrap();
    assert_eq!(
        submit(&mut node, &post(&signed(wrong))).unwrap_err().status,
        "400 Bad Request"
    );
    assert_eq!(node.height(), 0);
    assert!(node.native.history(None, 64).unwrap().is_empty());
}
