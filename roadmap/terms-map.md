---
title: terms map — the rename contract
tags: cyber, soft3, roadmap
crystal-type: spec
crystal-domain: cyber
status: active
alias: rename contract, terms refactor, vocabulary sweep
---
# terms map — the single rename contract

the authoritative old→new map for the stack-wide terms sweep. source of
truth is [[terms]]. every repo follows this exactly; nothing here is
discretionary. when a word is context-dependent (`cell`, `Cid`), the KEEP
rules below decide — do not guess.

## rename (everywhere — texts and code)

| old | new | notes |
|---|---|---|
| `noun` | `data` | the value type. lowercase word, comments, docs. |
| `nouns` | `data` | already collective; not `datas`. |
| `Noun` | `Data` | the type. |
| `Noun::Atom` / `Noun::Cell` | `Data::Atom` / `Data::Pair` | the variants. |
| `Noun::cell(` / `order.cell(` / `.cell(` | `.pair(` | the substrate join constructor. |
| `NounId` | `Order` (`order`) | a data node's *local* identity — its slot in a `reduction`. type `Order` (was `OrderId`). |
| `Order<N>` (the arena type) | `Reduction<N>` | the run/execution that holds all data; was the `Order` arena. |
| `ContentId` | `particle` / `Particle` | the *global* content identity. |
| `Cid` | `particle` / `Particle` | EXCEPT external IPFS CIDs (see KEEP). |
| `hash_noun` | `hash_data` | + `read_hash_noun → read_hash_data`, `nox_hash_noun → nox_hash_data`. |
| `parse_noun` / `print_noun` | `parse_data` / `print_data` | |
| `*_noun` locals | `*_data` | `root_noun → root_data`, `formula_noun → formula_data`, etc. |
| `cell_a` / `cell_b` / `root_cell` / `cell_dig` | `pair_*` | substrate pair locals/tests. |
| `crate::noun` / `mod noun` / `rs/noun/` | `crate::data` / `mod data` / `rs/data/` | nox module dir + path. |
| `U32` (surface value) | `Word` | trident surface only (ast/typecheck/lsp); NOT `Ty::U32` lowering. |

three distinct identities/types, never conflate:
- `reduction` (`Reduction`) — the run that holds data.
- `order` (`Order`, u32) — a data node's local handle within a reduction.
- `particle` (32 bytes) — a data node's global content identity.
and `Word` (a `field` proven in `[0, 2³²)`) is a value refinement, unrelated to `order`.

## keep (do NOT touch — different concept, same spelling)

- **`cell` (Rust std):** `RefCell`, `OnceCell`, `UnsafeCell`, `LazyCell`, `Cell<T>` — everywhere.
- **`cell` (WASM stack):** wysm `Cell(u64)`, `CellError`, `Cells` — all ~100.
- **`cell` (DAS / erasure / polynomial grid):** hemera, bbg, zheng, mir.
- **`cell` (memory/storage):** bbg `unimem` cells.
- **`cell` (hierarchy/shard):** cyber `hierarchy.md` / `cell.md` / `3c.md` — a first-class domain entity.
- **`Cid` (external IPFS / iroh / CAR):** radio's 26 `Cid` — the `cid` crate, not a cyber identity.
- **`digest` / `Digest`:** at trident's target boundary (`[Field; D]`) — stays `digest`; core uses `particle`.
- **`subject` / `object`:** actor (neuron) and data — never repurposed as the two halves of a pair/link.
- **nika:** out of scope entirely — its "noun" is Nockchain's (Urbit jam), and lives only in `target/` build artifacts.

## scope (clean counts, excl. `target/`/`.git/`/`node_modules/`)

| repo | noun→data | NounId→OrderId | Cid→particle | cell→pair (subset) |
|---|---|---|---|---|
| nox (source) | 246 | 22 | 1 | substrate only |
| trident | 281 | 457 | – | `Noun::cell`, `order.cell` |
| rune | 487 | – | – | `Noun::cell/Cell` |
| eidos | 80 | 10 | – | `Noun::cell` |
| cyber (docs) | 57 | – | – | keep hierarchy cells |
| soft3 | 32 | 9 | 8 | – |
| cyb | 19 | – | 4 | – |
| crystal | 17 | – | 4 | – |
| lens | 13 | – | – | – |
| hemera | 8 | – | – | keep DAS cells |
| bbg | 7 | – | 3 | keep unimem/DAS |
| zheng | 6 | 2 | – | keep DAS |
| cybics | 5 | – | – | – |
| inf | 4 | 18 | – | `.cell(` → `.pair(` |
| cybergraph | 4 | 10 | 2 | `order.cell` → `order.pair` |
| radio | – | – | keep (IPFS) | keep |
| wysm | 1 | – | – | keep all (WASM) |

## order of execution

1. **nox** — source of truth; finish internal identifiers + `rs/noun/`→`rs/data/`. public types (`Data`, `OrderId`, `Particle`) already final.
2. **code consumers** (parallel, non-overlapping repos; each verifies `cargo build`/test green): trident, rune, eidos, inf, cybergraph, zheng, bbg, lens, cyb.
3. **text/docs** (no build risk): cyber, soft3, crystal, cybics.

verify per repo: build + tests green; no `noun`/`NounId`/`ContentId` left except KEEP cases and historical quotes in roadmap/migration notes.
