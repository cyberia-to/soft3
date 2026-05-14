# soft3 — developer experience layer

## what this repo is

the soft3 SDK: language libraries, MCP server, CLI, and wire format schema for the [[soft3]] stack.

not the stack itself — a client layer on top of it.

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

## do not touch zones

- `js/` package.json dependency versions — discuss before changing
- `schema/` wire formats — changes must propagate to all language SDKs simultaneously

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

1. BBG `QueryProof` has no serde — `Commitment`/`Opening` are lens-internal types
2. query wire protocol not yet defined (`schema/` is a draft)
3. zheng accumulator size not yet stabilised (blocks checkpoint format)

scaffold readmes and command stubs are welcome. full implementations wait for blockers to resolve.
