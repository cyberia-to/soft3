# Neuron convergence — implementation ledger

User authorized the complete roadmap on 2026-09-12, including migration and a
final plan-compliance audit. This file preserves work between context windows.

## Scope and baseline

- Task: `roadmap/neuron-cell-convergence.md`, P00–P13 / G01–G14, in full.
- Preserve existing dirty work, native coordinator changes and legacy bytes.
- No push/deployment requested. Local source/repository and data migration is authorized.
- New inventory: `implementation-baseline/scan.json`; original audit retained.
- Fresh scan: 115 Git markers (108 valid roots, 6 generated, one broken nu),
  six non-Git groups; 28,928 text files. Same sparse Omi gap as original audit.
  cve-legacy-local and nested docs were removed by other work before this run.
- Initial `cargo test --workspace --offline --locked` in cell failed because
  the lock no longer resolves the changed companion workspace. No tests ran.
  Refresh the local lock and establish runtime fixtures before engine changes.
- Refreshed baseline: `cargo test --workspace --offline` passed all 15 tests.
  Fjall dependency emitted its pre-existing dead-code/lifetime warnings.
- `implementation-baseline/fixtures/legacy-v1.capture` contains 3,130 canonical
  contents and 15 successful graph writes from the unmodified v1 engine: completed
  counter (=7), unknown external attempt, and lost-receipt reservation (=1000).
  Generator is preserved as `implementation-baseline/capture-legacy-v1.rs`;
  manifest and actual legacy BBG store remain beside the capture.
- Local repository moved `cell` → `neuron`, preserving .git. Six packages now
  neuron-id/model/engine/rune/node/cli. Renamed workspace passed the same 15 tests
  before introducing the new identity module (new module tests tracked next).
- Shared NeuronId now re-exported by BBG and tok; mudra native types refer to it.
  Mudra optional path constraints had stale nox/zheng versions; aligned constraints
  to actual 0.3/0.4 companions without changing cryptographic algorithms.
- Identity-only model uses `--no-default-features`; Cargo tree has only neuron-id.
  Added domain/network-qualified references, bounded attachment projection,
  immutable ActionContext and typed Destination. Runtime semantics still v1;
  P05/P07 must replace that implementation before any merge completion claim.

## Packages

## Runtime implementation checkpoint

- New `neuron_model::execution` suite and bounded maps implemented: progs,
  invocations, per-job continuation/effect state, state revision CAS, aggregate
  charged/held budget, parent/child transfers, cycle/depth checks, archival.
- New `neuron_engine::Neuron` API implemented: activate/install/admit/tick,
  pause/retire/upgrade, effect attempt/result/failure, event wake, cancel/archive,
  policy epoch/rebind, explicit legacy import planning/activation.
- `neuron_node::LocalAuthority` uses opaque KeyVault custody and actual mudra
  ADR-036 signatures over canonical bound statements. Watch-only has no authority.
  This is the supported native H(pubkey) profile, not a proof-native implementation.
- `cargo test -p neuron-node --test neuron_runtime --offline`: five scenarios
  passed (multiple progs/jobs and stale state; unknown tool/restart/single result;
  child transfer/join; upgrade; subject/network/current grant denial).
- `cargo test -p neuron-node --test legacy_capture --offline`: fixed original
  v1 contents/history/checkpoints/unknown attempts validate and read unchanged.
  Generator label `completed_request` is the admission **commit**, not its lookup
  key. Legacy request keys bind the private owner particle + nonce; reader preserves
  that exact rule instead of substituting origin or NeuronId.
- BBG ApplicationStore now atomically validates source heads, publishes target,
  and installs immutable source fences; old exact receipt retries remain readable.
  `neuron-v1` completed-generation marker is accepted by new Database open and
  rejected by the original pre-migration executable. Migration fingerprint binds
  source heads + manifest in cybergraph. No source history is deleted.
- Original compiled CLI retained locally at
  `implementation-baseline/legacy-cell-binary` for old-reader rehearsal (machine
  artifact; do not commit the executable). Test uses `NEURON_LEGACY_BINARY`.
- BBG regression: application_storage (5) + shared_application (8) passed with
  `cargo test --offline --features backend-ssd --test shared_application --test application_storage`.
- `neuron_migration` passed both tests with `NEURON_LEGACY_BINARY` pointing to the
  original executable: 3 origins→1 subject; old executable rejects migrated open;
  new writer fences; old heads/claims unchanged; unknown result reconciles once;
  lost reservation charges 1000; stale source/target activation rolls back.
  One initial fixture assertion expected the inner migration diagnostic, while
  the old CLI intentionally exposes Storage(Corrupt); corrected to its actual API.
  The three-live-origin scenario requires historical charges **in addition** to
  held allowances; its test target budget was corrected from 3M to 4M.
- CLI now uses explicit native identity and program/invocation arguments. Keygen
  is explicit/exclusive/0600; --key-file selects custody and --grant-act supplies
  current rights. Legacy inspect/history/import commands preserve provenance.
  `cargo test -p neuron-cli --offline`: 6 process scenarios + 3 storage guards pass.
- `cargo clippy -p neuron-model -p neuron-engine -p neuron-node --all-targets
  --offline --no-deps -- -D warnings` passed after grouping bound action context.
  Companion Fjall's three pre-existing warnings remain dependency warnings.
- Mudra default tests passed (18 unit + 2 bridge vectors); no-default mode passed
  15 unit tests. Identity-only model passed three tests and has only neuron-id
  in the normal dependency tree.

- Retired the old Engine/LocalWard API and standalone writer implementation.
  Retained only pure v1 readers/profile IDs under `engine/src/legacy/records.rs`.
  Old runtime tests were superseded by new multi-program tests; recovery tests
  now exercise the new signed engine: four scenarios passed, including lost
  admission receipt, lost reservation, two sequential effects, failed/lost-receipt
  migration activation. Test-only legacy successor framing replaces old Engine
  usage in migration tests. No normal cell executor remains.
- Full neuron workspace passed **28 tests** after the CLI/old-engine retirement.
  Subsequent graph-boundary verification passed its two additional negative
  tests (fabricated evidence and a changed root with the old valid signature),
  plus all seven runtime/migration scenarios. Native publication now independently
  verifies the signature, exact proposed root/event, subject/network/policy/epoch,
  previous root and non-decreasing charges at the node graph adapter.
  This does not yet close dispatch-time revocation/worker fencing requirements.

Still required: validate authority at node publication/executor boundary,
complete revocation/worker/event/nested-budget tests,
external-store staging and exact retry/conflict/corruption migrations, domain/UI/
GraphSession/agent/Rs/SDK/docs migration and all final roadmap gates.

## Package status

### P04 baseline and seams

- `cyb/core/src/cell.rs` still owns a Cybergraph plus append-only tape file and
  silently discards replay/commit failures. `crates/cyb/src/cell.rs` is a duplicate
  in an excluded old crate. Replace these with GraphSession, never with a neuron.
- `cybergraph::native::NativeNode` is now implemented and must be reused for durable
  native publication. It pins genesis/profile, verifies contiguous recovery,
  has strict/resumable legacy tape import and preserves original wire prefix.
  `open_database` is private; a shared-Database constructor is a required seam.
- Cyb MoneyWallet reads and sometimes directly mutates `cell.graph.bbg` (test
  funding, settle credits, finalize_block). These writes need careful adaptation
  to the common publication boundary; preserve money/proof behavior, not a
  read-only facade with silently lost wallet writes.
- Shell SharedCell::open_default and cy CLI currently fall back to ephemeral
  state on storage failure. Remove false durable success while adapting callers.
- Before any P04 implementation, `cargo test -p cyb-core --offline` refreshed the
  stale companion lock and ran 24 tests: 23 passed; existing
  `money::tests::light_fold_tip_open_and_advance` failed with OpeningUnverified at
  core/src/money.rs:1040. Investigate current BBG/query/Tip certificate semantics;
  do not weaken proof verification to make this pass.
- Baseline diagnosis refined against state.md: dimension 10 is the opt-in
  **public plaintext** balance map, independent of private A/N commitments.
  Added explicit `prove_public_balance` disclosure API and `verify_public_balance`
  binding root/owner/token and both u32 limbs of u64. Existing prove_balances and
  open_cell remain contextless for balances; no implicit disclosure change.
  Certificates accept explicit namespace 10, never A/N. Nox look remains 0..9.
  BBG public_balance (1), query_authentication (6), state_certificate (4) pass;
  includes high-limb mutation and unchanged private/legacy disclosure checks.
  Wallet now verifies against its trusted tip and returns the authenticated u64.
  root_leaves_hash now encodes all actual leaves; no change to foculus trust claims.
- P04 naming started after the baseline: `core/src/cell.rs` moved to
  `graph_session.rs`, core facade exports GraphSession, Shell SharedGraphSession
  holds `session`, and cy/neural callers use the new name. Noun::Cell stays intact.
  The excluded publication crate now forwards graph_session/money/chroma/intent/
  sense/signal to cyb-core, removing the duplicate implementation; companion path
  constraints aligned for those shared types. Backend behavior is still the old
  signal-file implementation until the NativeNode facade is completed.
  Storage error handling and the full UI/registry migration remain open.

### P04 implementation checkpoint (shared coordinator)

- GraphSession now owns NativeNode, exposes graph read-only and a cloned shared
  Database for neuron ApplicationGraph. Ephemeral explicitly uses a disposable
  temporary database, exercising the same coordinator. No parallel RAM writer.
- NativeNode gained from_database/database seams and typed LocalCredit events.
  BBG NativeChange captures/checks balance+focus adjustments in its existing undo
  transaction. Canonical history binds reason and exact request; old event kinds
  unchanged. Host-only local credits do not claim network issuance authority and
  are not accepted by peer signal decoders. Credit receipt survives restart.
- GraphSession open strictly imports a legacy file into adjacent .bbg storage;
  preserves original bytes, checks identical source on subsequent opens, rejects
  malformed/truncated logs. Signal receive distinguishes duplicate/equivocation/
  gap/error, batches validate framing before applying and report durable prefix.
  CGVIEW v1 signal snapshots retain complete native proof/network bytes; legacy
  tape reader remains explicit. Full economic history remains NativeNode history.
- Wallet no longer mutates graph.bbg: bootstrap/settlement and block advance go
  through coordinator; settle reason prevents repeated credit. Restore rebuilds
  local settlement/tok projection from history; original mint height retained.
  Failed writes do not advance wallet mint projection. Private note custody and
  mature-notification persistence remain part of later vault/agent integration.
- Cyb shell and CLI no longer fall back to ephemeral on open failure. Chain-sync
  retains cursor after import errors. UI/CLI propagate funding/finalize failures.
- `cargo test -p cyb-core --offline`: **24 unit + 4 integration** pass after all
  above changes (log implementation-baseline/cyb-core-tests.log).
  Integration: real neuron + native publication into one DB; stale coordinator
  CAS; legacy multi-neuron log import/reopen/source preservation/corruption;
  wallet credit retry/conflict/overflow/reopen; full proof+network signal snapshot.
- `cargo check -p cyb -p cy --offline` passed (cyb-check.log). Two earlier attempts
  encountered actively changing glia gated_delta signatures; those were fixed
  by concurrent work, which was preserved. Latest wallet restore changes still
  require another shell check before final gate.
- Native unit fault tests passed all 3, including injected failure after staging
  LocalCredit and exact retry on reopen. Native integrations passed applications
  (5), native_import (11), native_storage (10); log cybergraph-native-tests.log.
  Excluded publication facade built, and neural CLI passed its 3 tests. Facade
  receive subsequently changed to strict parsing/error propagation and correct
  duplicate handling; its final build is being repeated.
- Full cybergraph test run currently fails to compile old stack_* tests: obsolete
  nox Order/Tag/OrderId APIs and zheng Statement/LookOpening shape (log
  cybergraph-tests.log). Do not delete/weaken those tests; adapt during final
  compatibility audit. Native-specific tests compile separately.
- Remaining P04/consumers: source-path retirement against old log writers; shared
  particles.jsonl content writer migration (soma and shell); published facade and
  neural checks, relay immutable subject/network behavior, final failure audit.

### P06/P07 current-grant activation checkpoint

- Authority::with_current is an explicit atomic policy guard, defaulting to
  Unsupported. LocalAuthority checks its GrantHandle under a read lock and holds
  that lock through the callback; revocation serializes on the corresponding
  write lock. ImportPlan retains the imported allowed-act union.
- activate_import rechecks target head and current grant through this guard
  around the source/target atomic migration transaction. A prepared signature
  cannot bypass later revocation. Exact already-committed import retry is read-only.
- Existing migration (2) and recovery (4) tests pass after this change. Added
  `revocation_between_prepare_and_activate_keeps_origins_writable` passed: no
  mapping/fence on denied activation, old writer still valid, fresh regrant
  imports, and revoked exact receipt retry still resolves historical success.
- Physical effect dispatch still needs an enforcing local worker boundary,
  durable one-use dispatch claims and monotonic boot/writer fencing. Do not infer
  that begin_attempt alone supplies those guarantees. Existing begin_attempt
  records a manual-reconciliation attempt and returns clonable metadata.
  Ward/vault standalone directories do not yet exist; their local adapter roles
  remain node::authority and the cyb organs. No new worker-as-subject hierarchy.
- Accepted next implementation contract: neuron/specs/worker-dispatch.md.
  Root placement generation + explicit worker descriptor; private affine token;
  separate durable dispatch claim; current-grant guard through publication and
  immediate synchronous call. Never reconstruct tokens on recovery. Manual take
  stays explicit unknown/reconciliation and cannot feed the enforcing executor.
  This contract is **not implemented yet**. Avoid reentrant authorize inside a
  held GrantHandle read guard: prepare signed publication before entering the
  guard, then commit/check+call under it; publish outcome after guard release.
- Worker model implementation has started: model::worker::Worker has bounded
  five-selection descriptor codec; NeuronState now has writer_generation/worker;
  PendingOperation has optional dispatch and attempt_record references. New suite
  lists worker/placement/dispatch/attempt-authorization records. Node validation
  enforces monotonic one-step placement generations and immutable placement within
  a generation. Worker records are validated on root read; dispatch records on
  pending read. Initial roots and imported pending records initialize these to
  0/None. **No engine/node worker bind/claim/execute methods yet.** Check workspace
  compilation and add full worker scenarios when finishing this slice; old v1
  fixtures remain immutable, while the unregistered new suite is still in development.
- Worker execution now implemented (not yet full G07 closure): engine::place_worker,
  begin_worker_attempt and dispatch_once; node::LocalWorker with explicit supported
  native-rust/host-act/no-proof/embedded-sync descriptor. Opaque AttemptPermit and
  DispatchToken are non-Clone and cannot be reconstructed from public Dispatch.
  Manual attempts retain generation 0 and cannot enter the worker dispatch path.
- Found and fixed a concurrency trap before tests: ordinary graph commit can
  return the same receipt to two racers, so it is insufficient for one-use
  dispatch. Added BBG apply_once + cybergraph commit_fresh + GraphPort commit_once
  (default Unsupported), rejecting even exact duplicates **inside the transaction**.
  Dispatch holds current ward guard over fresh claim commit and synchronous call.
  Outcome publication occurs after guard release; denied result acceptance is
  returned with the actual outcome for reconciliation, never an implicit rerun.
- Worker tests passed 3: full tool round-trip and unknown/restart/no second token;
  revocation after prepare and boot replacement prevent any adapter call; wrong
  network/unsupported proof and mutated public metadata cannot alter actual call.
  Dedicated commit_fresh test synchronizes two preflight reads to exercise the
  atomic duplicate race; currently running. Add crash-after-claim and policy-change
  outcome/placement race coverage and wire CLI/headless adapters next.

| Package | State | Remaining / evidence |
|---|---|---|
| P00 | active | Inventory, dirty manifest, legacy fixtures and signature vectors |
| P01 | active | Accepted cyb/soft3 identity and ownership contracts; final consistency audit pending |
| P02 | active | New execution/identity/worker/migration contracts; remaining old conceptual specs need full rewrite |
| P03 | active | neuron-id, model projection, package move and shared re-exports implemented; SDK gate pending |
| P04 | active | Shared NativeNode/GraphSession and strict signal import implemented; content store and path retirement remain |
| P05 | active | Durable programs/jobs, budgets and Rs module API implemented; final integrated scenarios pending |
| P06 | active | Signed publication and guarded fresh worker dispatch implemented; robot/vault/network integration remains |
| P07 | active | Same-store activation and sealed paged separate-store transfer implemented; remaining edge/profile audit pending |
| P08 | active | CLI/neural graph callers migrated; robot registry, typed navigation and relay still pending |
| P09 | pending | Soma/headless composition and task recovery scenarios |
| P10 | pending | SDK/identity consumers and native compatibility vectors |
| P11 | pending | Domain ontology and every assigned documentation owner |
| P12 | pending | Repository/path/package/config/URI migration, generated context/docs |
| P13 | pending | Per-gate evidence and final repository rescan |

Record actual commands, failures, fixes, artifacts and remaining work here or in
the owning repository's audit. Close gates only with evidence of their behavior.
### 2026-09-12 — worker policy recovery and Rs module boundary

Neuron workspace passed after CLI automatic emit moved onto LocalWorker.
Additional worker tests (5 total, `worker-policy-tests.log`) cover crash after
the durable dispatch claim, recovery without redispatch, and reconciliation after
policy/epoch change with the act revoked. Reconcile/cancel/archive and stopping
management paths still require the current subject/network/policy/prog grant but
do not reauthorize the old act. Future execution requires explicit authorized
rebind. Budget exhaustion now retains consumption evidence for a resolved result.

Rs uses `module!`, `Module`, `ModuleMetadata` throughout canonical source,
reference, tutorials and rsc diagnostics. Old 0.1 API spellings are deprecated
aliases scheduled for removal in 0.2; old documentation URLs redirect. The same
module has no signing identity. Macro parser/codegen paths moved to `module/`.
The migration test exposed and fixed preexisting bounded-async return type and
static deadline-metadata bugs; unbounded async methods now fail with RS101.
State migration, step reset, static/borrowed async methods and the alias all pass.
`rs-module-tests.log`: 107 core + 1 macro + 16 integration + 2 lifecycle tests;
four unrelated documentation examples remain ignored as before.
No-std core check passes. Rsc builds against installed nightly-2025-11-26 with
explicit toolchain PATH/RUSTC (system Cargo otherwise invoked stable rustc).
Compiler harness: 3 compile-pass + 7 error cases pass; preexisting unreachable
pattern warning in emit_mir remains outside the renamed diagnostics.

P05 Rs naming and preserved capabilities are implemented. This does not claim
DeadlineFuture preempts arbitrary native code; worker enforcement is separate.

### 2026-09-12 — sealed separate-store transfer

BBG ApplicationStore now provides TransferSource and a paged cursor over the five
original application tables. The source must be an existing application-only SSD
store; the exclusive backend process lock must be available. Native data or
unrecognized partitions reject this profile. Source heads, target, transfer key
and original transaction marker pin a seal; `export-v1` rejects normal old/new
Database open. The explicit transfer reader can resume the same seal. Every
source application write is fenced even through a retained handle.

Target preparation reserves source namespaces, preserves unrelated application
and native state, and installs the old-binary generation guard. Per-page copies
are immutable, cursor+data atomic; staged origins cannot write. Semantic
activation checks transfer completion and its pinned target inside the final
transaction. Content and structural closure validation use cybergraph's existing
Content codec. Common BBG history/receipt validation is shared with redb conversion;
its temporary index is now idempotent across restart. No open database files are
copied, no source history is deleted, and no staged program can execute.

`neuron stage-import NEURON --source DIR --nonce KEY --pages N` performs the
authorized staging step; repeated calls report identical manifest and monotonic
counts until complete. Ordinary authenticated `import` then activates the mapped
programs. Invalid page bounds are rejected before sealing. Source retention is
forward recovery; cancellation leaves it read-only, never silently resurrects
an old effect dispatcher. This is distinct from the existing redb conversion.

Evidence under implementation-baseline:

- transfer-tests.log: 3 same-store migration + 2 separate-store tests pass;
  reopen after every page, original history/receipts/unknown attempt preserved,
  preexisting target program retained, wrong key/denied grant/busy source rejected.
- transfer-storage-tests.log: 3 transfer fault/integrity tests + 1 backend
  migration unit test pass. Panic before page publication leaves cursor/data
  unchanged; incomplete transfer cannot activate; wrong target fails; broken
  receipt coverage never reaches completion.
- transfer-cli-tests.log: 7 real CLI process scenarios pass, including paged
  separate source staging then import and provenance inspection.
- transfer-old-binary.log: 2 tests pass with the original saved cell executable;
  it rejects both the sealed source and the target after transfer/import.
- backend-migration-regression.log: all 13 redb migration/integrity tests pass.
- transfer-integrity-lint.log: historical artifact integrity test passes and
  engine/model/node/CLI clippy all-targets --no-deps -D warnings passes. Legacy
  inspection now validates artifacts in every historical snapshot/event/operation;
  losing an old state is detected even when the latest state remains readable.

Correction to earlier checkpoint prose: current three-origin migration test uses
3M total budget successfully; the stale-target scenario uses 4M. Full worker,
publication and separate-store implementations supersede earlier "not yet"
notes above. No roadmap release gate is claimed complete until the remaining
consumers, documentation, packaging and integrated audit are finished.

### Current continuation — text archive and robot integration

The full neuron workspace passed after sealed transfer and historical artifact
validation (`neuron-after-transfer.log`). Accepted `cyb/specs/robot-registry.md`
now specifies bindings, custody, pending action selection and full-signal relay.
Implementation of that UI registry/relay contract is still pending: the current
shell Identity remains a single key resource with unsafe old fallback behavior;
relay splits signals into links and selects the first endpoint; remote pay posts
an unsigned body and currently overstates HTTP receipt finality. These are concrete
remaining P06/P08 seams, not implemented guarantees.

Started P04 content migration at its owner: `cybergraph::text_archive::TextArchive`
uses ApplicationGraph on the existing Database. Canonical versioned JSON Blob
observations retain H(text), request, previous record, optional created time and
raw legacy line provenance. Same-request retry/conflict and bounded projection
implemented. `import_jsonl` validates every complete source line and hash before
writing any prefix, preserves duplicate observations/unknown dates/raw bytes,
and handles the former writer's literal tab/CR string form explicitly. Limits:
8MiB text/raw line, 256MiB source, 64MiB full projection, 1M observations.
New serde/serde_json dependencies are optional under cybergraph local-storage.

Text archive builds and its owner tests are in `text-archive-tests.log`.
**Not yet wired** into GraphSession, shell content.rs or soma-kernel. Both shell
and soma still append the old JSONL file. Old file path retirement (graph.log and
particles.jsonl), shared-handle callers, error propagation, and complete import/
restart tests remain next. Do not claim P04 complete from the archive alone.

Storage transfer helper changes are under BBG/application/{transfer,validation}
and cybergraph/application/transfer. New normal Database open deliberately rejects
export-v1; only explicit TransferSource opens it. Source normal writer retirement
for BBG is implemented, unlike the raw graph/text file entry points above.

### 2026-09-12 — raw file retirement, shared content and robot core

GraphSession now uses cybergraph::LegacyFile to retire the old graph.log and
particles.jsonl writer paths after verified durable import. The retained original
is `<old-path>/source`, with a canonical manifest; the original path becomes a
directory so old appenders fail. The adjacent graph.bbg remains the target.
Missing target after retirement cannot silently create an empty replacement.
Unix no-replace rename (rustix) and synced staging markers recover each rename
boundary; existing competing paths remain untouched. Old writers must be quiesced.
LegacyFile tests 3 pass, GraphSession integration tests 5 pass
(graph-retirement-tests.log); includes shared text views and retired-source guards.

Shell content.rs now requires SharedGraphSession and persists text observations
before its signal callback. There is no global JSONL writer or silent empty reader.
Graph/Memory/viewer use one verified text projection; failed recall prevents a
misleading successful grounded ask. Shell startup imports/retires particles.jsonl.
Soma kernel returns computation results; host poll persists question, answer and
concepts through the same content adapter. Removed SomaConfig::particles and all
kernel/example sidecar writers. Shell check passed after this integration
(content-shell-check.log), before subsequent robot core/custody additions.

Real-model Soma regression initially failed all three inference scenarios: glia's
new row-only quantized embedding lookup lost its host bytes during GPU upload.
The relevant glia files were clean before our fix. Added embedding-custody spec;
to_backend retains quantized embedding host bytes, uploads its tensor only when
it also serves the tied lm_head. No full vocabulary dequantization reintroduced.
Rerun soma-kernel-content-tests.log: all six tests pass, including real generation,
model switch and grounded answer, with installed model weights (not skip results).

Added cyb-core::robot Registry, based on neuron-model SubjectRef/NetworkRef/Binding
and neuron-node's existing opaque SigningVault/NSIG1 verifier. One host clones its
Registry handle, preserving the current-grant mutex. Canonical retained graph
records hold robot name/selection, bounded sorted attachments, control evidence,
key locator and devices; no robot key or copied mnemonic. Observe foreign accounts
without coercing their bytes. Control supports the native signing profile only.
Revision CAS, suspension/revocation/detach, read-only exact mutation retry,
immutable durable admission, fresh signed dispatch claim, and retained terminal
result/unknown inspection are implemented. Selection changes affect future asks;
current subject/network/revision/policy/grant/payload are rechecked at dispatch.
The callback executes under the current registry grant lock. Results may reconcile
after revocation; conflicting outcomes fail. Raw Database remains trusted host-only.

Core registry tests initially passed 3 scenarios: two keys/two networks plus same
subject on another network, foreign watch-only and two device labels; persistence,
selection change, changed payload/wrong custody, state revision/revocation, two
racing dispatches (one call), and panic-after-claim/reopen unknown/no rerun.
Now adding canonical signal publication/reconciliation and custody tests; final
results are in cyb-core-content-registry-tests.log, not assumed passed yet.

Registry::cast_signal pins complete canonical foculus signal bytes and signs the
bound action before NativeNode publication; reconcile_signal resolves exact stored
bytes after a lost result. It does not relabel local acceptance as remote finality.
Explicit mnemonic custody load/create lives in robot::custody, preserving existing
mudra derivation. Create uses create_new + 0600 + file/parent sync; load failure
cannot generate/replace a key. `cy identity create [file]` is now explicit, while
CLI help/tool delegation no longer opens or creates identity. CLI build/process
checks after these changes are still required.

Fixed MoneyWallet's preexisting failed-private-payment mutation: preserve the
original note and construct change without inserting it until pay succeeds;
checked nonce overflow. Added rejection regression. Persisting private custody
and handling commit-unknown note recovery remain later vault integration work.

**Next:** Shell Identity still has its old unsafe load_or_mint and single-key UI.
The new Registry is NOT yet installed into the shell, direct shell content callbacks
still call raw GraphSession cast, and SomaPending still captures only the question.
Wire registry/custody/optional selection, freeze Soma pending author/network, adapt
all mutation paths, relay/full-signal receipts, late-balance routing and typed
navigation. Old robot name file writer remains. P04/P08 must not be marked complete.

### Shell registry checkpoint (supersedes previous "not installed" note)

Registry now installs at shell startup. Existing mnemonic derives the same mudra
native ID and attaches to cybergraph's existing private-network ID; invalid/read
failure stops startup without replacement. A fresh GUI has no invented Identity,
can use the commander, and creates keys only via explicit `neuron create`.
Native selection is an optional Identity resource; foreign/watch-only records
remain typed, custody is optional, and stale captured revisions cannot sign.
`neuron` commander operations cover attach/create/watch/native or foreign/use/
suspend/resume/revoke/detach/devices. Sigma shows an attachment summary (first 8,
explicit full-list command); robot naming uses Registry and retires the old name
file. Later rename survives repeated old-name import. No automatic new mnemonic
writer remains in shell or cy CLI.

Content's publication callback now receives SelectedSession, which calls
Registry::cast_signal, not raw GraphSession. Complete native bytes with explicit
network are admitted, NSIG1-signed and fresh-claimed before native publication.
SomaPending holds captured Identity along with the question; answers validate
queue correlation. ComSay::LineAt/StreamEndAt retain that selection through Log
rendering. Watch-only/absent selection never invents a native author. Core records
retain exact evidence and outcomes; native unknown publication can reconcile by
full stored signal bytes. This is host-local authorization, not proof finality.

Evidence:
- cyb-core-content-registry-tests.log: 26 unit + 5 GraphSession + 5 registry
  integration tests pass, including custody, bad/foreign control, legacy name,
  immutable selection, fresh-racer/crash, full multi-link signal reconciliation.
- cy-identity-tests.log: actual CLI process test passes; help does not create a
  key, explicit creation is exclusive, invalid existing mnemonic is unchanged.
- robot-shell-tests.log: real shell identity/content test passes; no selection,
  two key files/two networks, captured author after UI switch, durable answer
  content, revocation and foreign watch-only without an Identity resource.
- robot-shell-check.log: `cargo check -p cyb -p cy --offline` passes after Sigma
  attachment summary and optional chrome identity. Full shell unit suite is
  running next in cyb-shell-tests.log (not yet assumed passed).

Remaining P08 seams are still substantial: legacy body relay splits links and
posts unsigned requests to the first endpoint; remote pay likewise posts unsigned
and overstates finality, and late balances are not context-correlated. Fix the
actual owner protocol, not just a UI label or disabled functionality. The current
soft3 node explicitly declares an **unsigned local chaosnet bridge**, but already
has complete `/v2/frame`, idempotency keys and native receipt/history routes.
Relevant files: soft3/crate/src/node/{requests,routes,http,genesis}.rs and
soft3/specs/native-node.md. New signed/routed profile needs a reviewed wire contract
and owner-side verification plus local integration fixtures. Do not claim the
current registry signature is remotely verified by old `/v1/link` or `/v1/pay`.

Other P08/P09 remaining context edges: asynchronous shell output still needs its
captured caller; model work is not yet an engine invocation; SomaThread is still a
single in-memory last answer and must be scoped/persisted; body task/worker grants
and vault custody integration remain. UI private wallets currently rebuild per
selection (public history restores); private note custody is not durable yet.
Prysm/chroma typed destinations and old cell URI resolution are also pending.
Fleet harness currently expects automatic mnemonic creation and flat graph.log;
update it to explicit creation and native-store/full-signal assertions, then run.

Full shell unit run completed: cyb-shell-tests.log reports **24 passed, 1 ignored**
(the preexisting ignored case), no failures. All current compiler/core/CLI/Soma
checks above are now complete for this slice. The rest of the roadmap remains open.

Next remote-profile investigation (design notes, NOT implemented):
- soft3 Node has canonical genesis bytes, fixed SpacePussyTest product enum,
  native accept/receipt/history, `/v2/frame` complete canonical foculus signal,
  `/v1/pay` Operation::Pay and `/v1/link` Operation::Link. `/status` currently has
  no pinned cryptographic network ID/auth capability. Its spec explicitly labels
  the product unsigned local chaosnet. Do not silently relabel it authenticated.
- A new explicitly activated authenticated local-network profile should bind a
  concrete network ID to canonical genesis, publish capabilities, verify full
  signed action/context/payload and preserve proof/evidence in the shared graph.
  Old unsigned mutation routes must be rejected when that profile is active;
  old executables must be fenced by a supported BBG generation marker, not only
  an ignored config file. Do not deploy this work to external nodes.
- Move the **unchanged** NSIG1 crypto envelope implementation from neuron-node's
  authority adapter to its crypto owner mudra (node delegates). This lets a
  network verifier/SDK use it without importing engine/Rune. Public action data
  can move from cyb-core::robot::PendingAction into neuron-model's identity-only
  serde API. Preserve proof-domain bytes; settle the new unregistered action
  schema/codec in a spec before emitting its network writer.
- Relay must transmit one complete signal, retain exact author/network/step/prev/
  proof and a correlated durable cursor/result. Current NetHub config only has
  `(name,url)` strings; add explicit network/profile pinning, not name hashing or
  first-endpoint routing. Unknown HTTP outcome must reconcile the exact receipt
  using the idempotency key, never blindly issue another effect. Late balance
  replies must keep account/network/request generation and cannot change selection.
- Important protocol constraint: current cybergraph.chains indexes by NeuronId,
  not (network,neuron). One native subject has one ordered signal lineage;
  multiple compatible network bindings do not create independent chains. A
  partial relay to a destination missing predecessors must fail visibly, not
  rewrite steps/previous hashes or broadcast private history to fill the gap.
  User's different-key/different-network attachments remain the independent case.
- Remote pay can retain the existing Native Operation::Pay semantics under a
  verified outer action envelope plus retained authorization evidence; it must
  not fabricate a remote Signal step from a stale local private chain. Its receipt
  is endpoint acceptance, not verified network finality. Exact request/payload
  replay/conflict behavior must survive storage and result-publication failure.

### Auth owner extraction started

Implemented mudra::neuron::{sign_statement,verify_statement} by moving the existing
NSIG1 message/sign/verify logic unchanged; neuron-node KeyVault/verify_evidence now
delegate. New mudra/specs/neuron-auth.md records the exact 102-byte profile and its
limited native secp256k1 assurance. No signature domain, signer HRP or native ID
algorithm changed. The no-default mudra test log is mudra-auth-owner-tests.log.

Moved public PendingAction fields unchanged into neuron-model::action::ActionRequest;
cyb-core re-exports the old local name. Retained the existing cyb/robot-action/1
statement domain for byte compatibility. Added bounded SignedAction canonical
serde codec/schema neuron/signed-action/1, structural validation without implied
crypto authorization, and owner spec action-envelope.md. Model serde adds optional
serde_json; ID-only mode remains independent. Registry admission/read now validates
the common public action. New tests check canonical/unknown-field/size/context
rejection; a structurally valid fake proof explicitly still requires mudra verify.

Running/next evidence: neuron-auth-owner-tests.log (full workspace),
model-action-tests.log (serde/identity-only plus no-default normal tree),
robot-shared-auth-tests.log (registry after extraction). Inspect actual results;
do not infer passing from compilation. No signed HTTP profile or relay changes
have been implemented yet.

### Signed native adapter and robot outbox checkpoint

The above "not implemented" statement is superseded here. Auth owner checks
passed: mudra no-default 16, model serde/identity 4 with only neuron-id in the
no-default normal tree, full neuron workspace 40, robot registry 5. Their logs
are the files listed above. The external zheng tagged/hash module temporarily
blocked builds; it subsequently appeared through concurrent work. No external
module was removed or gated to bypass that failure.

BBG ReaderGeneration is monotonic (Original, NeuronV1, AuthenticatedV1), with
`auth-v1` persisted in the existing migration metadata. Application migration
and separate-store staging promote rather than overwrite the marker. Evidence:
reader-generation-tests.log (1 actual generation test; the initial wrong filter
ran zero, was corrected), auth-transfer-tests.log (3 transfer tests, including
preservation of AuthenticatedV1 through staging and final activation).

Soft3 now explicitly enables `neuron/signed-native/1` with `soft3 auth enable
[--home DIR] [--import-legacy]`. It pins H(canonical existing genesis), advertises
capabilities, verifies the common NSIG1 envelope, journals its complete bytes in
ApplicationGraph before NativeNode accept, and supports exact receipt lookup.
Signed kinds are full native/signal and endpoint-sequenced native/pay. All old
HTTP mutation routes reject when enabled. No external node was deployed/upgraded.

The retained old soft3 executable was discovered to be a **pre-BBG flat-log**
implementation: it ignored the BBG marker and swallowed append errors. A marker
alone did not fence it. Auth activation now also retires mandatory genesis.json
via LegacyFile; original bytes live in genesis.json/source, canonical genesis
and network ID stay unchanged. Startup finishes interrupted retirement before
listening and rejects a retired source without its original authenticated DB.
Old uncooperative processes must be stopped before upgrade; this mechanism does
not revoke already-open descriptors or running RAM state. `--import-legacy`
explicitly imports the old full log before activation, retaining original bytes.

signed-native-tests.log: **5 tests pass**, including full proof/signal/payment
retry, prepared-journal restart, mismatched subject/network/signature/canonical
bytes, unsigned-route denial, activation interruption and missing-target failure.
auth-old-reader-tests.log: real original binary created a flat-log signal; current
CLI imported it, retained log/genesis bytes and height; original binary then
failed its mandatory genesis read; current binary reopened and denied all four
unsigned routes. Rehearsal source is check-auth-old-reader.py. It uses local
temporary homes/listeners and retained binaries (ignored machine artifacts).

Cyb core robot::remote implements native outbox delivery over a bounded transport
trait, validates endpoint capability/network, resolves historical receipts even
after revocation, and permits exact POST retransmission only with the current
original binding grant held through the call. Generic tool dispatch cannot use
this adapter. It retains correlated endpoint receipts separately from local
execution outcomes. Per-binding (subject/network) scan cursors validate every
crossed native Dispatch has a complete accepted receipt; pages bound to 64 rows.
Old relayed marks import as provenance and their file writer path is retired;
they never manufacture new signatures, receipts or successful high-water marks.

Shell relay now reads the signed registry, preserves whole signal/proof bytes,
and routes by explicit network/profile pin. Old name/URL-only configuration stays
read-only; `net probe NAME`, `net pin NAME NETWORK_HEX` make selection explicit.
Config edits are strict/bounded/atomic, old probe generations cannot mutate a
replacement endpoint's observation. Sigma payments and balance queries use the
captured subject/network/revision. Replies are generation-correlated, and the UI
calls endpoint acceptance by its actual name rather than finality. Oracle now
uses selected network and drops superseded endpoint responses too.

A robot that locally casts also assigns payment sequence locally: pay_signal
creates the complete native/signal with delta_pi before network delivery. This
prevents local linking from racing server-assigned payment steps. Insufficient
local synchronized native funds fail before admission. Standalone native/pay
remains available for clients that delegate sequence assignment to the endpoint.
The old single chain per NeuronId constraint still applies; unrelated private
predecessors are never silently broadcast or rewritten to repair a gap.

native-outbox-tests.log: **4 integration tests pass** — lost response/restart and
revoked read-only resolution, byte-identical retransmission/current-grant denial,
earn→pay→link on one sequence with wrong-receipt rejection, retained legacy marks.
native-shell-tests.log: **25 passed, 1 preexisting ignored**. native-relay-check.log:
shell check passes. Native genesis retirement and full core suite are being
rechecked after the latest custody zeroization change (native-core-tests.log).
NativeKey load/create now zeroize temporary mnemonic buffers; existing custody
derivation/files remain unchanged. Initial private attachment import no longer
overrides a preexisting explicit selection or attachment configuration.

Still open immediately: old body chainsync uses /log and name-only synced marks,
which cannot carry full network/proof bytes; replace with a full native history
view and durable scoped observation cursors. GUI/Fleet real-process end-to-end,
headless registry/CLI authorship, typed navigation, scoped durable agent work,
private vault custody and the remaining P09–P13 packages are not complete.
No release gate is closed by this checkpoint alone.

### Complete native history projection and real HTTP composition

The open chainsync seam above is now implemented. Cybergraph NativeNode owns
history_view: canonical original operation/receipt plus its exact committed
Signals (including generated Link/Pay block signals). Soft3 /v3/history exposes
bounded 1–16 entry, 32 MiB pages with pinned network/profile, after and next.
Native source positions start at **1**; ApplicationGraph heads start at **0**.
Initial mirror tests caught an incorrect shared-origin assumption; the codec
now explicitly stores source position = observation head index + 1.

Cyb core native_mirror retains receipt + original operation Blob + complete
signal Blobs and their previous observation in the same Database. Whole-page
validation precedes writes; exact historical observations compare provenance.
Native-before-observation failure retries duplicate signals without redoing their
economics. Copied Intent/LocalCredit stays provenance, never executes as a host
adjustment. Mirror binds its graph adapter to the GraphSession supplied to each
operation, so it cannot publish provenance into a different retained Database.
Legacy synced files now import/retire through the same provenance-only marks
path as relayed. Body chainsync uses /v3/history and durable network-scoped source
positions, with no remaining old /log byte-offset writer. Body displays relay and
sync errors. Protocol/profile/finality qualifications remain explicit.

native-mirror-tests.log: **3 pass** — complete proof/payment/restart without copied
host credit, malformed-tail/wrong-network/next rejection before mutation, crash
after native commit before cursor. native-mirror-shell-check.log passed before
the latest small follow-ups (repeat with the new tests below).

Soft3 exposes serve_connection for an embedding host using the same HTTP path.
The first real socket test caught inherited O_NONBLOCK on macOS; handle_client
now explicitly sets blocking mode with existing finite read/write deadlines.
The same test then passed: native-http-integration-tests.log has **1 real HTTP
integration**, two independent genesis/network IDs and controlled keys/devices,
whole signal relay, payment and subsequent linking, second graph mirror,
receipt/cursor restart, byte-identical signal hashes, wrong-network denial.
Shell dev-dep soft3 is test composition only; no production GUI dependency was
added to the native server. No fixtures use an external endpoint.

Another boundary found while testing: original native economics deliberately
skip an insufficient delta_pi leg during legacy replay. A signed action must not
turn that into an accepted payment. Owner Ledger now exposes whether any leg was
skipped without changing legacy economics; NativeNode::validate_payments rejects
such fresh signed submissions. Exact existing request resolution runs first so
already-paid retries remain valid. The new signed-signal insufficient-leg test
is running in native-history-server-tests.log (inspect the actual result).

Network money caution: the shared GraphSession's native ledger is an aggregate
local graph projection, not proof of funds on every network represented there.
Sigma pay now requires the matching subject/network/revision observation and
sufficient observed endpoint balance before local admission; actual acceptance
still checks all payment legs at the pinned node. Ledger/source scopes and stale
observations must remain explicit in the final audit; do not relabel the shared
local balance as network-finalized funds. The strict native adapter prevents a
successful receipt for a skipped leg. Unknown/rejected requests remain visible
and retained; local admission alone is never remote success.

Latest full core suite native-core-tests.log passed 26 unit + 5 GraphSession +
4 outbox + 5 registry tests before adding the three mirror tests. Latest full
shell suite native-shell-tests.log passed 25 + 1 preexisting ignored before the
real HTTP test. Run appropriate combined suites after final changes, not just
the focused test, and preserve concurrent dependency changes.

Next P08 task: headless cy registry commands and all CLI/neural authored paths.
CLI main currently still loads ~/cyb/mnemonic directly and raw-links with that
key-derived ID, without the Registry grant boundary. `bind` accepts another
neuron's valid legacy claim then raw-authors a new Signal as that neuron: keep
claim bytes/verification but remove that fabricated fresh author capability.
MoneyWallet local operations likewise need current-bound host authorization;
wallet proof formats/economics must stay intact. Shared command parsing should
live in core/host, reused by shell and CLI, enabling Fleet to preseed explicit
public bindings without GUI automation. Initial private import must preserve a
preexisting selection (already fixed). CLI read-only views should work without
minting a key. After that run/update real Fleet, typed prysm/chroma destinations,
queued shell/model identity, agent composition, SDK/domain/docs/packages/audit.

### Shared host commands, headless authors and typed navigation

The common `cyb-core::robot::host::Host` now owns create/attach/watch/use/state/
devices/name commands for shell, cy and neu. Fresh robots need no key to inspect
or enter the REPL. Creation validates names/networks/device before exclusive
custody creation. Existing mnemonic import preserves a preseeded selection and
reports damaged custody without replacing it. CLI authored links and verified
foreign claim publications use the selected controlled attachment; a claim is
retained evidence and never authorizes a fabricated new author. `neu speak`
uses the same registry, keeps the historical @you chain, and requires a real key
for new writes. NEU_LOG's old default location remains compatible and an explicit
path can share cyb's store. No key is generated by readonly startup.

CLI local MoneyWallet commands bind their full arguments to a Registry action
and use the captured network on the original Signal/proof codecs. REPL wallets
are cached by (native subject, network); this cache is process-local, not private
note durability. The unregistered, unused demo wallet controller and state were
removed from shell; actual Sigma payments use the previously tested native
outbox. Core wallet commands remain available through cy. Shell graph statistics
now count canonical encoded signal bytes through GraphSession, not the retired
flat log path. No disk size or network finality is inferred from these numbers.

`SubmittedCommand` captures optional Identity at commander submission. None stays
None. Pay/cast/ask and shell execution preserve that capture. Trusted local
shell/eval launches its worker exactly once under a fresh current-grant guard;
selection changes do not retarget it, revocation before launch denies it, and a
receipt cannot authorize a second launch. Input must equal admitted bytes.
Nushell/Rune run off the GUI thread with a 64-message backpressure queue. Results
retain complete tape frames up to 256 KiB, total observed byte count and explicit
omission flag in a Registry Outcome; terminal errors no longer show status zero.
A missing terminal observation remains unknown and is never auto-reexecuted.
This is the trusted interactive console profile: Nushell cancellation is
cooperative; it is not a hard sandbox/CPU bound or the future agent-tool worker.
Soma's actual admission/worker/recovery integration remains P09 work.

`neuron-model::navigation::Route` provides bounded URI codecs for View, Particle,
Neuron and Prog with optional explicit network. Native/foreign bytes and domain
qualification round trip without secret/runtime dependencies. Legacy cell://
requires an exact resolver; prysm maps only published `landing` to robot View.
Unknown origins fail. Prysm launcher/pins/context contracts now carry Routes;
shell opens neuron/prog inspection in Sigma without selecting/attaching/running.
Particle URI entry handles content without a geometry node (optional index),
and empty graph views retain known content metadata. The unregistered prototype
page loader that silently accepted host acts was removed from robot view.
`neuron actions [AFTER]` and `neuron result REQUEST` expose bounded admission
pages and admitted/unknown/completed observations without enabling replay.

Verification in implementation-baseline:
- neural-author-tests.log: four real authoring/legacy/revocation tests passed.
- host-cli-tests.log: identity safety + five process/REPL tests passed.
- navigation-model-tests.log: identity-only unit/integration suite passed,
  including exact foreign URI bytes and explicit legacy mapping.
- shell-host-tests.log: three shell tests passed; the full rerun below also
  validates exact input binding added after the first targeted run.
- host-shell-tests.log: 30 passed, one preexisting ignored. Includes actual
  Nushell arithmetic worker, fresh-claim/current-grant checks, captured None,
  bounded transcript, typed navigation, and two real signed HTTP networks.
- host-core-cli-tests.log: combined core/CLI rerun started; read final summaries
  before promoting this run to evidence (not a status claim here).

P08 remains active. Fleet still assumes automatic root mnemonic and flat log;
its mockchain accepts old unsigned links and cannot validate the new native path.
It must use the actual signed soft3 endpoint for relay, retain the deterministic
mock only for readonly beacon isolation, and seed bindings explicitly. Before
that, the live vault UI still derives from ~/cyb/mnemonic/vault.enc; this is an
actual remaining authority/custody seam, not a reason to seed a fake root in
Fleet. Implement subject-scoped private storage and explicit selected-key access,
retain/migrate legacy encrypted bytes, and test revoke/switch/empty/watch paths.
P09 private note durability and the remaining agent composition are still open.

### Private vault authority and ciphertext migration

The live vault no longer reads an ambient robot root. `cyb-core::private_vault`
now owns CYBVLT1 decode/encode, bounded entries, subject-qualified encrypted BBG
ApplicationGraph revisions, expected-head CAS and immutable update receipts.
The original SHA256(cyb-vault-v1 || BIP39 seed) derivation is unchanged and checks
the locator's original native subject. Entry values, seeds, plaintext encoding
buffers and key material zeroize on drop. Crypto temporary storage stays local;
no plaintext entry is put into an action, native signal or text archive.

Legacy vault.enc is validated with its original mnemonic's key and imported as
exact original ciphertext, then retired through LegacyFile. Retained bytes and
deterministic import receipt permit copy/retire retry even after newer revisions;
missing retired target, incompatible target, wrong key or damaged ciphertext fail
without an empty-store fallback. Host initialization attempts this for the legacy
owner independently of selection and surfaces failures. GUI private access
rechecks the source fence; unresolved old ownership is an explicit error.

Registry.private_access signs only operation kind and dispatches under the exact
current binding. Shell captures its RobotHost data directory, not ambient HOME
at future execution. View/reveal/copy/add/remove use selected custody; clipboard
write runs inside the current-grant callback, and its timeout is retained even if
recording the result later fails. Switching/revoking/exit drops owned entries and
clears rendered Text buffers; the clipboard timeout survives navigation. The
identity seed row comes from the selected locator, never ~/cyb/mnemonic fallback.
Vault command replies do not expose entry names/values into Com history, and raw
vault command buffers are cleared after interception. `neuron private-status`
provides headless encrypted-head/revision/count inspection with the same grants.

Evidence:
- private-vault-core-tests.log: three passed, including fixed known mnemonic
  derivation ace064…9cdd02, original ordered JSON/CYBVLT1 ciphertext reader,
  wrong-key denial, exact import/retention, CAS conflict/retry and size limits.
- private-vault-ownership-tests.log: selected independent keys, retained capture,
  revocation/foreign denial, separate seed rows and no secret action payloads
  passed. This was before the final reveal-name refresh/private-status additions;
  those still need the final combined rerun.
- private-vault-shell-tests.log: 28 passed, one preexisting ignored. Two old
  standalone cipher tests moved to the actual owner codec/migration checks above;
  TOTP presentation vectors remain in shell. A new ownership test followed.
- host-core-cli-tests.log completed earlier: CLI 1+5, core 26+5 graph migration+
  3 mirror+4 outbox+5 registry tests passed (pre-private-vault additions).

Fleet has been rewritten to create native attachments with cy, use real signed
soft3 for relay/history/root receipts, and keep mockchain as readonly deterministic
beacon plus a separate blackhole. It checks encrypted vault status through the
headless API, scans actual BBG files for its secret canary, and kills only owned
process groups. First make fleet build/run is active; inspect fleet-signed-native.log
before assigning a result. No release gate closes on this paragraph.

Fleet completed: **32 passed, 0 failed** (`fleet-signed-native.log`). Three actual
GUI bodies rendered screenshots, retained distinct explicitly created keys and
BBG graph histories (22/21/22 observed signals), relayed into authenticated soft3,
matched native history receipt roots, verified proof tickets and isolated the
blackhole from the readonly beacon. The vault canary recovered through the
headless selected-key API and was absent from all body files/stdout as plaintext.
All owned processes stopped. Successful temporary fixtures were cleaned by the
harness; the complete assertion log is retained. A follow-up harness-only change
clamps a residual sleep interval at zero, avoiding a timing-boundary exception.
This is P08/P06 evidence, not P09 tool/agent recovery or a release-wide conclusion.

Final P08 composition rerun (`p08-composition-tests.log`): **81 passed, one
preexisting ignored**, across cy process/REPL tests, cyb shell tests and cyb-core
unit/integration suites. This includes final private-status/reveal-name refresh,
fixed CYBVLT1 derivation compatibility, selected seed/ciphertext ownership,
clipboard grant placement, typed navigation and real HTTP relay/mirror scenarios.
Git diff --check for cyb passed. Dedicated new/shared host implementation files
were rustfmt'd after the run (formatting only; no sibling/package-wide format).
P09 is next; SDK/domain/packaging/full release gates remain open.

P09 entry checkpoint: `soma/agent` is a new *library within the existing Soma
repository*, not a new signing component. It currently contains only the pinned
Rune host-loop source and `runtime_seam` test. The real Rune machine suspends,
restores and loops over 42→43→0 without a new VM; test passed after correcting the
test caller's total-budget argument (it is a ceiling, not consumed steps).
`soma/specs/neuron-tasks.md` now records the required application/engine boundary,
context/controls/schedules/joins/reconciliation and conformance matrix. No durable
Soma task implementation, real tool adapter or G10 completion is claimed yet.
Soma CLAUDE ontology now follows cyb (named robot, attached neurons, versioned soul)
and distinguishes current local inference from planned proof/provider inventory.

### P09 durable task library and publication guard

The previous "seam only" checkpoint is superseded: soma/agent now implements a
bounded durable task catalog, immutable input/context/step/observation records,
explicit prog slots, model/tool dispatch over the existing Rune loop, trusted
receipt ingestion, engine reconciliation, controls, three child joins, occurrence
scheduling and proposal-only learning. Built-in tools are pure text.hash and real
Unix workspace.read (directory fd, no-follow regular file, optional expected
hash and output cap). No model, task, child or worker gets another signing key.

Task inputs pin subject/network/attachment/revision, soul/model/tool schemas,
workspace/disclosure and memory source heads. Steering and model/context changes
retain accepted old results; future tool/delegation adoption checks the control
revision, and even an outstanding final answer gets another model boundary if
steering arrived. Unknown/panicked adapters do not rerun or poison other work.
Adapter observations precede engine reconciliation; duplicate historical replies
resolve by operation/attempt and raw output hash. Admission preparations survive
a missing engine or application receipt and expose exact recovery; schedule
occurrences derive stable nonces before admission. Parent grants/disclosure stay
subsets; child instruction refunds preserve already incurred charges.

Two authorization corrections were necessary during composition:
- BBG Database now shares bounded application coordination locks between
  independently constructed Registry objects. Expired weak entries are reclaimed;
  no database/wire layout changes. bbg-coordination-tests.log: **1 passed** with
  backend-ssd explicitly enabled (the first default-feature invocation selected
  zero tests and is not evidence). shared-registry-grant-tests.log: **6 passed**.
- Neuron activation and ordinary publication now use Authority::with_current
  through the actual commit, in addition to the existing import/dispatch guard.
  A signature prepared before revocation cannot publish afterward. Canonical
  record/signature bytes are unchanged. neuron-publication-guard-tests.log:
  authority 2 + migration 3 + runtime 5 + worker 5 passed. A new direct
  revoke-after-signing test then passed with the two authority tamper tests:
  neuron-revocation-publication-tests.log **3 passed**. The fabricated-authority
  test explicitly invokes the guarded callback so the graph verifier is still
  exercised, rather than passing merely because a guard was unsupported.

Cyb core now exposes RuntimeAuthority and Host.task_agent for both body entry
points, using the existing native custody and exact captured attachment/device.
Its device ID is a placement label H(cyb/local-device/1 || UTF-8 label), not hardware
attestation. TaskReader inspects without placing a worker; reopening execution
fences the prior worker and retains its original budget despite a larger requested
initial amount. soma-host-tests.log: **6 registry + 1 runtime authority + 1 actual
Soma host composition passed**, including real file tool, restart, two independent
keys/networks, selection retention, revocation and no task-created accounts.

soma-agent-tasks.log last completed run: **11 passed**. These cover real tool
round trip, independent work, suspend/reopen/exact observation, all joins and
refunds, parent cancellation/failure, context/model and steering (including a
pending final reply), revoked grants, panic/unknown, schema/path rejection,
schedules and injected admission/receipt/result-write boundaries. The latter
fault tests first exposed the missing ordinary publication guard and failed;
they passed after fixing the owner boundary above. A subsequent memory/subset
scenario and additional catalog/parent checks are currently under the final
soma-agent-all-tests.log rerun; inspect the result before claiming it passed.

P09 remains open: the actual local model provider, GUI and headless task commands
must now use this composition; private wallet notes also remain to integrate.
This paragraph is owner evidence, not G10 or release completion. SDK/domain/
packaging/full migration gates remain open as recorded earlier.

The task-library rerun completed: soma-agent-all-tests.log **12 integration + 1
Rune seam passed**. A follow-up adjusted the memory test to check an actual new
tool scope rather than a duplicate schema; soma-memory-subset-tests.log **1 passed**.
New shared poll_tree scheduling and a terminal-control admission check followed;
these still require their targeted validation. Source formatting was limited to
new Soma files and the specifically changed core/neuron owner files.

The actual glia provider now exists in soma/kernel/src/agent.rs. It pins exact
weight-file SHA-256 under a named Hemera revision profile, verifies before/after
load, caches only the loaded revision, bounds queue/events/context/output and
checks cooperative deadlines between prefill/decode steps. Structured decisions
use the task schema; ordinary prose is final output; malformed attempted JSON is
an observed failure. It has no keys. Provider events correlate task/operation/
attempt; accepted work may finish after revocation, and lost outcomes stay
unknown. soma/specs/local-provider.md defines this local handoff profile.

soma-real-provider-tests.log: **1 actual-weight composition test passed**, using
local qwen3-0.6b-abl.model (430,237,651 bytes), actual captured cyb Host/Registry,
Rune worker dispatch, glia generation/stream, durable observation, engine completion
and store reopen. It took 67.84 seconds. The first attempt was interrupted during
unoptimized debug SHA-256 pinning; a process sample identified the checksum path,
then the sha2 dev-package optimization made this practical without changing bytes.
The explicit model-integration feature now selects this asset-required test;
missing weights fail when selected and do not silently count as a successful skip.

Headless `cy task` now has initial commands for reading, admission, driving,
controls/model changes, schedules and recovery, using common Host.task_agent and
kernel::agent::drive. CLI check passed (soma-cli-check.log); process/real-command
verification is next. It is also routed in the REPL before key-specific legacy
commands. New task-commands.md specifies the interface. The Bevy bridge still
uses the old direct kernel path and must be migrated next. P09 and G10 remain open.


### P09 product composition and private-state completion

The prior direct-Bevy warning is superseded. The Bevy bridge now uses a bounded
background body, the common captured Host.task_agent and the same local provider
as cy task. It resumes existing current-attachment work, drives child trees,
processes loaded owners' durable schedules, routes provider events by actual
catalog ownership and keeps admission-time author/network/model through UI
changes. Read/list/show/run, cancel/steer/model controls are exposed in commander;
full context and schedule files are available through cy task/API. Empty GUI
catalogs do not implicitly activate a runtime. Owner/command/event queues are
bounded; repeated identical schedule errors are suppressed. Failed model-choice
persistence is now reported before changing UI selection.

The common tree driver uncovered two real bugs: direct child submission omitted
Soma's parent.children update, and a parent cancellation could prevent recovery
of an already engine-admitted child whose Soma receipt was missing. Both are
fixed. Neuron::lookup_admission reads the exact original nonce without a new
permission; recovery materializes the existing child and inherits cancellation.
soma-child-receipt-tests.log: **1 passed**. soma-agent-final.log: **15 integration
+ 1 Rune seam passed** (the first invocation accidentally selected zero tests due
to shell redirection parsing; that log was replaced by a real unfiltered rerun).

soma-bevy-correlation-tests.log: **3 passed** for out-of-order two-subject results,
identity/model capture, mismatched admission refusal and child/structured stream
handling. soma-cli-process-tests.log: **1 passed** for real task command processes,
controls and revoked explicit-attachment readback. Full context CAS and engine
status in show were then included in the combined rerun below.

cyb/harness/tasks.py now runs actual cy and cyb executables with isolated HOME,
explicit native custody and the local qwen3-0.6b-abl.model. soma-real-bodies.log:
**both bodies passed**. Each produced a model answer, persisted the task and
observation, stopped, reopened its store and returned the exact terminal result
without another inference or attachment. The GUI screenshot was inspected and
shows the original question with its completed answer. Only owned process groups
were stopped; fixture artifacts remain under the logged cyb-tasks directory.
Kernel unit rerun soma-kernel-final.log: **7 passed**, including actual local model,
grounding and model switching. The explicit durable provider feature test is
being rerun in soma-real-provider-final.log after correlated metrics/QoS changes;
its result must be read before claiming that particular rerun passed.

Private wallet notes now have a real durable product path in cyb/core PrivateNotes
and cy notes, under the existing Registry.private_access current-grant guard.
The same vault key/unchanged CYBVLT1 cipher stores a separate subject/network
catalog head. Original notes, nullifiers, exact local signal IDs, planned change
and phase receipts are encrypted. No private command arguments enter the public
Registry journal. A prepared unknown note is unavailable; explicit reconciliation
observes the sole local coordinator and never calls pay. Import supports the old
live in-memory wallet material (no historical note file is invented). Commitment,
nullifier and payment bytes stay unchanged; this is local public-balance movement
with private wallet material, not remote settlement or a new privacy protocol.

private-notes-tests.log: **3 passed**, including genuine payment/reopen/change,
exact receipt, wrong subject/network separation, bounded catalogs, stale CAS,
revocation and injected failures before preparation, after preparation and after
payment. private-notes-cli-tests.log: **1 process test passed** for import/spend,
reopen, reconcile and revoked access with original custody. PrivateNote Debug is
redacted and its secret is zeroized on drop; import bytes and encrypted plaintext
buffers are zeroized. REPL refreshes its public graph view after this writer.

p09-composition-tests.log final: **92 passed, 0 failed, 1 preexisting ignored**
across cy, cyb and cyb-core. Source formatting was limited to explicit newly owned
files, with skip_children; no package-wide sibling reformat. The remaining real
provider rerun and final fleet/release matrix are tracked independently.

Soma README, CLAUDE and soma-spec identity/resource/task/memory sections now follow
cyb: named robot, explicit domain-qualified neuron attachments, versioned soul,
multiple bodies, no root or automatic child signer. Working adapter guarantees
are separated from the staged model/market/proof inventory. The required
requirement→owner→scenario→evidence matrix is soma/audit/neuron-composition.md.
P09/G10 native local scenarios are implemented and evidenced; P10–P13 and the
remaining migration/release checks remain active. This is not completion of the
whole convergence goal or full Hermes upstream feature parity.


P09 provider rerun finished: soma-real-provider-final.log **1 actual-weight test
passed**, 63.56 seconds, including correlated metrics. The previous pending line
is superseded. Work continued into 2026-09-13 with P10 consumer verification.

P10 found and fixed an actual lytics identity bug: live ingestion derived
H(pubkey), but replay invented H(bech32) after discarding signature provenance.
New stored events retain the original public key/signature/PoW/accepted target
and HRP inside the existing encrypted payload. Replay validates before publishing
its rebuilt projection. Old rows without the lost key remain analytics history
under their original address, explicitly legacy/unresolved; no synthetic native
subject is created. The graph provenance endpoint exposes this distinction.
Native projection remains an observation, not authority to relay a visitor's new
action. App.cell and derived graph parameters are now named graph; cohort-table
cells remain unchanged. lytics/specs/neuron-history.md defines this migration.

Current evidence: lytics event 22 original tests plus one new frozen domain vector;
ingest **49 passed** including native chain equality before/after replay, continued
sequence, retained legacy row and atomic rejection of damaged readable history.
The unrelated inf-source optional BBG version constraint was stale (^0.1 vs the
checked-out 0.3); it was aligned so the consumer workspace resolves.

Exact old domain outputs were captured by compiling mudra HEAD source primitives
from revision 348c46195388ac009667144987511a0e33bdc859 in an isolated fixture.
Original source bytes and SHA-256 provenance are retained under
implementation-baseline/legacy-domain-capture; domain-vectors.json records keys,
addresses, native IDs, ADR-036 documents and signatures for two public test inputs.
Both cyberia-my checkouts now call their same pure stored-envelope verifier;
each passed its original-domain/serialized-history/tamper fixture without changing
browser storage names or signed bytes. Mudra re-exports the minimal NeuronId and
its outdated H(secret)/signature-free current-profile comments were corrected.
Its original bridge and new domain vectors pass. JS SDK build and **3 offline
compatibility tests passed**, including independent CosmJS verification of those
old Rust signatures, Cosmos HD address, MsgCyberlink protobuf/amino and CosmWasm
MsgExecuteContract bytes. Dependency versions were not changed; the previously
missing npm lockfile now pins the installed graph. Neptune CLI's existing
**8 wallet process tests passed**; no trisha source was changed by this work.

P10 remains open while browser-target builds, dependency-weight evidence and the
remaining consumer/facade map are checked. P11–P13 and final migration/release
rehearsals remain active. No external deployment or commit was performed.


### P10 consumer and facade completion — 2026-09-13

The pending browser-target warning above is superseded. The actual problem was
Homebrew rustc winning PATH while rustup had the WASM stdlib. Explicit stable
RUSTC/RUSTDOC produced successful WASM builds for lytics-event and both
cyberia-my checkouts. Their full browser normal/build trees each contain 198
unique packages and no neuron engine/node, BBG, Rune, Bevy or inference dependency.
check_identity_consumers.py now additionally compiles seven isolated WASM Cargo
roots and audits their actual normal/build graph (not workspace/dev unification).
All seven passed; exact package sets/features/compiler are in
implementation-baseline/identity-consumers/results.json.

True-cyber's former `--neuron 01` unsigned frame path has been replaced completely.
It now uses common cyb-core Host/Registry/custody, GraphSession, NativeMirror and
signed-native delivery. Network is explicitly pinned, request is printed before
publication, full amount/valence remain signed, exact retry uses original bytes,
and read-only receipt resolution survives revocation. Its original log path is
strictly imported via the existing coordinator and retired with source bytes
intact; `legacy inspect/export` preserves and exposes that provenance. No key or
network is inferred from old label authors. Unknown CLI/options exit nonzero.
Registry.publish_signal is now public for complete host-prepared Signals and
checks captured subject/network/revision and link authors before normal guarded
admission/dispatch. It does not introduce a signing implementation.

p10-true-cyber-tests.log: **4 real-process scenarios passed** against actual
soft3 authenticated HTTP nodes: two keys/networks, exact/conflicting retries,
wrong destination, revoke/read-only recovery, lost HTTP response, crashes after
admission/dispatch with absent local receipt, old-log migration/export and bad
input. An initial lost-response test omitted the explicit attachment after
revocation cleared selection; it was corrected to test historical attachment
readback, then all four passed. README/help/specs now describe the actual client
and current local source install, rather than the earlier two-dependency probe.

Inf's Value::neuron takes the minimal common ID without changing Hash bytes.
BbgSource's stale private-balances comment was corrected: it reads public opt-in
balances, not A/N secrets. Stale local BBG/nox/zheng version constraints prevented
its workspace resolving and were aligned with current owners. Evidence:
p10-inf-tests.log **24 passed** (source with BBG + values), and
p10-inf-workspace-tests.log **99 passed** (default whole workspace), no bootstrap.

Execution query schemas belong to neuron/specs/query-projection.md and the
optional neuron-node/query adapter. It reads actual bounded canonical snapshots
into immutable inf relations, with subject/network/commit/revision on every row.
Prog/task/operation IDs remain data, artifact plaintext/custody never enters the
projection, and a commit index is not mislabeled network height/proof. The query
language does not import its executor. p10-neuron-query-tests.log **1 passed**
for actual unknown operation, independent prog, revoked read, reopen, completion
and unchanged previously captured snapshot.

Soft3's optional `stack` now resolves the current sibling cyb facade rather than
an unrelated published 0.2.1 copy; p10-soft3-stack-check.log passed. Soft3 and
true-cyber manifests now declare the actually tested Rust 1.95 baseline, replacing
stale 1.85/1.74 claims; WASM identity checks used installed stable 1.98 explicitly.
The remaining packaging/build profiles are still P12/P13 work.

The supported consumer/profile disposition and all evidence are consolidated in
consumers.md and soft3/specs/identity-consumers.md. Foreign Cosmos/Neptune/CosmWasm/
EVM wire/storage fields, numeric subnet UIDs, transport endpoint keys and table
cells are preserved. Radio/tape documentation separates presentation/transport
from authority. Fs patch/sync now bind explicit subject/network/grant and do not
pretend CRDT convergence fences two writers; its 64-byte native-ID/automatic
pubkey rotation draft was corrected to the actual native profile. Target patch
proof services are explicitly distinguished from current local implementation.
P10/G11 supported profiles are complete; this is not whole-plan completion.

Next: P11 domain ladder and the retained old neuron/specs drafts; P07 sealed
inspect/export/estimates/fault matrix; P12 packaging/generated contexts; P13 full
integration/revision manifest and final plan audit. Full cybergraph stack_* tests
still need obsolete nox/zheng API adaptation, and final GUI fleet must rerun after
integration. No commits, pushes, deployments or user key replacements occurred.


### P11 normative convergence — 2026-09-13 (in progress)

The neuron specs index/model/data/foundations/integration/lifecycle and remaining
execution/authority/history/communication/evidence/evolution/API/agent/evaluation/
conformance contracts now use accepted neuron/prog semantics. Native APIs and
persisted fields were checked against engine/model source. Important corrections:
queued/unknown are derived states, native upgrade retains old invocation source
and detects revision conflict, current-grant publication is independently checked,
and proposed generic wire/ports are no longer described as implemented native
fields. Full agent requirements and comparative gates remain separate from the
local migration profile. Original local-runtime.md is explicitly archived; its
immutable suite and reader vocabulary remain unchanged.

Cyb anatomy retains 21 organs with prog as the extension ability. Log is history
presentation, while Cybergraph/BBG retain canonical records and necessary content.
Ward and scripting documentation now bind host-held current grants rather than
claim mutable ~caps values supply authority; one-shot gates, durable progs, tasks,
trusted native console and future plugin conventions are separated accurately.
Robot/soul descriptions no longer require a root neuron. Crystal neuron/prog/
progs/soul/node/component pages now share these meanings and preserve foreign
contract identity profiles. Satoshi soul/skin design now makes subject separation
an explicit authority/attribution choice instead of minting a key for every persona.
Biological, battery, physical estate, table, Noun/VM and DAS terms were inspected
and retained. The roadmap's 22-file functional matrix was corrected to the user's
current anatomy meanings for brain/memory/plan/time; no agent requirements removed.

Cyber graph-wide documentation instructions explicitly required disjoint parallel
agents. protocol_domain completed the seven canonical cell/shard/hierarchy/3c/
network/oikos/spectral pages and continues the enumerated AOS/Cyberia/MiDAO domain
pages. protocol_product updated nine product/spec/roadmap pages and is aligning
README with the launcher. protocol_remaining completed five other protocol pages
and is implementing the launcher P12 gap discovered during that review. Their
source scopes do not overlap. No commits or deployments were requested/performed.

Current checks: neuron and cyb/crystal git diff --check passed; 37 owned normative
Markdown files have no broken local Markdown links. Canonical cyber domain agent
reports seven valid YAML frontmatters and scoped diff-check pass. Broad final
semantic/alias/source scan remains P12/P13, rather than a claim of zero cell words.

P12 discovery now being fixed: cyber launcher still emitted unsigned /v1/link and
lacked operator storage/auth commands despite soft3's migration/auth API. The
launcher agent reports actual native process tests passing and is completing
release build plus audit. Final integration and source-revision capture are still
pending. P07 sealed read/export and sizing/fault edges remain open; root is now
examining that operator path. Other remaining gates are recorded above.


### P12 cyber launcher verified — 2026-09-13

The source/packaging gap discovered during P11 is closed in cyber/src/{main,
connection}.rs, tests/node.rs and Cargo manifests. Explicit `storage import-legacy`
and `auth enable [--import-legacy]` call shared soft3 owners under the existing
node/auth-upgrade/BBG locks. They retain original bytes/authors/network and never
create custody automatically. Authentication is monotonic and explicit.

Descriptor v2 separates offline configuration-only metadata (unknown identity and
authentication are null) from `cyb --live` bounded validated capabilities. Only an
observed supported signed profile supplies v3 submission and explicit net pin.
Redirects, oversized responses and inconsistent profiles fail; local endpoint
acceptance remains distinct from consensus. Root read the final implementation
and audit, and the product-docs agent independently aligned all ten owned pages.

Agent evidence: p12-cyber-launcher-tests.log **1 unit + 6 real-process scenarios
passed**; p12-cyber-launcher-release.log release --locked build passed; the same
**6 process scenarios passed** against that release binary in release-tests.log.
cyber/audit/neuron-launcher.md records exact commands/scenarios and binary SHA-256.
Rust 1.95 is declared. All owned processes stopped, no commits/deployments.
This closes the launcher subtask, not whole P12/P13 release/registry/generated
context checks. P11 AOS/Cyberia/MiDAO canonical pages still being completed.

### P07 operator completion and P11 canonical domains — 2026-09-13

The previous P07 sealed inspection/export/sizing gap is closed. Keyless readonly
inspection validates full application history/receipts/content closure and
reports exact resources/unknown attempts/checkpoint compatibility, logical
digest and observational destination headroom. It preserves the application
transaction, including on sealed archives. Export retains/seals the source and
reuses bounded staged BBG pages; only explicit authenticated activation/import
creates target progs. Validation and sealing share one exclusive source owner.
Limits, failures, recovery commands and actual evidence are recorded in
[legacy-cutover.md](legacy-cutover.md), with an operator guide in neuron/docs.
Final runs: 9 CLI process tests, 6 substantive BBG parent scenarios plus one child
helper, 7 node legacy/migration/transfer tests using the original binary, and
4 shared-backend injected fault tests all passed. No production data touched.

All enumerated canonical P11 cyber/AOS/Cyberia/MiDAO pages are now updated and
validated. The compact cyber/specs/domain-ladder.md states subject/prog/service/
book/shard/network duties and preserved governance, conservation, finality and
availability obligations. The former cell page remains an explicit compatibility
landing; research titles retain historical naming where warranted. Twenty
AOS/Cyberia/MiDAO pages preserve service governance and foreign Cybernet message
fields (eight lines compared byte-for-byte). Seven canonical domain frontmatters,
twenty service frontmatters and new local links pass. Full/partial/light node
roles and launcher docs now agree with implemented signed native interfaces.
Neuron's accepted specs, cyb organ definitions, Crystal concepts and Satoshi
persona ownership were aligned with the same subject model. A final residual
scan also found old Prysm design-page terminology; its UI-only convergence is
being completed in the assigned P12 documentation scope.

Additional final verification discovered and fixed an independent returned-error
atomicity defect in Cybergraph::commit_signal: rejected nullifiers/lineage or
overflow could mutate chains. Six failures were reproduced first; eight exact
state regression tests now pass, with checked touched balances, batch nullifier
uniqueness and rollback of a tentative chain append. All nine historical stack
test binaries were migrated to actual supported APIs without disabling tests.
The default suite passed 57 tests; all-features passed 93. Recursive TensorMerkle
openings remain explicitly unsupported in zheng; supported public DirectProof
execution authenticates complete BBG reads at the selected root. Full evidence:
cybergraph/audit/neuron-stack-compatibility.md. TextArchive boundary work is
still running independently and requires its own closure results.

### P12 final source and packaging pass — 2026-09-13 (continuing)

cyb/audit/neuron-packaging.md now records completed CLI/registry/remote-script
work: `cy neuron-runtime` runs the standalone `neuron` binary while `cy neuron`
manages robot attachments. Child/unknown/missing-tool exit codes propagate;
readonly tool dispatch needs no graph/key. All16 CLI process/unit tests and8
build-script/closure checks passed; actual dry-run made zero transport calls.
Cargo metadata resolves24 sibling repos,82 local packages and87 manifests,
including previously omitted neuron and soft3. Remote source/artifact directories
are invocation-isolated and build failures propagate; no remote build/deploy ran.

Prysm's unused cyb-core dependency/re-export was removed (standalone check passed),
the unused landing asset moved from cyb/cells to cyb/pages, and remaining shell
GraphSession locals/comments now use session. Compatibility cell://landing stays
an explicit view route. The remaining Rune public driver is now run_events;
run_cell is a deprecated0.1 forwarder scheduled for0.2 removal, with no subject.
All84 rune-interp tests passed. Old neuron/docs reports moved into dated audit
files with current explanatory/compatibility pages. All216 local Markdown links
across42 neuron files pass; old deleted code links use verified original Git
revisions. The new local-links.nu checker supports percent-encoded paths.

Context generation now pins17 actual current owner documents, understands the
root-layout protocol graph, rejects missing pins/overbudget inputs/child failures,
and emits selected source hashes, omitted paths and an output checksum.42/194
candidate pages fit the64k estimate in the first pass. Final regeneration after
remaining docs edits is required. The script uses temporary outputs and preserves
the previous context on failure; source/output hash detects interrupted pair
replacement. The canonical cyber site built244 source pages into1484 output
pages; the public cyberia-blog build (now declaring neuron at /soft3/neuron)
built19122 graph pages into22164 output pages. These are local builds; no deploy.
Their final rerun must include remaining assigned Prysm/site/Foculus docs.

Strict neuron workspace all-target/all-feature Clippy passed after the navigation
parity check adopted is_multiple_of. Neuron all-feature tests passed48 cases,
including real original-binary migration and9 process tests. The isolated seven
WASM identity-consumer profiles and dependency-weight checks passed again.
Foculus NodeMode::Cell was found by broader semantic review and renamed Partial
with original middle-variant order/branch duties;5 live owner tests passed.
The old conformance scaffold's constant placeholder hash is explicitly excluded
from release-fingerprint evidence; actual owner suites/vectors remain required.

Release verification: first make dmg completed the macOS release compile, then
failed because Trunk parsed inherited NO_COLOR=1 as an invalid bool. Make apps now
normalizes NO_COLOR=true; actual Trunk release build passed. make android completed
the Rust library and signed release APK (48 Gradle tasks,11s final packaging).
Final DMG rebuild and source-fresh Android check remain pending after Foculus.
To make room, only reproducible debug incremental caches in cyb/neuron/prysm were
removed (about17 GiB); sources/keys/DB/model/fixtures/logs/artifacts were retained.

In progress now: final BBG all-feature suite, cyb-core suite, fresh cy/cyb debug
build for repeated Fleet and actual model CLI/GUI task rehearsal;64MiB TextArchive
boundary test and26 Prysm docs with protocol_domain;3 soft3 HTML pages with
protocol_remaining; final residual-review report with protocol_product.
Still required: final source rescan+every-repo dispositions/scoped compatibility
allowlist, regenerated context/site verification, release revision/feature/log
manifest and G01–G14 closure table. Goal is active; no completion claim yet.

### P13 closure — 2026-09-13

This entry supersedes the pending implementation/validation items above. The
final contract and gate review is [release.md](release.md), backed by
[validation-manifest.json](validation-manifest.json),
[source-manifest.json](source-manifest.json),
[final-repositories.md](final-repositories.md) and
[artifact-manifest.json](artifact-manifest.json). P00–P13 are implemented;
G01–G14 pass for the declared local profiles. The original inventory, intermediate
failures and earlier pending checkpoints remain historical evidence.

Final independent discovery accounts for 116 Git roots/markers and six non-Git
groups. All 122 receive dispositions; neuron inherits the Git-preserving cell
move. The selected source inventory hashes 12,902 files in 56 roots, including
all 87 manifests needed by the 24-repository cyb build closure. The 12,397 absent
Omi reference blobs and broken nu Git metadata remain explicitly named source
limits. Separate worktrees, foreign bytes, vendors and inactive generated copies
retain the plan's original preservation rules. Scoped runtime CellId/old package/
NodeMode::Cell checks return no hits. Git diff --check passes in all 55 valid
selected Git roots; nu cannot run Git checks because its preexisting gitdir is
missing. No source changes are attributed merely from unrelated dirty status.

TextArchive's final source preflight/transaction/projection limits and exact raw
legacy provenance pass four boundary, three existing and five GraphSession tests.
The largest suite exercised actual 8 MiB text and a 64 MiB projection, source
preflight before publication, a late CAS race, persistence error and reopen;
shared BBG fault injection separately covers storage failures. The migration
rehearsal retains the original executable and three-origin fixture, preserves
unknown effects, and resumes after real process loss at four durable boundaries.
The operator path is documented in neuron/docs/legacy-cutover.md. No production
store or identity mapping was invented or silently rewritten.

All 26 assigned Prysm pages and all remaining current soft3 pages/sites were
reviewed; the UI-only atom/molecule/view vocabulary follows current cyb anatomy.
Context was regenerated from actual owner pins, and all 43 input hashes plus the
output hash verify with zero stale entries. Both canonical sites rebuilt from
their current source declarations; neuron is mounted at /soft3/neuron. The exact
legacy API/URI/readers and their lifetimes are in compatibility.json.

Final owner suites pass: neuron 48; BBG 162; cyb-core 52; Cybergraph 57 default /
93 all-features before the separately tested final TextArchive patch; Rune 84;
cy CLI 16; Foculus five targeted mode scenarios; build closure/refusal eight;
seven isolated identity-only WASM profiles. Neuron all-target/all-feature Clippy
passes with warnings denied in its no-deps scope. P09 composition and P10 native/
foreign vectors retain their detailed owner evidence, including the real local
model and actual Rune suspend/result/recovery path.

Fresh debug cy/cyb/soft3 builds and a fresh cyber release build include all final
Foculus and archive source changes. Fleet passes 32 checks and stops all owned
processes; cyber's release executable passes six real-process scenarios (plus the
unit check). Actual CLI and release Bevy tasks execute the local model, persist
terminal results and reopen without another inference or attachment mutation.
Screenshots and non-secret task receipts are retained in p13-bodies. Binary
dependency files additionally verify no newer or missing compiled source inputs.

macOS release, web Trunk release and source-fresh Android release APK build pass.
The DMG required fixes for inherited NO_COLOR parsing, internal filesystem space
and a hardcoded bundle version. Verified bundles now report Cargo version 0.13.1;
the readonly mounted macOS executable/web asset match their build products, DMG
checksum passes and the owned mount is detached. APK ZIP, staged arm64 library,
signature and package/version checks pass. Artifact hashes and exact local build
profiles are retained. No remote builds, publication, deployment or notarization
are inferred. Full Hermes integration breadth remains the explicitly separate
agent roadmap in accepted plan section 11, rather than an unimplemented part of
this neuron/cell convergence.
