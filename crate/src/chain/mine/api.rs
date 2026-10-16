//! The node's mining API (chain-wire.md §7): `GET /template?to={neuron}`,
//! `POST /submit`, `GET /tip` — the JSON shapes and a blocking client with
//! retry.

use super::target::{from_bytes, Words};
use crate::chain::params::HEADER_BYTES;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `GET /template?to={neuron}` (chain-wire §7). Hex is lowercase, bytes in
/// wire order; `target` is the 32 little-endian bytes `decode(bits)`
/// (redundant with the header's `bits`, so an external worker needs no
/// compact-bits decoder); `expires` is the unix second after which the
/// node will answer `409` to a submit of this template (the template also
/// expires the moment the tip moves — poll `GET /tip`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub chain: String,
    pub height: u64,
    pub prev: String,
    pub header: String,
    pub bits: String,
    pub target: String,
    pub body: String,
    pub expires: u64,
}

/// `POST /submit` body: the solved header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submit {
    pub header: String,
}

/// `POST /submit` → `200`: the accepted block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Accepted {
    pub id: String,
    pub height: u64,
}

/// `GET /tip`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tip {
    pub height: u64,
    pub id: String,
    pub chainwork: String,
}

/// What a submit came back with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted(Accepted),
    /// `409`: the template's tip is gone — refresh.
    Stale,
    /// `400`/`422`: the node rejected the header, with its reason.
    Rejected(String),
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

impl Template {
    pub fn header_bytes(&self) -> Result<[u8; HEADER_BYTES], String> {
        unhex(&self.header)?.try_into().map_err(|v: Vec<u8>| {
            format!("template header is {} bytes, not {HEADER_BYTES}", v.len())
        })
    }

    pub fn target_words(&self) -> Result<Words, String> {
        let b: [u8; 32] = unhex(&self.target)?
            .try_into()
            .map_err(|v: Vec<u8>| format!("template target is {} bytes, not 32", v.len()))?;
        Ok(from_bytes(&b))
    }

    pub fn bits_u32(&self) -> Result<u32, String> {
        u32::from_str_radix(self.bits.trim_start_matches("0x"), 16)
            .map_err(|e| format!("template bits: {e}"))
    }
}

fn json<T: serde::de::DeserializeOwned>(r: ureq::Response, what: &str) -> Result<T, String> {
    let text = r.into_string().map_err(|e| format!("{what}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{what}: {e}"))
}

/// A blocking client of one node.
#[derive(Clone, Debug)]
pub struct Client {
    pub node: String,
    pub address: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(node: &str, address: &str) -> Self {
        Client {
            node: node.trim_end_matches('/').to_string(),
            address: address.to_string(),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout(Duration::from_secs(15))
                .build(),
        }
    }

    pub fn template(&self) -> Result<Template, String> {
        let url = format!("{}/template?to={}", self.node, self.address);
        let r = self
            .agent
            .get(&url)
            .call()
            .map_err(|e| format!("GET /template: {e}"))?;
        json(r, "GET /template")
    }

    pub fn tip(&self) -> Result<Tip, String> {
        let url = format!("{}/tip", self.node);
        let r = self
            .agent
            .get(&url)
            .call()
            .map_err(|e| format!("GET /tip: {e}"))?;
        json(r, "GET /tip")
    }

    pub fn submit(&self, header: &[u8; HEADER_BYTES]) -> Result<Verdict, String> {
        let url = format!("{}/submit", self.node);
        let body = serde_json::to_string(&Submit {
            header: hex(header),
        })
        .expect("json");
        match self
            .agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(r) => Ok(Verdict::Accepted(json(r, "POST /submit")?)),
            Err(ureq::Error::Status(409, _)) => Ok(Verdict::Stale),
            Err(ureq::Error::Status(code @ (400 | 422), r)) => {
                let reason = r.into_string().unwrap_or_default();
                Ok(Verdict::Rejected(format!("{code}: {}", reason.trim())))
            }
            Err(e) => Err(format!("POST /submit: {e}")),
        }
    }

    /// `f` until it succeeds, with backoff 1 s, 2 s, … capped at `cap`,
    /// at most `tries` attempts (0: forever); the last error otherwise.
    pub fn retry<T>(
        mut f: impl FnMut() -> Result<T, String>,
        tries: u32,
        cap: Duration,
        log: impl Fn(&str),
    ) -> Result<T, String> {
        let mut wait = Duration::from_secs(1);
        let mut n = 0;
        loop {
            match f() {
                Ok(v) => return Ok(v),
                Err(e) => {
                    n += 1;
                    if tries != 0 && n >= tries {
                        return Err(e);
                    }
                    log(&format!("{e}; retry in {}s", wait.as_secs()));
                    std::thread::sleep(wait);
                    wait = (wait * 2).min(cap);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_json_round_trip() {
        let t = Template {
            chain: "pussy".into(),
            height: 7,
            prev: "11".repeat(32),
            header: "00".repeat(168),
            bits: "1e010000".into(),
            target: hex(&crate::chain::mine::target::to_bytes(
                &crate::chain::mine::target::pow2(232),
            )),
            body: "0100000001000700".into(),
            expires: 1_793_885_850,
        };
        let s = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Template>(&s).unwrap(), t);
        assert_eq!(t.bits_u32().unwrap(), 0x1e01_0000);
        assert_eq!(
            t.target_words().unwrap(),
            crate::chain::mine::target::pow2(232)
        );
        assert_eq!(t.header_bytes().unwrap(), [0u8; 168]);
        assert!(
            serde_json::from_str::<Template>(&s.replace("\"chain\"", "\"extra\":1,\"chain\""))
                .is_err()
        );
    }

    #[test]
    fn retry_gives_up_after_tries() {
        let mut calls = 0;
        let r: Result<(), String> = Client::retry(
            || {
                calls += 1;
                Err("no".into())
            },
            2,
            Duration::from_millis(1),
            |_| {},
        );
        assert_eq!(r, Err("no".into()));
        assert_eq!(calls, 2);
    }
}
