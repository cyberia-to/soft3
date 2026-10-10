//! nebu — Goldilocks field arithmetic (strata-nebu), the number system every
//! other surface is built on. Field arithmetic only: every value is a
//! canonical element of p = 2^64 − 2^32 + 1, written as 8 little-endian bytes.

use nebu::{Fp2, Fp3, Fp4, Goldilocks as G};

use super::{Bytes, Context, EDGES, Out, ramp};

fn g(v: u64) -> G {
    G::new(v)
}

fn put(b: &mut Bytes, x: G) {
    b.u64(x.as_u64());
}

/// Run the nebu surface.
pub fn run(_ctx: &Context, out: &mut Out) {
    let edges: Vec<G> = EDGES.iter().map(|&v| g(v)).collect();
    let rnd: Vec<G> = ramp(0x6e65_6275, 32).into_iter().map(g).collect();

    // canonical encoding of the element, including reduction of u64 inputs ≥ p
    let mut enc = Bytes::new();
    for &v in EDGES.iter().chain(&[nebu::field::P, nebu::field::P + 1, u64::MAX]) {
        put(&mut enc, g(v));
    }
    out.encoding("nebu::Goldilocks@v1", &enc.0);

    let binary: [(&str, fn(G, G) -> G); 3] = [
        ("nebu::add@v1", |a, b| a + b),
        ("nebu::sub@v1", |a, b| a - b),
        ("nebu::mul@v1", |a, b| a * b),
    ];
    for (name, op) in binary {
        for (scenario, set) in [("edges×edges", &edges), ("ramp×ramp", &rnd)] {
            let mut b = Bytes::new();
            for &x in set.iter() {
                for &y in set.iter() {
                    put(&mut b, op(x, y));
                }
            }
            out.mechanism(name, scenario, &b.0);
        }
    }

    let unary: [(&str, fn(G) -> G); 4] = [
        ("nebu::neg@v1", |a| -a),
        ("nebu::square@v1", |a| a.square()),
        ("nebu::pow7@v1", |a| a.pow7()),
        ("nebu::inv@v1", |a| a.inv()),
    ];
    for (name, op) in unary {
        for (scenario, set) in [("edges", &edges), ("ramp", &rnd)] {
            let mut b = Bytes::new();
            for &x in set.iter() {
                put(&mut b, op(x));
            }
            out.mechanism(name, scenario, &b.0);
        }
    }

    let mut b = Bytes::new();
    for &x in &edges {
        for e in [0u64, 1, 2, 7, 1 << 32, nebu::field::P - 2, nebu::field::P - 1, u64::MAX] {
            put(&mut b, x.exp(e));
        }
    }
    out.mechanism("nebu::exp@v1", "edges^exponents", &b.0);

    let mut b = Bytes::new();
    for &x in edges.iter().chain(&rnd) {
        match nebu::sqrt::sqrt(x) {
            // the root's sign is part of the mechanism: record what nebu returns
            Some(r) => {
                b.u8(1);
                put(&mut b, r);
            }
            None => {
                b.u8(0);
            }
        }
        put(&mut b, nebu::sqrt::legendre(x));
    }
    out.mechanism("nebu::sqrt@v1", "edges+ramp", &b.0);

    let mut inv = vec![G::ZERO; rnd.len()];
    nebu::batch::batch_inv(&rnd, &mut inv);
    let mut b = Bytes::new();
    for x in inv {
        put(&mut b, x);
    }
    out.mechanism("nebu::batch_inv@v1", "ramp", &b.0);

    for n in [8usize, 32] {
        let mut a: Vec<G> = ramp(n as u64, n).into_iter().map(g).collect();
        let original = a.clone();
        nebu::ntt::ntt(&mut a);
        let mut b = Bytes::new();
        for &x in &a {
            put(&mut b, x);
        }
        out.mechanism("nebu::ntt@v1", &format!("ramp-{n}"), &b.0);
        nebu::ntt::intt(&mut a);
        if a != original {
            out.fail(format!("nebu::intt(ntt(x)) != x for n={n}"));
        }
    }

    let evals: Vec<G> = ramp(16, 16).into_iter().map(g).collect();
    let point: Vec<G> = ramp(4, 4).into_iter().map(g).collect();
    out.mechanism("nebu::multilinear_eval@v1", "ramp-16@ramp-4", &nebu::multilinear_eval(&evals, &point).as_u64().to_le_bytes());

    let src: Vec<u8> = (0..100u32).map(|i| (i * 37 % 256) as u8).collect();
    let mut elems = vec![G::ZERO; 16];
    let n = nebu::encoding::bytes_to_field_elements(&src, &mut elems);
    let mut b = Bytes::new();
    b.u64(n as u64);
    for x in &elems[..n] {
        put(&mut b, *x);
    }
    out.mechanism("nebu::encoding::bytes_to_field_elements@v1", "ramp-100", &b.0);

    // extension fields: products and inverses of fixed elements
    let r = |i: usize| rnd[i % rnd.len()];
    let a2 = Fp2 { re: r(0), im: r(1) };
    let b2 = Fp2 { re: r(2), im: r(3) };
    let a3 = Fp3 { c0: r(4), c1: r(5), c2: r(6) };
    let b3 = Fp3 { c0: r(7), c1: r(8), c2: r(9) };
    let a4 = Fp4 { c0: r(10), c1: r(11), c2: r(12), c3: r(13) };
    let b4 = Fp4 { c0: r(14), c1: r(15), c2: r(16), c3: r(17) };
    let mut b = Bytes::new();
    for x in [a2 * b2, a2.inv(), a2.sqr()] {
        put(&mut b, x.re);
        put(&mut b, x.im);
    }
    out.mechanism("nebu::Fp2@v1", "mul+inv+sqr", &b.0);
    let mut b = Bytes::new();
    for x in [a3 * b3, a3.inv(), a3.sqr()] {
        put(&mut b, x.c0);
        put(&mut b, x.c1);
        put(&mut b, x.c2);
    }
    out.mechanism("nebu::Fp3@v1", "mul+inv+sqr", &b.0);
    let mut b = Bytes::new();
    for x in [a4 * b4, a4.inv(), a4.sqr(), a4.frobenius()] {
        put(&mut b, x.c0);
        put(&mut b, x.c1);
        put(&mut b, x.c2);
        put(&mut b, x.c3);
    }
    out.mechanism("nebu::Fp4@v1", "mul+inv+sqr+frobenius", &b.0);

    // spec vectors (nox specs/patterns/README.md, test vectors) as invariants
    if g(1) + g(2) != g(3) || g(nebu::field::P - 1) * g(nebu::field::P - 1) != G::ONE || g(2).inv() != g(9223372034707292161) {
        out.fail("nebu disagrees with the canonical arithmetic vectors");
    }
}
