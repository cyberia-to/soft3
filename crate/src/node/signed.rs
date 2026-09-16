//! Authenticated local native adapter; receipts are endpoint acceptance.
use super::{
    Node, hash32, hex,
    http::{Error, Request, Response},
    now_unix,
};
use cybergraph::{
    Particle,
    application::{ApplicationGraph, Head, Proposal, ReaderGeneration},
    content::{Codec, Content},
    native::{Event, Operation, Receipt},
};
use neuron_model::{
    action::{SignedAction, statement_bytes},
    identity::{NetworkRef, SubjectRef},
};
use serde::{Deserialize, Serialize};
pub const PROFILE: &str = "neuron/signed-native/1";
#[cfg(test)]
mod tests;
fn bad(e: impl std::fmt::Display) -> Error {
    Error::bad(e.to_string())
}
fn storage(e: impl std::fmt::Debug) -> Error {
    Error::unavailable(format!("action journal: {e:?}"))
}
fn unauthorized(message: &str) -> Error {
    Error {
        status: "401 Unauthorized",
        code: "authentication_required",
        message: message.into(),
    }
}
fn conflict() -> Error {
    Error {
        status: "409 Conflict",
        code: "action_conflict",
        message: "request already names a different signed action".into(),
    }
}
fn hash_json(value: &impl Serialize) -> Result<Particle, Error> {
    Ok(hash32(&serde_json::to_vec(value).map_err(bad)?))
}
fn request_key(network: Particle, subject: Particle, request: Particle) -> Result<Particle, Error> {
    hash_json(&("neuron/native-request/1", network, subject, request))
}
fn journal_key(request: Particle) -> Result<Particle, Error> {
    hash_json(&("neuron/native-journal/1", request))
}
fn parse_id(value: &str) -> Result<Particle, Error> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad("expected 64 hex characters"));
    }
    let mut bytes = [0; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).map_err(bad)?;
    }
    Ok(bytes)
}
impl Node {
    pub(super) fn authenticated(&self) -> Result<bool, Error> {
        Ok(self
            .native
            .database()
            .reader_generation()
            .map_err(storage)?
            >= ReaderGeneration::AuthenticatedV1)
    }
    /// Explicit offline/local host upgrade. Raw genesis and old native history stay unchanged.
    pub fn enable_authentication(&self) -> std::io::Result<Particle> {
        self.native
            .database()
            .require_reader_generation(ReaderGeneration::AuthenticatedV1)
            .map_err(std::io::Error::other)?;
        super::genesis::retire(&self.home)?;
        Ok(self.network_id)
    }
    pub fn network_id(&self) -> Particle {
        self.network_id
    }
}
pub(super) fn capabilities(node: &Node) -> Result<Response, Error> {
    let enabled = node.authenticated()?;
    Ok(Response::bytes("application/json",serde_json::to_vec(&serde_json::json!({
        "schema":"neuron/native-capabilities/1","profile":if enabled{PROFILE}else{"soft3/unsigned-local/1"},
        "network":hex(&node.network_id),"authenticated":enabled,
        "kinds":if enabled{vec!["native/signal","native/pay"]}else{vec![]},
        "max_envelope_bytes":neuron_model::action::MAX_ENVELOPE_BYTES,
        "idempotency":"network-subject-request/1","receipt":"/v3/receipt/{subject}/{request}"
    })).map_err(bad)?))
}
pub(super) fn history(node: &Node, after: Option<u64>, limit: usize) -> Result<Response, Error> {
    if !node.authenticated()? { return Err(unauthorized("signed native profile is inactive")); }
    if !(1..=16).contains(&limit) {return Err(bad("history limit must be 1..16"));}
    let mut entries=Vec::new();
    let mut total=1024usize;
    for entry in node.native.history_view(after,limit)? {
        let signals=entry.signals.iter().map(|s|foculus::signal_codec::encode_signal(s).map(|b|hex(&b)).map_err(bad)).collect::<Result<Vec<_>,_>>()?;
        let value=serde_json::json!({"receipt":super::requests::receipt_value(&entry.receipt),"operation":hex(&entry.operation),"signals":signals});
        let size=serde_json::to_vec(&value).map_err(bad)?.len();
        if total+size>32*1024*1024 { if entries.is_empty(){return Err(bad("native history entry exceeds 32 MiB"));} break; }
        total+=size;entries.push(value);
    }
    let next=entries.last().and_then(|v|v["receipt"]["position"].as_u64());
    Ok(Response::bytes("application/json",serde_json::to_vec(&serde_json::json!({
        "schema":"neuron/native-history/1","profile":PROFILE,"network":hex(&node.network_id),
        "after":after,"entries":entries,"next":next,
    })).map_err(bad)?))
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pay {
    to: Particle,
    amount: u64,
}
fn operation(node: &Node, envelope: &SignedAction) -> Result<(Particle, Operation), Error> {
    envelope.validate().map_err(bad)?;
    let SubjectRef::Native(subject) = envelope.action.context.subject else {
        return Err(bad("unsupported subject profile"));
    };
    if envelope.action.context.network != NetworkRef::Native(node.network_id) {
        return Err(bad("wrong destination network"));
    }
    let statement = hash32(&statement_bytes(&envelope.action).map_err(bad)?);
    if !mudra::neuron::verify_statement(subject, statement, &envelope.evidence) {
        return Err(unauthorized("invalid native action signature"));
    }
    let operation = match envelope.action.kind.as_str() {
        "native/signal" => {
            let signal =
                foculus::signal_codec::decode_signal(&envelope.action.payload).map_err(bad)?;
            if signal.neuron != subject || signal.network != node.network_id {
                return Err(bad("signal author/network differs from signed context"));
            }
            Operation::Events(vec![Event::Signal(signal)])
        }
        "native/pay" => {
            let pay: Pay = serde_json::from_slice(&envelope.action.payload)
                .map_err(|_| bad("invalid pay payload"))?;
            if pay.amount == 0 || serde_json::to_vec(&pay).map_err(bad)? != envelope.action.payload
            {
                return Err(bad("pay payload must be canonical and positive"));
            }
            Operation::Pay {
                from: subject,
                to: pay.to,
                amount: pay.amount,
            }
        }
        _ => return Err(bad("unsupported signed action kind")),
    };
    Ok((
        request_key(node.network_id, subject, envelope.action.request)?,
        operation,
    ))
}
fn pin(node: &Node, request: Particle, bytes: &[u8]) -> Result<Particle, Error> {
    let graph = ApplicationGraph::from_database(node.native.database());
    let namespace = journal_key(request)?;
    let content = Content::new(Codec::Blob, bytes.to_vec()).map_err(storage)?;
    let head = Head {
        index: 0,
        commit: content.id(),
    };
    if let Some(stored) = graph.head(&namespace).map_err(storage)? {
        if stored != head {
            return Err(conflict());
        }
        if graph.resolve(&namespace, &request).map_err(storage)? != Some(head) {
            return Err(storage("missing journal receipt"));
        }
        let stored = graph
            .get(&head.commit)
            .map_err(storage)?
            .ok_or_else(|| storage("missing journal content"))?;
        if stored.codec() != Codec::Blob || stored.bytes() != bytes {
            return Err(storage("journal content differs"));
        }
        return Ok(head.commit);
    }
    graph
        .commit(
            &Proposal {
                namespace,
                request,
                expected: None,
                head,
                content: vec![content],
                required: vec![],
                claims: vec![],
            },
            |_| Ok(()),
        )
        .map_err(storage)?;
    Ok(head.commit)
}
fn response(
    node: &Node,
    envelope: &SignedAction,
    id: Particle,
    native_request: Particle,
    receipt: Option<&Receipt>,
) -> Result<Response, Error> {
    let subject = envelope
        .action
        .context
        .subject
        .native()
        .ok_or_else(|| bad("unsupported subject"))?;
    Ok(Response::bytes("application/json",serde_json::to_vec(&serde_json::json!({
        "schema":"neuron/native-receipt/1","profile":PROFILE,"network":hex(&node.network_id),
        "subject":hex(&subject),"request":hex(&envelope.action.request),"native_request":hex(&native_request),
        "envelope":hex(&id),"state":if receipt.is_some(){"accepted"}else{"prepared"},
        "receipt":receipt.map(super::requests::receipt_value)
    })).map_err(bad)?))
}
pub(super) fn submit(node: &mut Node, request: &Request) -> Result<Response, Error> {
    if !node.authenticated()? {
        return Err(unauthorized(
            "signed native profile requires explicit host activation",
        ));
    }
    let envelope = SignedAction::decode(&request.body).map_err(bad)?;
    if request
        .idempotency_key
        .as_ref()
        .is_some_and(|h| h != &hex(&envelope.action.request))
    {
        return Err(bad("Idempotency-Key differs from signed action request"));
    }
    let (native_request, operation) = operation(node, &envelope)?;
    let id = pin(node, native_request, &request.body)?;
    if let Some(receipt)=node.native.resolve(native_request,&operation)? {
        return response(node,&envelope,id,native_request,Some(&receipt));
    }
    node.native.validate_payments(&operation)?;
    let receipt = node
        .native
        .accept(Some(native_request), operation, now_unix())?;
    response(node, &envelope, id, native_request, Some(&receipt))
}
pub(super) fn lookup(node: &Node, path: &str) -> Result<Response, Error> {
    if !node.authenticated()? {
        return Err(unauthorized("signed native profile is inactive"));
    }
    let parts: Vec<_> = path.trim_start_matches("/v3/receipt/").split('/').collect();
    let [subject, request] = parts.as_slice() else {
        return Err(bad("receipt path requires subject/request"));
    };
    let (subject, request) = (parse_id(subject)?, parse_id(request)?);
    let native_request = request_key(node.network_id, subject, request)?;
    let graph = ApplicationGraph::from_database(node.native.database());
    let namespace = journal_key(native_request)?;
    let head = graph
        .head(&namespace)
        .map_err(storage)?
        .ok_or_else(|| Error {
            status: "404 Not Found",
            code: "unknown_action",
            message: "no matching action journal".into(),
        })?;
    if head.index != 0
        || graph
            .resolve(&namespace, &native_request)
            .map_err(storage)?
            != Some(head)
    {
        return Err(storage("invalid action journal head/receipt"));
    }
    let content = graph
        .get(&head.commit)
        .map_err(storage)?
        .ok_or_else(|| storage("missing action journal"))?;
    if content.codec() != Codec::Blob {
        return Err(storage("invalid action journal codec"));
    }
    let envelope = SignedAction::decode(content.bytes()).map_err(storage)?;
    let (derived, operation) = operation(node, &envelope)?;
    if derived != native_request
        || envelope.action.context.subject.native() != Some(subject)
        || envelope.action.request != request
    {
        return Err(storage("action journal context mismatch"));
    }
    let receipt = node.native.resolve(native_request, &operation)?;
    response(
        node,
        &envelope,
        head.commit,
        native_request,
        receipt.as_ref(),
    )
}
