//! zheng — proof production and verification over committed fixtures.
//!
//! The scenarios are the proof shapes zheng's default branch exposes:
//! `zheng::commit` / `zheng::verify` of whole nox traces (the universal
//! group, plus the hash-binding group for a hash row) and the Brakedown PCS
//! `zheng::open` / `zheng::verify_eval`. The `ZHENGPF1` profiles (0 public,
//! 3 state-public, …) live on the 0.4 integration line, not on zheng's
//! default branch; they enter this surface when they land there.
//!
//! Each scenario has a committed fixture under `conformance/fixtures/zheng/`
//! — the proof's wire bytes (postcard), written by `--bless`. `--check`
//! verifies the committed bytes with the current verifier (rejection is a
//! hard failure), verifies a fixed set of single-bit tamperings (their
//! verdicts are snapshotted), and re-proves the scenario to snapshot the
//! prover's current output.

use nebu::Goldilocks as G;
use nox::{NullCalls, VecTrace};
use serde::{Deserialize, Serialize};
use zheng::{HashAux, ProofParams, Statement, TraceProof};

use super::nox::{Arena, build};
use super::{Bytes, Context, Fixtures, Out};
use crate::text::parse_data;

/// A trace scenario: name, object, formula, budget, whether hash rows get aux.
struct TraceScenario {
    name: &'static str,
    object: &'static str,
    formula: &'static str,
    budget: u64,
}

/// Programs without axis/look rows: their proofs need no opening provider.
const TRACES: &[TraceScenario] = &[
    TraceScenario { name: "quote-add", object: "0", formula: "[5 [[1 3] [1 5]]]", budget: 100 },
    TraceScenario { name: "quote-branch", object: "0", formula: "[4 [[9 [[1 1] [1 1]]] [[1 100] [1 200]]]]", budget: 100 },
    TraceScenario { name: "quote-cons-mul", object: "0", formula: "[3 [[7 [[1 6] [1 7]]] [1 9]]]", budget: 100 },
    TraceScenario { name: "hash-42", object: "42", formula: "[15 [1 42]]", budget: 100 },
];

/// Wire form of a PCS fixture.
#[derive(Serialize, Deserialize)]
struct EvalFixture {
    commitment: lens::Commitment,
    #[serde(with = "zheng::wire::opening")]
    opening: lens::Opening,
}

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

/// Prove one trace scenario. Returns the statement and the proof wire bytes.
fn prove(s: &TraceScenario) -> Result<(Statement, Vec<u8>), String> {
    let mut r = Box::new(Arena::new());
    let object = build(&mut r, &parse_data(s.object)?).ok_or("arena")?;
    let formula = build(&mut r, &parse_data(s.formula)?).ok_or("arena")?;
    let mut trace = VecTrace::default();
    let _ = nox::reduce(&mut r, object, formula, s.budget, &NullCalls, &mut trace);
    if trace.0.is_empty() {
        return Err("empty trace".into());
    }

    // hash rows need the sponge rate: the structural digest of the hashed
    // subject (zheng's own `demo hash` derivation)
    let mut aux = Vec::new();
    for row in &trace.0 {
        if row.r()[0] == 15 {
            let digest = r.digest(object).ok_or("no digest for subject")?;
            let z = G::ZERO;
            aux.push(HashAux { rate: [digest[0], digest[1], digest[2], digest[3], z, z, z, z] });
        }
    }

    let statement = Statement {
        program_hash: nox::particle_id(&r, formula).ok_or("no formula particle")?,
        input_hash: zheng::row_hash(&trace.0[0]),
        output_hash: zheng::row_hash(trace.0.last().expect("non-empty")),
        focus_bound: trace.0.len() as u64,
        bbg_root: [0u8; 32],
    };
    let proof = zheng::commit(&trace, &aux, &[], &[], &statement, &ProofParams::default())
        .map_err(|e| format!("commit: {e:?}"))?;
    let bytes = postcard::to_allocvec(&proof).map_err(|e| format!("encode: {e}"))?;
    Ok((statement, bytes))
}

fn verify_trace(bytes: &[u8], statement: &Statement) -> u8 {
    match postcard::from_bytes::<TraceProof>(bytes) {
        Err(_) => 1,
        Ok(p) => match zheng::verify(&p, statement, &ProofParams::default()) {
            Ok(()) => ACCEPT,
            Err(_) => 2,
        },
    }
}

/// The PCS scenario's fixed polynomial, point and claimed value.
fn eval_inputs() -> (Vec<G>, Vec<G>, G) {
    let poly: Vec<G> = super::ramp(0x7a68_656e_67, 16).into_iter().map(G::new).collect();
    let point: Vec<G> = super::ramp(4, 4).into_iter().map(G::new).collect();
    let value = lens::MultilinearPoly::new(poly.clone()).evaluate(&point);
    (poly, point, value)
}

fn prove_eval() -> Result<Vec<u8>, String> {
    let (poly, point, _) = eval_inputs();
    let (commitment, opening) = zheng::open(&poly, &point, &ProofParams::default()).map_err(|e| format!("open: {e:?}"))?;
    postcard::to_allocvec(&EvalFixture { commitment, opening }).map_err(|e| format!("encode: {e}"))
}

fn verify_eval(bytes: &[u8]) -> u8 {
    let (_, point, value) = eval_inputs();
    match postcard::from_bytes::<EvalFixture>(bytes) {
        Err(_) => 1,
        Ok(f) => match zheng::verify_eval(&f.commitment, &point, value, &f.opening, &ProofParams::default()) {
            Ok(()) => ACCEPT,
            Err(_) => 2,
        },
    }
}

/// Snapshot one scenario: the fresh proof, then the verdicts on the fixture.
fn scenario(ctx: &Context, out: &mut Out, name: &str, fresh: Result<Vec<u8>, String>, verify: &dyn Fn(&[u8]) -> u8) {
    let fresh = match fresh {
        Ok(b) => b,
        Err(e) => return out.fail(format!("zheng {name}: prover failed: {e}")),
    };
    if verify(&fresh) != ACCEPT {
        out.fail(format!("zheng {name}: the verifier rejects a proof the prover just produced"));
    }
    out.mechanism("zheng::commit@v1", name, &fresh);

    let rel = format!("zheng/{name}.proof");
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
        out.fail(format!("zheng {name}: the current verifier rejects the committed fixture {rel} (verdict {verdict})"));
    }
    let mut b = Bytes::new();
    b.raw(&crate::Fingerprint::of(&fixture).0).u8(verdict);
    for t in tamperings(&fixture) {
        b.u8(verify(&t));
    }
    out.mechanism("zheng::verify@v1", name, &b.0);
}

/// Run the zheng surface.
pub fn run(ctx: &Context, out: &mut Out) {
    let params = ProofParams::default();
    let mut p = Bytes::new();
    p.u64(params.security.lambda() as u64).u8(matches!(params.lens, zheng::LensBackend::Brakedown) as u8).u32(params.max_trace_log);
    out.encoding("zheng::ProofParams::default@v1", &p.0);
    let statement = Statement {
        program_hash: core::array::from_fn(|i| i as u8),
        input_hash: [1u8; 32],
        output_hash: [2u8; 32],
        focus_bound: 1 << 20,
        bbg_root: [3u8; 32],
    };
    out.encoding("zheng::Statement@v1", &postcard::to_allocvec(&statement).unwrap_or_default());

    for s in TRACES {
        match prove(s) {
            Ok((statement, fresh)) => {
                let st = statement.clone();
                scenario(ctx, out, s.name, Ok(fresh), &move |b| verify_trace(b, &st));
            }
            Err(e) => scenario(ctx, out, s.name, Err(e), &|_| 1),
        }
    }
    scenario(ctx, out, "pcs-eval", prove_eval(), &verify_eval);
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
