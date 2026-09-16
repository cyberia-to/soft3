# P10 consumer compatibility — 2026-09-13

The [consumer contract](../../specs/identity-consumers.md) separates the current
native action profile from foreign networks, transport keys and historical
unsigned observations. This report records local source/build evidence. It does
not assert that a published package or remote endpoint has been upgraded.

| Consumer | Implemented/retained behavior | Evidence in implementation-baseline/ |
|---|---|---|
| mudra | unchanged domain derivation/address/native ID/ADR-036 and NSIG1; public minimal NeuronId re-export | `p10-mudra-final.log`: 22 tests |
| lytics-event | verified signature returns the same H(pubkey) used by ingestion | `p10-lytics-event-final.log`: 23 tests; frozen domain fixture |
| lytics-ingest | replay preserves verified native author; old address-only records remain legacy analytics history; validates readable records before replacing projection | `p10-lytics-ingest-tests.log`: 49 tests, including restart/sequence and corruption |
| cyberia-my, cyberia/research/cyberia-my | same domain/HRP/format/storage names; pure stored-envelope verification | `p10-cyberia-my-domain.log`, `p10-cyberia-research-domain.log`: one fixture test each; both WASM builds passed |
| soft3/js | independent CosmJS verification of old Rust signatures; Cosmos HD address, MsgCyberlink/amino and MsgExecuteContract bytes unchanged | `p10-js-build.log`, `p10-js-compat-tests.log`: build and 3 offline tests |
| trisha/Neptune | selected upstream wallet/network/address lifecycle retained | `p10-neptune-wallet-tests.log`: 8 process tests; no trisha source edits by this migration |
| true-cyber | common explicit custody/registry, signed native link/relay/receipt, per-network mirror, strict old-log import and exact export | `p10-true-cyber-tests.log`: 4 real-process scenarios against actual soft3 HTTP nodes |
| inf | unchanged native Hash representation plus NeuronId constructor; public BBG balance scope corrected | `p10-inf-workspace-tests.log`: 99 tests; optional BBG source/value tests: 24 |
| neuron-node/query | actual canonical execution snapshot → immutable inf progs/invocations/operations/origins with subject/network/commit scope | `p10-neuron-query-tests.log`: actual runtime, unknown effect, independent prog, revoke/reopen and snapshot preservation |
| soft3 optional stack facade | resolves current sibling cyb facade, whose graph owner is cyb-core | `p10-soft3-stack-check.log`: feature build passed |

Frozen domain outputs were captured by compiling the original mudra primitives
at `348c46195388ac009667144987511a0e33bdc859` in
`legacy-domain-capture/`. Its provenance records original source SHA-256;
`domain-vectors.json` records exact keys, addresses, native IDs, sign documents
and signatures for public fixture inputs. Copies are checked by actual consumer
code and independently by CosmJS. No production secret is in these fixtures.
The two browser checkouts are separate source roots with the same identity
profile; both were migrated and tested, neither treated as a generated copy.

The WASM build initially selected Homebrew rustc without WASM stdlib. Explicit
rustup stable RUSTC/RUSTDOC resolved this environment error. The actual app builds
passed in `p10-cyberia-wasm.log`, `p10-cyberia-research-wasm.log`; their dependency
trees are retained separately. The event core WASM build passed as well.

[check_identity_consumers.py](check_identity_consumers.py) constructs isolated
Cargo roots, audits normal/build dependency edges for cycles and forbidden
runtime/storage/GUI/inference crates, and compiles each on WASM. All seven
profiles passed: neuron-id, model without defaults, model+serde without defaults,
mudra without defaults, lytics-event without defaults, inf-value and inf-source.
`identity-consumers/results.json` records compiler, features and full package sets.
No dev-dependency/workspace feature unification is misreported as SDK weight.

Full graph products may opt into runtime/storage dependencies. True-cyber now
does so through the common host; it is not marketed as a two-dependency probe.
Its local source manifest and soft3 now declare the tested Rust 1.95 baseline,
instead of stale 1.74/1.85 claims. Inf's old local BBG/nox/zheng version constraints
were aligned with the current owners so its actual workspace resolves; its
existing default workspace suite passes without RUSTC_BOOTSTRAP.

Foreign protocol owners remain unchanged at their wire/storage boundary:

- `cybernet/cw/src/{uids,neuron_info,state}.rs`: netuid/UID is a reassignable slot;
  hotkey/coldkey Addr and stake ownership are contract semantics, not CellId.
- `hub/contracts/hub-skills/src/{state,msg}.rs`: numeric entry ID, owner Addr and
  neuron/network/protocol/endpoint strings retain their original indexed storage.
- `portal/packages/cyber-std/src/types.rs`: NeuronBandwidth and Thought/program
  fields remain CosmWasm schema; no native subject cast was added.
- `go-cyber`, `space-pussy`, `bostrom`, `bootloader` retain Cosmos addresses,
  genesis/snapshots and historical encodings. JS bridge fixtures exercise the
  used protobuf/amino boundary; this is not a new full-chain release claim.
- `midao/contracts` retains EVM/LayerZero token/address semantics. Collective
  agent documentation is handled by P11, independently of deployed contract data.
- Radio iroh endpoint/CID fields, tape framing/table cells and foreign protocols
  stay unchanged. New owner documentation separates transport and action authority.
  Patch documentation now distinguishes explicit neuron/network/grant from
  prog/task/data IDs and prevents CRDT claims from implying multiple valid writers.

The old lytics encryption/erasure store still skips unreadable erased ciphertext
under its preexisting profile. Only successfully decrypted/readable records can
be provenance-validated; this report does not claim a newly hardened key-erasure
service or recovery of signatures never stored. Old true-cyber log authors likewise
stay historical; migration does not infer custody/genesis from their label hashes.

This evidence closes the supported P10/G11 consumer profile. Final integration,
feature/revision freeze and the whole-plan audit remain P12/P13 work. P07's
remaining migration operator/fault checks and P11's domain rewrite are tracked in
[the implementation ledger](implementation.md), not silently removed from scope.
