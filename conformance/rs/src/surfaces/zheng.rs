//! zheng — proof production and verification over committed fixtures.
//!
//! The surface is zheng 0.4's production wire format `ZHENGPF1`
//! (`zheng::envelope`): one envelope with a profile byte. Each scenario is a
//! small nox formula proven under
//!
//! - profile 0, the public certificate (`zheng::execution::certify_execution`);
//! - profile 1, the succinct proof (`zheng::execution::succinct::prove_default`,
//!   WHIR, the shipped device parameters).
//!
//! Only profiles whose proving is deterministic are fingerprinted: profile 0
//! is a function of the witness; profile 1 draws every challenge from the
//! Fiat-Shamir transcript and its proof-of-work grinding returns the least
//! passing nonce. Profile 2 (zk) draws OS entropy (its seeded mode is behind
//! zheng's `test-hooks`, tests only), profile 3 (state) needs authenticated
//! state evidence that these programs do not read, and profiles 4-6 (machine,
//! recursive, wrapped) are too slow for the release gate.
//!
//! zheng 0.3's folded trace API (`commit`, `verify`, `open`, `verify_eval`,
//! `TraceProof`, `ProofParams`, `Statement`) is not fingerprinted: zheng
//! compiles it only under its `legacy` feature, documents it as unsound, and
//! says never to enable it on a production path, so this crate does not.
//!
//! Each scenario and profile has a committed fixture under
//! `conformance/fixtures/zheng/<scenario>.p<profile>.zheng` -- the envelope's
//! wire bytes, written by `--bless`. `--check` verifies the committed bytes
//! with the current verifier against the scenario's program (rejection is a
//! hard failure), verifies a fixed set of single-bit tamperings (their
//! verdicts are snapshotted), and re-proves the scenario to snapshot the
//! prover's current output.

use zheng::envelope::{Envelope, Profile, SuccinctStatement};
use zheng::execution::{ExecutionNoun, ExecutionStatement, NounToken};

use super::{Bytes, Context, Fixtures, Out};
use crate::text::{Data, parse_data};

/// Budget of every scenario (the statement binds it).
const BUDGET: u64 = 1_000_000;

/// The envelope profiles this surface proves, in snapshot order.
const PROFILES: [u8; 2] = [0, 1];

/// A scenario: a nox formula run on the public inputs. These programs read no
/// subject, so they take no inputs.
struct Scenario {
    name: &'static str,
    formula: &'static str,
    input: &'static [u64],
}

/// Programs without axis/look rows.
const SCENARIOS: &[Scenario] = &[
    Scenario { name: "quote-add", formula: "[5 [[1 3] [1 5]]]", input: &[] },
    Scenario { name: "quote-branch", formula: "[4 [[9 [[1 1] [1 1]]] [[1 100] [1 200]]]]", input: &[] },
    Scenario { name: "quote-cons-mul", formula: "[3 [[7 [[1 6] [1 7]]] [1 9]]]", input: &[] },
    Scenario { name: "hash-42", formula: "[15 [1 42]]", input: &[] },
];

/// Verdict byte: 0 accept, 1 wire bytes do not decode, 2 verifier rejects.
const ACCEPT: u8 = 0;

/// Deterministic single-bit tamperings of `bytes`: the low bit at
/// k/8 of the length for k = 0..8, and the last byte.
pub fn tamperings(bytes: &[u8]) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut positions: Vec<usize> = (0..8).map(|k| bytes.len() * k / 8).collect();
    positions.push(bytes.len() - 1);
    positions.dedup();
    positions
        .into_iter()
        .map(|i| {
            let mut t = bytes.to_vec();
            t[i] ^= 1;
            t
        })
        .collect()
}

fn noun(d: &Data) -> ExecutionNoun {
    match d {
        Data::Atom(v) => ExecutionNoun::Atom(*v),
        Data::Pair(a, b) => ExecutionNoun::Pair(Box::new(noun(a)), Box::new(noun(b))),
    }
}

/// Prove `program` on `input` under `profile`; the envelope's wire bytes.
fn prove(profile: u8, program: &ExecutionNoun, input: &[u64]) -> Result<Vec<u8>, String> {
    let envelope = match profile {
        0 => {
            let (statement, certificate) = zheng::execution::certify_execution(program, input, BUDGET)?;
            Envelope::Public { statement, certificate }
        }
        1 => {
            let (statement, proof) = zheng::execution::succinct::prove_default(program, input, BUDGET)?;
            Envelope::Succinct { statement: SuccinctStatement::Execution(statement), proof }
        }
        other => return Err(format!("profile {other} is not a surface profile")),
    };
    Ok(envelope.to_bytes())
}

/// The verdict on `bytes` as an envelope of `program`.
fn verify(bytes: &[u8], program: &[NounToken]) -> u8 {
    match Envelope::from_bytes(bytes) {
        Err(_) => 1,
        Ok(e) => match e.verify_program(program, None) {
            Ok(()) => ACCEPT,
            Err(_) => 2,
        },
    }
}

/// Snapshot one scenario: the fresh proof, then the verdicts on the fixture.
fn scenario(ctx: &Context, out: &mut Out, profile: u8, name: &str, fresh: Result<Vec<u8>, String>, verify: &dyn Fn(&[u8]) -> u8) {
    let fresh = match fresh {
        Ok(b) => b,
        Err(e) => return out.fail(format!("zheng {name} p{profile}: prover failed: {e}")),
    };
    if verify(&fresh) != ACCEPT {
        out.fail(format!("zheng {name} p{profile}: the verifier rejects a proof the prover just produced"));
    }
    out.mechanism(&format!("zheng::prove/p{profile}@v1"), name, &fresh);

    let rel = format!("zheng/{name}.p{profile}.zheng");
    let fixture = match ctx.fixtures {
        Fixtures::Fresh => fresh.clone(),
        Fixtures::Committed => match std::fs::read(ctx.fixture(&rel)) {
            Ok(b) => b,
            Err(e) => return out.fail(format!("zheng fixture {rel}: {e} (run --bless)")),
        },
    };
    out.fixtures.push((rel.clone(), fresh));

    let verdict = verify(&fixture);
    if verdict != ACCEPT {
        out.fail(format!("zheng {name} p{profile}: the current verifier rejects the committed fixture {rel} (verdict {verdict})"));
    }
    let mut b = Bytes::new();
    b.raw(&crate::Fingerprint::of(&fixture).0).u8(verdict);
    for t in tamperings(&fixture) {
        b.u8(verify(&t));
    }
    out.mechanism(&format!("zheng::verify/p{profile}@v1"), name, &b.0);
}

/// Run the zheng surface.
pub fn run(ctx: &Context, out: &mut Out) {
    // the envelope header: magic, version, header length, the profile bytes
    let mut h = Bytes::new();
    h.raw(zheng::envelope::MAGIC).raw(&zheng::envelope::VERSION.to_le_bytes()).u8(zheng::envelope::HEADER_BYTES as u8);
    for p in [Profile::Public, Profile::Succinct, Profile::Zk, Profile::StatePublic, Profile::Machine, Profile::Recursive, Profile::Wrapped] {
        h.u8(p as u8);
    }
    out.encoding("zheng::envelope::Header@v1", &h.0);

    // a fixed execution statement's canonical transcript bytes
    let statement = ExecutionStatement {
        program: vec![NounToken::Pair, NounToken::Atom(5), NounToken::Pair, NounToken::Atom(1), NounToken::Atom(3)],
        public_input: vec![7, 5],
        public_output: vec![8],
        cycles: 3,
        budget: BUDGET,
    };
    out.encoding("zheng::ExecutionStatement@v1", &statement.transcript_bytes());

    for s in SCENARIOS {
        let built = parse_data(s.formula).and_then(|d| {
            let program = noun(&d);
            let tokens = ExecutionStatement::encode_program(&program)?;
            Ok((program, tokens))
        });
        for profile in PROFILES {
            match &built {
                Ok((program, tokens)) => {
                    let tokens = tokens.clone();
                    scenario(ctx, out, profile, s.name, prove(profile, program, s.input), &move |b| verify(b, &tokens));
                }
                Err(e) => scenario(ctx, out, profile, s.name, Err(e.clone()), &|_| 1),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tamperings_flip_one_bit_each() {
        let b = vec![0u8; 100];
        let t = tamperings(&b);
        assert_eq!(t.len(), 9);
        for x in &t {
            assert_eq!(x.iter().map(|v| v.count_ones()).sum::<u32>(), 1);
        }
        assert!(tamperings(&[]).is_empty());
    }
}
