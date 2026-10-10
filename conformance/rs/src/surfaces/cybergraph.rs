//! cybergraph — identities of the five primitives' graph-facing values:
//! a file's particle, an axon's particle, a signal's chain-position hash and
//! content id, a neuron's private network, an intent scope, and the bbg
//! state root after a fixed sequence of signals passes through the
//! cybergraph `link` verb.

use cybergraph::{Cybergraph, CyberlinkRecord, NeuronId, Particle, Scope, Signal};

use super::{Bytes, Context, Out};

fn n(seed: u8) -> NeuronId {
    [seed; 32]
}

/// A file's particle: the hemera hash of its bytes.
fn particle(content: &[u8]) -> Particle {
    *hemera::hash(content).as_bytes()
}

fn link(neuron: NeuronId, from: &[u8], to: &[u8], amount: u64, valence: i8) -> CyberlinkRecord {
    CyberlinkRecord {
        neuron,
        from: particle(from),
        to: particle(to),
        token: particle(b"CYB"),
        amount,
        valence,
        height: 0,
    }
}

fn signal(neuron: NeuronId, step: u64, prev: Particle, links: Vec<CyberlinkRecord>) -> Signal {
    Signal {
        neuron,
        network: cybergraph::SELF_NETWORK,
        links,
        delta_pi: vec![],
        box_moves: vec![],
        prev,
        step,
        height: 0,
        proof: None,
    }
}

/// Run the cybergraph surface.
pub fn run(_ctx: &Context, out: &mut Out) {
    for (name, content) in [
        ("empty", b"".to_vec()),
        ("hello world", b"hello world".to_vec()),
        ("ramp-5000", (0..5000u32).map(|i| (i % 253) as u8).collect()),
    ] {
        out.mechanism("cybergraph::particle@v1", name, &particle(&content));
    }

    let a = particle(b"cyber");
    let b = particle(b"soft3");
    let mut ax = Bytes::new();
    ax.raw(&bbg::state::axon_id(&a, &b)).raw(&bbg::state::axon_id(&b, &a));
    out.mechanism("bbg::axon_id@v1", "cyber<->soft3", &ax.0);
    out.mechanism("bbg::balance_key@v1", "neuron-7/CYB", &bbg::balance_key(&n(7), &particle(b"CYB")));

    out.mechanism("cybergraph::private_network@v1", "neuron-7", &cybergraph::private_network(&n(7)));
    let scope = Scope { target: a, predicate: b"do-thing".to_vec(), deadline: Some(100), constraints: b"none".to_vec() };
    out.mechanism("cybergraph::Scope::hash@v1", "do-thing", &scope.hash());

    let s0 = signal(n(7), 0, [0u8; 32], vec![link(n(7), b"cyber", b"soft3", 10, 1), link(n(7), b"soft3", b"nox", 3, -1)]);
    let mut s1 = signal(n(7), 1, s0.hash(), vec![link(n(7), b"nox", b"zheng", 1, 0)]);
    s1.delta_pi = vec![(a, 5)];
    for (name, s) in [("step-0", &s0), ("step-1", &s1)] {
        out.mechanism("foculus::Signal::hash@v1", name, &s.hash());
        out.mechanism("foculus::Signal::content_id@v1", name, &s.content_id());
    }

    let mut graph = Cybergraph::new();
    let mut st = Bytes::new();
    for s in [s0, s1] {
        if let Err(e) = graph.link(s) {
            out.fail(format!("cybergraph link rejected a fixture signal: {e:?}"));
        }
        st.raw(&graph.bbg.state.root());
    }
    out.mechanism("cybergraph::link@v1", "neuron-7/two-signals/bbg-root", &st.0);
}
