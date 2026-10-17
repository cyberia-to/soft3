//! The focus graph (chain-economy §4.3) and the φ* / gradient engine seam
//! (§4.4, §5.1).
//!
//! The chain hands the engine one canonical graph: present axons
//! `(p, q, A_pq)` ascending by `p ‖ q`, particles indexed by ascending id.
//! The engine returns φ* (tru's raw fixed point, 2^60 = one unit of mass),
//! the syntropy J(φ*) at the same scale, and for the gradient solve one
//! signed fixed-point `∂J/∂A_pq` per present axon.
//!
//! [`FocusEngine`] is the seam tru's epoch entry point plugs into
//! (tru branch `feat/epoch-settlement`). Until it lands the chain runs
//! [`Reference`]: the same contract — cold start from uniform, exactly
//! `PHI_STEPS` steps, integers only, thread-count independent, gradient by
//! one adjoint solve — over the diffusion term of the tri-kernel alone. Its
//! bits are not tru's: swapping engines is a consensus change and moves the
//! epoch vectors of conformance §14, which is why the engine's
//! [`FocusEngine::id`] is part of every cached result's key.

use super::super::state::Id;
use super::weights::Axon;

/// One unit of focus mass: φ is at scale 2^60 (§4.4).
pub const PHI_BITS: u32 = 60;
/// The scale of `ln` inside the solves.
pub const LN_BITS: u32 = 40;
/// The scale of the gradient: `∂J/∂A` in nats per token at 2^100
/// (φ at 2^60 times λ at 2^40). Attribution multiplies it by a stake
/// change and divides it out only as a ratio (§5.3, §5.5), so the scale
/// cancels in every payout.
pub const GRAD_BITS: u32 = PHI_BITS + LN_BITS;

/// The canonical input of one solve (§4.3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FocusGraph {
    /// every endpoint of a present axon, ascending
    pub particles: Vec<Id>,
    /// edges ascending by `(src, dst)` = by `p ‖ q`
    pub src: Vec<u32>,
    pub dst: Vec<u32>,
    pub weight: Vec<u128>,
}

impl FocusGraph {
    /// From present axons ascending by `p ‖ q` (every weight > 0).
    pub fn build(axons: &[Axon<u128>]) -> FocusGraph {
        let mut particles: Vec<Id> = Vec::with_capacity(axons.len() * 2);
        for (p, q, _) in axons {
            particles.push(*p);
            particles.push(*q);
        }
        particles.sort_unstable();
        particles.dedup();
        let index = |x: &Id| particles.binary_search(x).expect("endpoint indexed") as u32;
        let mut g = FocusGraph {
            src: Vec::with_capacity(axons.len()),
            dst: Vec::with_capacity(axons.len()),
            weight: Vec::with_capacity(axons.len()),
            particles: Vec::new(),
        };
        for (p, q, w) in axons {
            debug_assert!(*w > 0);
            g.src.push(index(p));
            g.dst.push(index(q));
            g.weight.push(*w);
        }
        g.particles = particles;
        g
    }

    pub fn n(&self) -> usize {
        self.particles.len()
    }

    pub fn edges(&self) -> usize {
        self.src.len()
    }
}

/// φ* of one graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Focus {
    /// φ*, indexed as [`FocusGraph::particles`], at 2^60
    pub phi: Vec<u64>,
    /// `J(φ*) = Σ φ ln(n φ)`, nats at 2^60
    pub syntropy: i128,
}

/// The φ* / gradient engine (§4.4, §5.1). Implementations must be pure
/// functions of the graph: same bits on every machine and thread count.
pub trait FocusEngine: Send + Sync {
    /// A short version tag; results cached on disk are keyed by it.
    fn id(&self) -> &'static str;

    /// φ*(A): cold start, `PHI_STEPS` steps.
    fn focus(&self, g: &FocusGraph) -> Focus;

    /// `∂J(φ*(A))/∂A_pq` for every edge of `g` in edge order, at scale
    /// 2^[`GRAD_BITS`]: one forward solve to the fixed point of `g`, one
    /// adjoint solve, one read per edge (§5.1).
    fn gradient(&self, g: &FocusGraph) -> Vec<i128>;
}

/// `ln 2` at 2^40.
const LN2_Q40: i128 = 762_123_384_786;

/// `log2(v / 2^frac_bits)` at 2^40 for `v > 0`: integer part from the bit
/// length, 40 fraction bits by repeated squaring (no table, no float).
pub fn log2_q40(v: u128, frac_bits: u32) -> i128 {
    debug_assert!(v > 0);
    let k = 127 - v.leading_zeros(); // floor(log2 v)
    // y ∈ [1, 2) at 2^62
    let mut y: u128 = if k >= 62 { v >> (k - 62) } else { v << (62 - k) };
    let mut frac: i128 = 0;
    for i in 1..=LN_BITS {
        y = (y * y) >> 62;
        if y >= 1u128 << 63 {
            y >>= 1;
            frac |= 1 << (LN_BITS - i);
        }
    }
    ((k as i128 - frac_bits as i128) << LN_BITS) + frac
}

/// `ln(v / 2^frac_bits)` at 2^40.
pub fn ln_q40(v: u128, frac_bits: u32) -> i128 {
    (log2_q40(v, frac_bits) * LN2_Q40) >> LN_BITS
}

/// The integer reference engine: the diffusion term of the tri-kernel as
/// a random walk with teleport α = 15/100 to the stake prior
/// `π = deg / Σ deg` (weighted total degree — tru's epoch engine makes the
/// same choice, so a particle's focus follows the stake on its axons),
/// dangling mass spread by π, `PHI_STEPS` steps from uniform,
/// `J = Σ φ ln(n φ)`, and the exact adjoint of that map.
pub struct Reference {
    pub steps: usize,
}

impl Default for Reference {
    fn default() -> Self {
        Reference { steps: super::super::params::PHI_STEPS }
    }
}

const ALPHA_NUM: u128 = 15;
const ALPHA_DEN: u128 = 100;
const KEEP_NUM: u128 = ALPHA_DEN - ALPHA_NUM;
/// transition probabilities at 2^62
const T_BITS: u32 = 62;

struct Walk {
    /// out-weight of each particle
    out: Vec<u128>,
    /// `floor(A_e · 2^62 / out(src))` per edge
    t: Vec<u64>,
    /// `Σ deg = 2 Σ A`
    w2: u128,
    /// the stake prior at 2^60
    prior: Vec<u128>,
}

impl Walk {
    fn new(g: &FocusGraph) -> Walk {
        let mut out = vec![0u128; g.n()];
        let mut deg = vec![0u128; g.n()];
        for e in 0..g.edges() {
            let w = g.weight[e];
            out[g.src[e] as usize] += w;
            deg[g.src[e] as usize] += w;
            deg[g.dst[e] as usize] += w;
        }
        let t = g
            .src
            .iter()
            .zip(&g.weight)
            .map(|(s, w)| {
                // w ≤ d; both shifted by the same k so d fits 64 bits and
                // (w << 62) fits u128 — a fixed rule, so every node agrees
                let d = out[*s as usize];
                let k = (128 - d.leading_zeros()).saturating_sub(64);
                (((*w >> k) << T_BITS) / (d >> k)) as u64
            })
            .collect();
        let w2: u128 = deg.iter().sum();
        // deg < 2^67, so deg << 60 fits u128
        let prior = deg.iter().map(|d| (d << PHI_BITS) / w2).collect();
        Walk { out, t, w2, prior }
    }

    /// `⟨π, v⟩` at v's scale.
    fn prior_dot(&self, v: &[i128]) -> i128 {
        self.prior.iter().zip(v).map(|(p, x)| (*p as i128 * x) >> PHI_BITS).sum()
    }
}

impl Reference {
    fn forward(&self, g: &FocusGraph, w: &Walk) -> Vec<u64> {
        let n = g.n() as u128;
        let one = 1u128 << PHI_BITS;
        let mut phi = vec![(one / n) as u64; g.n()];
        let mut inflow = vec![0u128; g.n()];
        for _ in 0..self.steps {
            inflow.iter_mut().for_each(|x| *x = 0);
            let dangling: u128 = w.out.iter().zip(&phi).filter(|(d, _)| **d == 0).map(|(_, f)| *f as u128).sum();
            for e in 0..g.edges() {
                inflow[g.dst[e] as usize] += (phi[g.src[e] as usize] as u128 * w.t[e] as u128) >> T_BITS;
            }
            for (j, f) in phi.iter_mut().enumerate() {
                let pi = w.prior[j];
                *f = ((ALPHA_NUM * pi + KEEP_NUM * (inflow[j] + ((dangling * pi) >> PHI_BITS))) / ALPHA_DEN) as u64;
            }
        }
        phi
    }

    fn syntropy(phi: &[u64]) -> i128 {
        let n = phi.len() as u128;
        phi.iter().filter(|f| **f > 0).map(|f| (*f as i128 * ln_q40(n * *f as u128, PHI_BITS)) >> LN_BITS).sum()
    }

    /// `(Pλ)_i`: the walk's expectation of λ one step from `i` (a dangling
    /// particle jumps by the prior).
    fn expect(g: &FocusGraph, w: &Walk, lambda: &[i128]) -> Vec<i128> {
        let jump = w.prior_dot(lambda);
        let mut out: Vec<i128> = w.out.iter().map(|d| if *d == 0 { jump } else { 0 }).collect();
        for e in 0..g.edges() {
            out[g.src[e] as usize] += (lambda[g.dst[e] as usize] * w.t[e] as i128) >> T_BITS;
        }
        out
    }
}

impl FocusEngine for Reference {
    fn id(&self) -> &'static str {
        "soft3-reference-prior-walk-v1"
    }

    fn focus(&self, g: &FocusGraph) -> Focus {
        if g.n() == 0 {
            return Focus::default();
        }
        let w = Walk::new(g);
        let phi = self.forward(g, &w);
        let syntropy = Self::syntropy(&phi);
        Focus { phi, syntropy }
    }

    fn gradient(&self, g: &FocusGraph) -> Vec<i128> {
        if g.n() == 0 {
            return Vec::new();
        }
        let w = Walk::new(g);
        let phi = self.forward(g, &w);
        let n = g.n() as u128;
        // ∇_φ J = ln(n φ) + 1 at 2^40
        let grad_j: Vec<i128> = phi.iter().map(|f| ln_q40(n * (*f).max(1) as u128, PHI_BITS) + (1 << LN_BITS)).collect();
        // λ ← ∇J + (∂F/∂φ)ᵀ λ from λ = 0, with (∂F/∂φ)ᵀ λ = (1 − α) P λ
        let mut lambda = vec![0i128; g.n()];
        for _ in 0..self.steps {
            let pl = Self::expect(g, &w, &lambda);
            for (i, l) in lambda.iter_mut().enumerate() {
                *l = grad_j[i] + (KEEP_NUM as i128 * pl[i]).div_euclid(ALPHA_DEN as i128);
            }
        }
        let pl = Self::expect(g, &w, &lambda);
        // F = α π + (1 − α)(Pᵀ φ + D π), D the dangling mass. Through P:
        //   (1 − α) φ_p (λ_q − (Pλ)_p) / out_p
        // through π (∂π_j/∂A_pq = (δ_jp + δ_jq − 2 π_j) / Σdeg):
        //   (α + (1 − α) D) (λ_p + λ_q − 2 ⟨π, λ⟩) / Σdeg
        let one = 1i128 << PHI_BITS;
        let dangling: i128 = w.out.iter().zip(&phi).filter(|(d, _)| **d == 0).map(|(_, f)| *f as i128).sum();
        let c = (ALPHA_NUM as i128 * one + KEEP_NUM as i128 * dangling).div_euclid(ALPHA_DEN as i128); // 2^60
        let mean = w.prior_dot(&lambda);
        (0..g.edges())
            .map(|e| {
                let (s, d) = (g.src[e] as usize, g.dst[e] as usize);
                let walk = (KEEP_NUM as i128 * phi[s] as i128 * (lambda[d] - pl[s])).div_euclid(ALPHA_DEN as i128 * w.out[s] as i128);
                let prior = (c * (lambda[s] + lambda[d] - 2 * mean)).div_euclid(w.w2 as i128);
                walk + prior
            })
            .collect()
    }
}
