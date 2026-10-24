---
title: proofs of protocols
tags: cyber, soft3, zheng, eidos, proposal
crystal-type: proposal
crystal-domain: cyber
alias: protocol proofs, completeness and soundness in eidos
status: draft
date: 2026-10-24
---
# proofs of protocols

**status:** draft · **date:** 2026-10-24 · **question (owner):** zippel answers "is this IOP complete" by Gröbner bases; do we become dependent on it? such checks are routine while developing — can we do it in eidos, and how do we guarantee soundness?

## 1. the picture

A proof system is three statements, and each has a different kind of proof:

| statement | kind | proved by |
|---|---|---|
| completeness: an honest run is accepted | a polynomial identity over the field, after substituting the prover's definitions and the relation | normalisation — a `ring` decision procedure; with hypotheses, a Nullstellensatz certificate checked by `ring` |
| soundness of the IOP: a false claim passes with probability ≤ ε | a counting theorem over the field: Schwartz–Zippel per round, degree bounds, a union bound | a library of lemmas about polynomials over F and one short theorem per protocol |
| soundness of the argument: binding of the commitment, Fiat–Shamir | reductions to named assumptions (hemera as a random oracle, the code's distance) | paper proofs of the reductions, conformance of the parameters; never "guaranteed", only made explicit |

zippel covers the first row by searching an ideal membership with Gröbner bases, and a fragment of the third (special soundness for protocols with a witness). The search is the expensive part; the certificate it finds is cheap to check. That is the whole design below: **search anywhere, check in eidos.**

## 2. dependency, answered

We depend on zippel for nothing in the stack and for one thing in the workflow: the oracle that says "complete" in 43 ms. The protocol files are ours (`zheng/specs/proto`), the discipline — one choreography, the transcript derived from it — is ours to keep, and the verdict can be re-derived by any Gröbner engine (Singular today) or, after this proposal, checked by eidos from a certificate. So zippel is a tool like a SAT solver: replaceable, never trusted for truth once the certificate is checked at home.

## 3. the moves

### 3.1 protocols as eidos terms

An inductive `Protocol` in eidos: instance and witness types over F, a list of moves (`send e`, `challenge`, `check e`), with expressions as terms over F. `prover`, `verifier` and `transcript` are functions of the term — the projection zippel does by reachability becomes two definitions by recursion over the moves. `transcript.rs` is then generated from the same term (the absorb/squeeze order, the domain tags), which is the one way to make "the transcript is derived, not maintained" true in our code.

### 3.2 `ring` by reflection — the tactic eidos needs anyway

Completeness of a protocol term is `∀ x w c, R x w → accepts (verifier x c (prover x w c)) = true`. After unfolding, each `check` is an equation between two polynomials in the free variables; without hypotheses the two sides normalise to the same canonical form. The tactic: a normaliser written as an eidos function over a reflected polynomial syntax, proven correct once (`eval (normalise p) = eval p`), applied by `decide` on the canonical forms. This is the standard proof by reflection; every field-arithmetic fact in cyber (hemera's rounds, the CCS encodings, the economy's fixed point) needs the same tactic, so it is not special to protocols.

With hypotheses (the relation `a·b = c`, or a verifier check from an earlier round), completeness is ideal membership: goal `g = Σ hᵢ·qᵢ`. The `qᵢ` are the certificate; the search for them is Gröbner (zippel, Singular, or our own later); the check is `ring` on `g − Σ hᵢ·qᵢ = 0`. Certificate size is small for everything in `zheng/specs/proto` (the ideals have tens of generators).

### 3.3 soundness of the IOP in eidos

The library: polynomials over F as lists of coefficients, degree, the root bound (a nonzero polynomial of degree d has at most d roots — a counting theorem, provable by induction), the multivariate Schwartz–Zippel, probabilities as counting over a finite challenge space (no measure theory: Pr is a ratio of counts over Fⁿ). The protocol theorem: if the verifier accepts and the claim is false, then in some round the sent polynomial differs from the true partial sum, and the next challenge is a root of their difference; the union over rounds gives ε ≤ Σ dᵢ / |F|. For SuperSpartan: the outer zero-check bound (the eq(τ,·) step), the batching with γ, and the inner sumcheck compose by the same union bound. This has been done in other assistants (sumcheck in Lean's ArkLib and in Isabelle); the shape is known, the size is thousands of lines of library and hundreds per protocol.

What this gives: a machine-checked ε for the IOP with oracle access to ẑ, which is exactly the statement zippel cannot make for us.

### 3.4 the argument: assumptions, named

The IOP becomes an argument through the lens (Brakedown now, WHIR in the family) and Fiat–Shamir with hemera. Neither is a field identity; both are reductions:

- the lens: binding and the proximity gap of the code, parameters checked by the lens's own vectors; the statement we rely on is written down with its constants (`lens/specs`), not re-proved here;
- Fiat–Shamir: round-by-round soundness of the IOP (which 3.3 gives, with the state-restoration argument) plus hemera modelled as a random oracle; the derived transcript guarantees the binding of every message and of the instance by construction, which is the condition the attacks on weak Fiat–Shamir exploit.

So "guarantee" means: every assumption has a name and a constant, every step between two assumptions is a theorem in eidos or a reduction in a paper we cite, and the executable spec is the thing the theorems are about.

### 3.5 implementation = spec

The theorems are about the protocol term, not about `rs/src/spartan`. The bridge today: the term and the Rust agree on every conformance vector and on the derived transcript. The bridge later: the verifier runs as a nox program and eidos reasons about that program (the kernel is already a nox program), closing the gap between "the spec is sound" and "the binary is sound". Until then the gap is explicit and tested, not hidden.

## 4. order

1. eidos: reflected polynomial syntax + `ring` by reflection, with its correctness theorem — the first decision procedure in `specs/tactics.md` beyond `omega`/`simp`. Blocks everything else, serves everything else.
2. eidos: `Protocol` terms, prover/verifier/transcript by recursion, the completeness statement; port `zheng/specs/proto` to terms; completeness of `zheng_inner`, `zheng_outer`, `zheng_spartan_fixed` by `ring` (+ Nullstellensatz certificates searched by zippel/Singular, checked by `ring`). `transcript.rs` generated from the term.
3. eidos: the polynomial library and the sumcheck soundness theorem; then SuperSpartan's bound.
4. zheng: the gate switches from "zippel says complete" to "eidos checks the certificate"; zippel stays as a search engine and as a second opinion.
5. the argument layer: the assumption page (lens constants, hemera as random oracle, the round-by-round condition) with the reductions cited; the nox verifier and its proof are the phase after.

## 5. acceptance

- `ring` proves `(a+b)² = a²+2ab+b²` and the hemera round identities in eidos; the normaliser's correctness theorem is in the kernel's trusted output, not assumed;
- completeness of the three protocols is an eidos certificate checked by the kernel, and the planted bug (`zheng_bug`) has no certificate — the search fails and no `ring` proof exists;
- the sumcheck soundness theorem states ε ≤ Σ dᵢ/|F| and is accepted by the kernel; instantiated at Goldilocks and the zheng sizes it reproduces the bound on `specs/superspartan.md`;
- `transcript.rs` is a generated file with the protocol term as its single source.

related: [[zheng/docs/explanation/zippel|zippel]] · [[zheng/audit/zippel-2026-10-24|the experiment]] · [[eidos]] · [[soft3/proposals/proven-present|the proven present]]
