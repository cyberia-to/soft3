//! hemera — the fingerprint function itself (cyber-hemera 0.3, 32-byte output).
//!
//! Scenarios cover hemera's published vectors (`hemera/vectors/hemera.json`),
//! sponge rate boundaries (56-byte blocks), the chunk tree across the
//! 4 KiB chunk boundary, keyed/derive-key/secret modes and the raw
//! Poseidon2 permutation. The published vectors are also checked as an
//! invariant: hemera disagreeing with its own vector file fails the run.

use hemera::field::Goldilocks as F;

use super::{Bytes, Context, Out, ramp};

fn ramp_bytes(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 251) as u8).collect()
}

/// `(scenario, input)` for the plain sponge.
fn hash_inputs() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("empty", b"".to_vec()),
        ("hello", b"hello".to_vec()),
        ("hemera", b"hemera".to_vec()),
        ("ramp-55", ramp_bytes(55)),
        ("ramp-56", ramp_bytes(56)),
        ("ramp-57", ramp_bytes(57)),
        ("ramp-112", ramp_bytes(112)),
        ("ramp-1000", ramp_bytes(1000)),
        ("zeros-56", vec![0u8; 56]),
        ("ones-64", vec![0xFFu8; 64]),
    ]
}

fn tree_inputs() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("empty", b"".to_vec()),
        ("hello", b"hello".to_vec()),
        ("4k_zeros", vec![0u8; 4096]),
        ("ramp-4097", ramp_bytes(4097)),
        ("ramp-12288", ramp_bytes(12288)),
        ("ramp-20000", ramp_bytes(20000)),
    ]
}

/// Run the hemera surface.
pub fn run(ctx: &Context, out: &mut Out) {
    // published vectors — invariant, then snapshot
    let published = ctx.stack.join("hemera/vectors/hemera.json");
    match std::fs::read_to_string(&published) {
        Ok(src) => check_published(&src, out),
        Err(e) => out.fail(format!("hemera vectors {}: {e}", published.display())),
    }

    for (name, input) in hash_inputs() {
        out.mechanism("hemera::hash@v1", name, hemera::hash(&input).as_bytes());
        let secret = hemera::hash_secret(&input);
        if secret != hemera::hash(&input) {
            out.fail(format!("hemera::hash_secret({name}) differs from hemera::hash"));
        }
    }
    for (name, input) in tree_inputs() {
        out.mechanism("hemera::tree::root_hash@v1", name, hemera::tree::root_hash(&input).as_bytes());
        out.mechanism("hemera::tree::fixed_chunk_root@v1", name, hemera::tree::fixed_chunk_root(&input).as_bytes());
    }
    let key: [u8; 32] = core::array::from_fn(|i| i as u8);
    out.mechanism("hemera::keyed_hash@v1", "key-ramp/hello", hemera::keyed_hash(&key, b"hello").as_bytes());
    out.mechanism("hemera::keyed_hash@v1", "key-ramp/ramp-1000", hemera::keyed_hash(&key, &ramp_bytes(1000)).as_bytes());
    out.mechanism("hemera::derive_key@v1", "test", &hemera::derive_key("test context", b"test material"));
    out.mechanism("hemera::derive_key@v1", "conformance", &hemera::derive_key("soft3 conformance v1", &ramp_bytes(77)));

    // permutation: zero state and a ramp state
    for (name, init) in [("zero-state", [0u64; 16]), ("ramp-state", {
        let r = ramp(0x6865_6d65_7261, 16);
        core::array::from_fn(|i| r[i])
    })] {
        let mut state: [F; 16] = core::array::from_fn(|i| F::new(init[i]));
        hemera::permutation::permute(&mut state);
        let mut b = Bytes::new();
        for e in state {
            b.u64(e.as_canonical_u64());
        }
        out.mechanism("hemera::permutation@v1", name, &b.0);
    }

    // parameters are part of the profile: a change here is a new function
    let mut p = Bytes::new();
    for v in [
        hemera::WIDTH,
        hemera::RATE,
        hemera::CAPACITY,
        hemera::ROUNDS_F,
        hemera::ROUNDS_P,
        hemera::OUTPUT_BYTES,
        hemera::OUTPUT_ELEMENTS,
        hemera::RATE_BYTES,
        hemera::CHUNK_SIZE,
    ] {
        p.u64(v as u64);
    }
    p.u64(hemera::SBOX_DEGREE as u64).u64(hemera::COLLISION_BITS as u64);
    out.encoding("hemera::params@v1", &p.0);
}

fn check_published(src: &str, out: &mut Out) {
    let v: serde_json::Value = match serde_json::from_str(src) {
        Ok(v) => v,
        Err(e) => return out.fail(format!("hemera vectors: {e}")),
    };
    let mut expect = |section: &str, key: &str, got: String| match v[section][key].as_str() {
        Some(want) if want == got => {}
        Some(want) => out.fail(format!("hemera vector {section}.{key}: published {want}, computed {got}")),
        None => out.fail(format!("hemera vector {section}.{key} missing from hemera.json")),
    };
    for (k, input) in [("empty", &b""[..]), ("hello", b"hello"), ("hemera", b"hemera")] {
        expect("hash", k, hemera::hash(input).to_string());
    }
    for (k, input) in [("empty", b"".to_vec()), ("hello", b"hello".to_vec()), ("4k_zeros", vec![0u8; 4096])] {
        // the published `tree` section is the fixed-chunk (BAO) tree:
        // root_hash moved to content-defined chunking and agrees with it
        // only below ~2 KiB (hemera rs/src/tree.rs, root_hash docs)
        expect("tree", k, hemera::tree::fixed_chunk_root(&input).to_string());
    }
    expect("derive_key", "test", crate::text::to_hex(&hemera::derive_key("test context", b"test material")));
}
