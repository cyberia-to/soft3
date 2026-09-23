# soft3 — stack contracts and developer interfaces

## what this repo is

soft3 owns foundational stack composition contracts and developer interfaces:
language libraries, MCP server, CLI and wire format adapters. Component
algorithms remain in their owning repositories.

`specs/execution-model.md` is the normative execution architecture.
A warrior supports a VM/OS family across an open-ended set of compatible
network instances; workers and hardware backends are independent deployment
choices. `specs/README.md` indexes contracts. Keep the root README and site
navigation linked to this foundation when changing architecture.

## components

| dir | what | status |
|-----|------|--------|
| `js/` | JavaScript/TypeScript SDK (Bostrom chain, CosmJS-based) | active |
| `schema/` | canonical wire format definitions | draft |
| `cli/` | `soft3` command-line tool (Rust) | scaffold |
| `mcp/` | MCP server for AI assistant access | scaffold |
| `py/` | Python SDK | scaffold |

## dependency chain

soft3 SDK depends on: hemera (particle), bbg (state + proofs), lens (verification), cybergraph (signal submission), radio (transport)

do NOT implement stack logic here. call into the stack repos via FFI, WASM, or RPC. the SDK is an adapter, not a reimplementation.

## release train

rules: `~/cyber/cyberia/dev.md` § release train. soft3 rides the weekly
train with cyb and cyber. a candidate is cut every friday 12:00 UTC by an
agent from `origin/main` only. gates: `cargo test` in `crate/`, the node
boots and answers `/status`, the conformance snapshot passes. one bump is
one PR `chore: soft3 <version>` (Cargo.toml, CHANGELOG, sibling pins in cyb
and cyber). `release/phase1.toml` is the phase-1 manifest: the sibling
revisions every candidate closes over; drift from a pin is a red gate and
is fixed by a bump PR, never by a path edit. receipts in
`audit/release-<date>/`; one row per candidate in the launch page work log.
agents cut, gate and open bump PRs; only the owner merges a bump, promotes
a candidate, publishes to crates.io or tags.

## do not touch zones

- `js/` package.json dependency versions — discuss before changing
- `schema/` wire formats — changes must propagate to all language SDKs simultaneously

## filenames (Windows / NTFS)

tracked paths must be valid on NTFS. never use `<>:"/\|?*`, trailing `.` or space, or reserved device names (`CON`, `PRN`, …).

- policy: [[filenames]] → `specs/filenames.md`
- check: `python3 scripts/check-filenames.py --root ~/cyber`

## companion repos

| repo | role |
|------|------|
| `cybergraph` | signal submission, local sync, query protocol |
| `bbg` | state machine, polynomial commitments, query proofs |
| `hemera` | particle computation (Poseidon2 hash) |
| `lens` | polynomial commitment backends |
| `radio` | P2P transport |
| `foculus` | consensus |

## current blockers (do not implement past these)

1. ~~BBG `QueryProof` has no serde~~ — RESOLVED 2026-09-07: serde behind a `serde` feature in lens (Commitment/Opening) and bbg (QueryProof), canonical point codec, golden wire fixtures; schema seed at `bbg/docs/api/query-proof-wire.md`
2. query wire protocol not yet defined (`schema/` is a draft)
3. zheng accumulator size not yet stabilised (blocks checkpoint format)

scaffold readmes and command stubs are welcome. full implementations wait for blockers to resolve.
