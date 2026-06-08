---
title: types & terms migration
tags: cyber, soft3, roadmap
crystal-type: spec
crystal-domain: cyber
status: draft
alias: vocabulary migration, terms migration, type cleanup plan
---
# types & terms migration — execution plan

the stack-wide cleanup of the type design and vocabulary. this is the
careful execution plan; the *what* lives in four source docs, this is the
*how* and the *order*. it touches every repo, and a subset of it changes
particle identities — so it must be sequenced, not done piecemeal.

## source of truth

- `soft3/specs/types.md` — the type theory (five mechanisms; two open
  systems: refinement + nominal).
- `soft3/specs/terms.md` — the vocabulary (one name per concept).
- `nox/roadmap/data-model.md` — the nox value layer (atom/pair/data/particle,
  Model B, encoding, identity).
- `hemera/roadmap/one-pure-hash.md` — capacity invariants-only, modes removed.

## the canonical renames

| old | new | scope |
|---|---|---|
| `noun` | `data` | nox specs + code, all consumers |
| `cell` | `pair` | nox specs + code |
| `NounId` / `ContentId` | `particle` | nox + bbg + consumers |
| `Cid` | `particle` | inf (done), any stragglers |
| `digest` (core) | `particle` | core only; stays `Digest` at trident target |
| `U32` (surface) | `Word` | trident surface + inf |
| `Tag` (field/word atom tags) | removed | nox |

plus structural changes: tag-free leaf encoding (Model B); drop the
`data[8]` framing byte; hemera drops keyed/derive modes.

## project-wide audit — what the sweep found

A five-agent read-only audit across ~30 stack repos. Headline: the
canonical terms (`particle`, `cyberlink`, `neuron`) are already used
consistently stack-wide (bbg, cybergraph, mir, tru, inf, zheng, hemera).
The rename surface is concentrated, and `cell` is genuinely
context-dependent.

### noun → data, NounId/ContentId → particle (propagate from nox)

Anchored in nox (`noun` ~271, `NounId` ~158). The TYPE rename
`nox::NounId → nox::Particle` cascades through the extern API to every
consumer: trident (~457 `NounId`), rune (~101), eidos (~22), inf (~17),
zheng/cybergraph/bbg (few). `noun` as a word: trident ~176, rune ~429,
eidos ~44, plus doc references in bbg/hemera/lens/cyber. Rename nox
first, rebuild downstream — the type rename does most of the work.

### cell → pair — CONTEXT-DEPENDENT (the careful one)

Rename ONLY the nox substrate pair. Keep every other `cell`.

RENAME → `pair`:
- nox: `cell(noun,noun)` in `specs/noun/`, `reduction.md`, `encoding.md`
  (~95); `Order::cell()`, `Noun::Cell`.
- consumers constructing the substrate pair: cybergraph (~29
  `order.cell()`), bbg (~7), inf (~18 `o.cell()`), rune (~101
  `Noun::cell`), eidos (~22), trident (~46), sync (1), cyber substrate
  docs (~12).

KEEP (different meaning — do NOT touch):
- cyber `cell` = the hierarchy/shard entity (geographic/semantic/economic/
  social shard) — ~57 in `hierarchy.md`/`cell.md`/`3c.md`. a first-class
  domain concept. (renaming the substrate to `pair` actually REMOVES this
  collision — a win.)
- DAS / erasure / polynomial-grid cells: hemera, bbg, zheng, mir (~25).
- memory/storage cells: bbg `unimem` (~8).
- WASM stack `Cell(u64)`: wysm (~40).
- terminal grid `Cell`: nu (~414, out of scope).
- Rust `std::cell` (RefCell …): everywhere.
- FHE / tower `pair`: strata (already `pair`).

### Cid → particle — essentially done

- radio: 24 `Cid` are all external IPFS/iroh CIDs (the `cid` crate, CAR
  format). KEEP, qualified — not cyber identities.
- cyber: already migrated (the rename was applied across eight repos).
  ~0 left.

### U32 → Word — trident surface only

~92 surface sites (`ast`/`syntax`/`typecheck`/`lsp`) → `Word`; ~14
lowering sites (`ir`/`cost`) keep `U32` (Triton's name); `Digest` (~115)
stays at the target boundary.

### polish (not blocking)

- cybergraph specs use "edge" in prose (~20) where `cyberlink` is meant;
  code is correct — standardise the prose.
- cyber research files (`polynomial nouns.md`, `bootstrap.md`) still say
  `noun` — historical; sweep with the rest.
- `particle` (identity) vs `name` (mutable label) is respected everywhere
  (whitepaper: "names are metadata, the hash is the truth"). no fix.

## two kinds of change — keep them apart

1. **renames** — safe, mechanical, compile-checked. Cross-repo but
   reversible; a rename never changes a hash.
2. **identity-affecting** — tag-free encoding, `data[8]` framing drop,
   `capacity` changes, hemera mode removal. These change every `particle`
   and every conformance fingerprint. They are a one-time hard fork of
   identity and MUST land together in a single epoch, then re-baseline
   conformance once.

Do all of (1) first; it stabilises the vocabulary with zero identity risk.
Then do (2) as one coordinated epoch.

## repos touched

nox, hemera, zheng, bbg, tape, trident, inf, cybergraph, radio, mudra,
conformance, soft3. (cyber graph docs reference the terms but rebuild
from the specs.)

## phases

### Phase 0 — freeze the names (no code)

Sign off the renames table and the two open names already chosen
(`data`, `pair`). Nothing builds until this is fixed — every later phase
depends on it. Gate: author approval of `terms.md`.

### Phase 1 — spec sweep (docs only, no build risk)

Update every `.md` spec to the new vocabulary, spec-before-code. Per repo,
independent, parallelisable by directory:

- nox: `specs/noun/` (→ rename concept; the dir name too), `encoding.md`,
  `reduction.md`, `vm.md`, `trace.md`, `object.md`. resolve the Model A/B
  duality in favour of B; fix the `capacity[14]` drift in `noun/hash.md`.
- hemera: `noun/hash.md` callers, capacity docs; land
  `one-pure-hash.md`’s lane layout into the reference.
- inf, trident `reference/`, cybergraph `specs/`: vocabulary only.

Verify: each repo’s docs read consistently; no `noun`/`cell`/`Cid`/`NounId`
left except where quoting history.

### Phase 2 — code rename sweep (mechanical, compile-checked)

In dependency order, rebuild downstream after each:

1. nox: `Noun → Data`, `NounId/ContentId → Particle`, `cell → pair`,
   remove `Tag` (collapse field/word). `cargo check` + tests.
2. consumers of nox types — bbg, cybergraph, zheng, tape, soft3 sdk —
   absorb the renamed types, rebuild.
3. trident: `U32 → Word` surface (`ast/mod.rs:193`, `typecheck/types.rs:13`
   + ~60 `Ty::U32`; leave `cost/scorer.rs const U32`; `Digest` untouched).
   add the lowering note. keep the builtin-sync rule (reference + typecheck
   + tir + cost in one commit).
4. inf: one site.

No hashes change in this phase. Verify: full-stack `cargo check` + tests
green; conformance fingerprints unchanged (proof that Phase 2 is
identity-neutral).

### Phase 3 — encoding + identity (the identity epoch begins)

nox only, contained:

- `nox/rs/encode.rs` — leaf-based, tag-free; sizes `8·N`; drop the tag
  byte; reference-by-particle. (bbg/tape decoupled — no change.)
- `nox/rs/noun/hash.rs:26-27` — drop `data[8] = tag` and the tag arg.

This changes particle identities. Verify: tests green; expect conformance
drift (do not re-baseline yet — batch with Phase 4).

### Phase 4 — hemera one pure hash (identity epoch)

- move radio’s handshake KDF/MAC to mudra (`radio/iroh-relay/.../handshake.rs`).
  gate: confirm mudra exposes KDF + keyed-MAC.
- delete `keyed_hash`/`derive_key` and the mode constants (hemera
  `lib.rs`, `sponge.rs`, cli). `state[11]` frees, stays empty.

Verify: radio handshake works against mudra; hemera is `hash` only.

### Phase 5 — re-baseline identity (close the epoch)

Once Phases 3–4 are in, recompute the conformance fingerprints in one
commit (`conformance`): one hemera fingerprint per encoding & mechanism.
After this, identity is frozen again on the new scheme.

### Phase 6 — word as a refinement (soundness, spec-first, gated)

The one piece needing real proof-system work; sequenced last because it is
independent and the riskiest.

- spec-first: resolve the `shl` conflict (`trace.md` vs
  `constraints.md:310-321`); specify `lt` 64-bit canonical form.
- zheng: add the decomposition-binding + booleanity constraints
  (`ccs/patterns.rs`), via a running-accumulator column (`ccs/mod.rs`).
- nox: retire the `Tag::Word` runtime gate (`reduce.rs:366-411`); `word`
  becomes a typing-only refinement.

Verify: word-gated ops sound in-circuit (the binding *is* the range
proof); bench no regression.

## verification gates (every phase)

per the stack’s own rules: `cargo check` zero-warning, tests green,
`trident audit` where applicable, conformance unchanged (Phases 1–2) or
re-baselined once (Phase 5). a phase that fails any gate is fixed before
the next begins.

## open gates

1. name sign-off (`data`, `pair`) — Phase 0.
2. mudra exposes KDF + keyed-MAC — blocks Phase 4.
3. `shl` spec conflict resolved — blocks Phase 6.
4. identity epoch timing — Phases 3–5 must be one uninterrupted batch; do
   not ship a half-migrated identity scheme.

## why this order

renames first (zero identity risk, stabilises vocabulary) → identity
changes batched into one epoch (so conformance re-baselines once, not
repeatedly) → soundness work last (independent, gated, spec-first). each
phase is independently verifiable, and nothing downstream of an identity
change ships until Phase 5 closes the epoch.
</content>
