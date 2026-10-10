//! nox — the proof-native VM: data encoding, content identity, wire
//! messages, and reduction of fixed programs.
//!
//! Programs come from two places: `conformance/fixtures/nox/*.nox` (the
//! pattern spec's test vectors and one program per pattern) and the nox
//! repo's own `jets/*.nox` formulas, read from the sibling checkout so that
//! a change to a jet formula is drift. Each program yields two mechanism
//! entries: the reduction outcome (result tree, cost, or halt/error kind)
//! and the execution trace that zheng proves.

use nebu::Goldilocks as G;
use nox::{JetRegistry, NullCalls, Order, Outcome, Reduction, VecTrace};

use super::{Bytes, Context, EDGES, Out, list};
use crate::text::{Data, Expect, Program, parse_program};

/// Arena size for every fixture reduction. Part of the scenario: a program
/// that exhausts the arena reports it as its outcome.
pub const ARENA: usize = 1 << 14;

/// Arena type used by every surface that reduces nox programs.
pub type Arena = Reduction<ARENA>;

/// Build `data` into the arena.
pub fn build(r: &mut Arena, data: &Data) -> Option<Order> {
    match data {
        Data::Atom(v) => r.atom(G::new(*v)),
        Data::Pair(a, b) => {
            let a = build(r, a)?;
            let b = build(r, b)?;
            r.pair(a, b)
        }
    }
}

/// Read a data tree back out of the arena.
pub fn read(r: &Arena, id: Order) -> Option<Data> {
    if r.is_atom(id) {
        return Some(Data::Atom(r.atom_value(id)?.as_u64()));
    }
    let (h, t) = (r.head(id)?, r.tail(id)?);
    Some(Data::Pair(Box::new(read(r, h)?), Box::new(read(r, t)?)))
}

/// Canonical bytes of a whole tree: every reachable node, children first,
/// as `particle ‖ len ‖ encoding`.
pub fn tree_bytes(r: &Arena, id: Order) -> Option<Vec<u8>> {
    let mut b = Bytes::new();
    for (pid, bytes) in nox::encode_tree(r, id)? {
        b.raw(&pid).lv(&bytes);
    }
    Some(b.0)
}

/// Reduce `program` with the genesis jet registry (the production path).
/// Returns the canonical outcome bytes and the trace bytes.
pub fn reduce(program: &Program) -> Result<(Vec<u8>, Vec<u8>), String> {
    let (outcome, trace, mismatch) = reduce_checked(program)?;
    match mismatch {
        Some(m) => Err(m),
        None => Ok((outcome, trace)),
    }
}

/// [`reduce`], plus the fixture's asserted outcome checked: the third value
/// names the mismatch, if any.
pub fn reduce_checked(program: &Program) -> Result<(Vec<u8>, Vec<u8>, Option<String>), String> {
    let mut r = Box::new(Arena::new());
    let object = build(&mut r, &program.object).ok_or("arena full building object")?;
    let formula = build(&mut r, &program.formula).ok_or("arena full building formula")?;
    let registry = JetRegistry::<ARENA>::genesis();
    let mut trace = VecTrace::default();
    let outcome = nox::reduce_with_registry(&mut r, object, formula, program.budget, &NullCalls, &mut trace, &registry);

    let actual = match &outcome {
        Outcome::Ok(result, remaining) => format!("result {:?} cost {}", read(&r, *result), program.budget - remaining),
        Outcome::Halt(_) => "halt".to_string(),
        Outcome::Error(kind) => format!("error {kind:?}"),
    };
    let mismatch = match (&program.expect, &outcome) {
        (None, _) => None,
        (Some(Expect::Result(want, cost)), Outcome::Ok(result, remaining))
            if read(&r, *result).as_ref() == Some(want) && cost.is_none_or(|c| c == program.budget - remaining) =>
        {
            None
        }
        (Some(Expect::Error(want)), Outcome::Error(kind)) if format!("{kind:?}") == *want => None,
        (Some(Expect::Halt), Outcome::Halt(_)) => None,
        (Some(want), _) => Some(format!("expected {want:?}, got {actual}")),
    };

    let mut b = Bytes::new();
    b.raw(&nox::particle_id(&r, formula).ok_or("no formula particle")?);
    b.raw(&nox::particle_id(&r, object).ok_or("no object particle")?);
    b.u64(program.budget);
    match outcome {
        Outcome::Ok(result, remaining) => {
            b.u8(0);
            b.raw(&nox::particle_id(&r, result).ok_or("no result particle")?);
            b.lv(&tree_bytes(&r, result).ok_or("result tree unencodable")?);
            b.u64(program.budget - remaining);
        }
        Outcome::Halt(remaining) => {
            b.u8(1).u64(remaining);
        }
        Outcome::Error(kind) => {
            b.u8(2).u8(kind as u8);
        }
    }
    Ok((b.0, trace_bytes(&trace), mismatch))
}

/// Canonical bytes of a trace: row count, then 16 little-endian columns per row.
pub fn trace_bytes(trace: &VecTrace) -> Vec<u8> {
    let mut b = Bytes::new();
    b.u64(trace.0.len() as u64);
    for row in &trace.0 {
        for &c in row.r() {
            b.u64(c);
        }
    }
    b.0
}

/// Every program the surface reduces, as `(scenario, source)`.
pub fn programs(ctx: &Context) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let own = ctx.fixture("nox");
    for name in list(&own, ".nox")? {
        let src = std::fs::read_to_string(own.join(&name)).map_err(|e| format!("{name}: {e}"))?;
        out.push((name.trim_end_matches(".nox").to_string(), src));
    }
    let jets = ctx.stack.join("nox/jets");
    for name in list(&jets, ".nox")? {
        let src = std::fs::read_to_string(jets.join(&name)).map_err(|e| format!("jets/{name}: {e}"))?;
        out.push((format!("jets/{}", name.trim_end_matches(".nox")), src));
    }
    Ok(out)
}

/// Run the nox surface.
pub fn run(ctx: &Context, out: &mut Out) {
    encodings(out);
    let programs = match programs(ctx) {
        Ok(p) => p,
        Err(e) => return out.fail(format!("nox fixtures: {e}")),
    };
    for (scenario, src) in programs {
        let program = match parse_program(&src) {
            Ok(p) => p,
            Err(e) => {
                out.fail(format!("nox fixture {scenario}: {e}"));
                continue;
            }
        };
        match reduce(&program) {
            Ok((outcome, trace)) => {
                out.mechanism("nox::reduce@v1", &scenario, &outcome);
                out.mechanism("nox::trace@v1", &scenario, &trace);
            }
            Err(e) => out.fail(format!("nox fixture {scenario}: {e}")),
        }
    }
}

fn encodings(out: &mut Out) {
    let mut b = Bytes::new();
    for &v in &EDGES {
        b.raw(&nox::encode_atom(G::new(v)));
    }
    out.encoding("nox::atom@v1", &b.0);

    let one = nox::particle_of(&nox::encode_atom(G::new(1))).expect("atom particle");
    let two = nox::particle_of(&nox::encode_atom(G::new(2))).expect("atom particle");
    let pair = nox::encode_pair(&one, &two);
    let mut b = Bytes::new();
    b.raw(&pair).raw(&nox::particle_of(&pair).expect("pair particle"));
    out.encoding("nox::pair@v1", &b.0);

    // the pattern spec's branch vector, as a tree, and as wire messages
    let formula = crate::text::parse_data("[4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]]").expect("literal");
    let mut r = Box::new(Arena::new());
    let id = build(&mut r, &formula).expect("arena");
    out.encoding("nox::tree@v1", &tree_bytes(&r, id).expect("tree"));
    let entries = nox::encode_tree(&r, id).expect("tree");
    let ids: Vec<nox::Particle> = entries.iter().map(|(p, _)| *p).collect();
    out.encoding("nox::wire::push@v1", &nox::write_push(&entries).unwrap_or_default());
    out.encoding("nox::wire::request@v1", &nox::write_request(&ids).unwrap_or_default());
    out.encoding("nox::wire::response@v1", &nox::write_response(&entries).unwrap_or_default());
}
