# Neuron/cell residual review

Read-only review started 2026-09-13 against
[the accepted plan](../../roadmap/neuron-cell-convergence.md),
[implementation ledger](implementation.md), and the 01:57–01:59 UTC
[final-scan observation](final-scan/scan.json). Source files were reread after
that scan because parallel work was still in progress. The one production gap
below was separately authorized for repair. This report does not close P13.

## Concrete findings

| Finding | Owner / exact path | Disposition and evidence |
|---|---|---|
| The public participation enum still exposed `NodeMode::Cell`, with active settlement matches and tests | `foculus/src/live.rs`; direct mode comments in `src/tip.rs`, `src/gossip.rs`; `README.md` and `docs/explanation/latency-targets.md` | Repaired during this review after explicit scope authorization. Docs changed first; code now uses `Partial`, preserving Full/Partial/Light variant order, all mode branches, settlement amounts and validation. No serializer, `repr`, CLI decoder or external `NodeMode::Cell` consumer was found, so no legacy decoding alias was invented. [Owner tests](implementation-baseline/p13-foculus-partial-mode.log): 5 passed, 0 failed, 139 deliberately filtered. This is targeted validation, not a full Foculus release claim. |
| Package metadata still described “soft3 cell runtime” although its README already described the neuron/prog host | `cyb/crates/cyb-reserve/Cargo.toml:8` | Fixed by the root owner during this review and reread: metadata now says “GUI shell with neuron and prog composition.” No runtime or persisted-data defect remains in this field. |
| Current Rs usage guidance still prescribed “cell-owned state” for RS505 | `hemera/CLAUDE.md:577` | Fixed by root during this review and reread: guidance now says “module-owned state or bounded channels.” No signature or codec changed. |
| Generated context matched its own recorded output hash but two selected inputs had changed | `ctx/ctx.sources.json`, `ctx/ctx.md`; selected inputs `cyber/CLAUDE.md`, `neuron/specs/migration.md` | G13 requires regeneration through `ctx/build.nu` after the final canonical edits. Both mismatches were verified by SHA-256 against the 43-entry source manifest. Editing ctx text alone would not satisfy this gate. |
| The ledger still lacks one final G01–G14 disposition table and a compatible working-copy/feature manifest | `soft3/audit/neuron-cell/implementation.md`; P12/P13 rows remain pending in its earlier status table | Required final release bookkeeping remains open. Later ledger entries supersede older checkpoints but do not supply one final per-gate verdict. Capture final source revisions plus dirty/untracked source fingerprints, lockfiles, actual feature/toolchain profiles, and links to the latest final tests. A clean HEAD list alone cannot identify the dirty sources tested. |

The first defect escaped the scanner's `domain` expression because that expression
does not include `NodeMode::Cell` or the full/cell/light mode triad. It was found
by reading broader `cell` matches in authored Rust. Consequently, zero remaining
`domain`/`dependency` hits would not by itself establish G12/G13 completion.

After the authorized Foculus change, this review found no other active authored
runtime `CellId`, `cyb_core::Cell`, `SharedCell`, `cell-*` package dependency or
unmapped `cell://` execution route among the inspected current matches. This is
a scoped source finding, not a universal claim about excluded upstream sources
or immutable history.

## Exact compatibility and KEEP dispositions

| Exact path or scoped set | Why it remains | Retirement / constraint |
|---|---|---|
| `neuron/model/schema-suite-v1.txt`; `neuron/model/src/records.rs` | Immutable v1 schema identities, old head/artifact/snapshot bytes and their bounded readers/builders | Indefinite history compatibility. Do not change `cell/*/1` tags or old particle hashes. These content records do not create a second signing subject. |
| `neuron/engine/src/legacy.rs`, `legacy/records.rs`, `legacy/artifacts.rs`; legacy branches of `neuron/node/src/validation.rs` and `neuron/cli/src/main.rs` | Inspection, closure validation, provenance and reconciliation of original records | Read/import compatibility under explicit mapping and authority. No blanket legacy writer or automatic CellId→NeuronId conversion. |
| `neuron/node/tests/support/legacy.rs`, `legacy_integrity.rs`; `soft3/audit/neuron-cell/implementation-baseline/capture-legacy-v1.rs`, `cell-Cargo.lock`, fixture directories | Original-codec/source/binary migration evidence | Preserve original bytes and names. Do not count fixture package names as current dependencies. |
| `neuron/model/src/navigation.rs`; `neuron/specs/navigation.md`; `cyb/shell/src/shell/chrome.rs`; `cyb/shell/src/worlds/robot/mod.rs` | Explicit legacy URI resolver, route handling and positive/negative tests | Only `cell://landing` maps to the robot view by default. Arbitrary legacy runtime IDs require an explicit validated import mapping; unknown, traversal and query-tail forms reject. Routes do not attach, sign or execute. |
| `cyb/parts/cells.md`, `cyb/reference/routing.md`, `cyb/reference/scripting.md`; `prysm/chroma/specs/{com,space}.md`, `prysm/molecules/specs/{launcher,stars}.md` | Compatibility documentation for the same bounded URI behavior | Keep the historical landing path and exact resolver contract. Do not restore independent active/pinned cell identity. |
| `rs/core/src/core_types.rs`, `rs/core/src/lib.rs`, `rs/macros/src/lib.rs`; `rs/tests/macro-integration/tests/module_lifecycle.rs` | Deprecated `Cell`, `CellMetadata`, `cell!` forward to `Module`, `ModuleMetadata`, `module!`; test verifies the delegation | Since 0.1.0; explicitly removed in 0.2. No extra ID or identity layer. |
| `rune/rs/interp/event.rs`; `rune/specs/checkpoints.md` | Deprecated `run_cell` delegates to current `run_events` with the existing event/checkpoint semantics | 0.1 compatibility forwarder; removal in 0.2. `rune/CHANGELOG.md` remains historical. |
| `cyber/cell.md`; references in `cyber/research/oikos.md` and `cyber/shard.md` | Exact historical graph aliases map runtime→neuron/prog, knowledge→shard, ledger→book/issuer, building→service | Keep aliases, governance, validation/completeness, money and finality duties; the alias does not assert one subject at all layers. |
| `neuron/audit/*`; `neuron/docs/cell-convergence.md` and the linked archived proposal | Audits preserve original observations; current convergence landing uses the accepted model | Keep historical filenames where links depend on them. Current claims must follow the new specs. |
| `rune/rs/ast/lib.rs`, `rune/rs/lower/lib.rs`, `eidos/rs/src/certificate.rs`, `trident/src/ir/tree/lower/mod.rs` and their Noun tests | VM/Noun pairs and literal constructors | KEEP in this migration; separate substrate terms work, never rename to Neuron. |
| `cybergraph/src/native/tests.rs`, `trident/src/diagnostic/mod.rs`, `nika/src/constraint_eval_gpu.rs`, `mona/crates/randomx/src/vm/jit/mod.rs` | `std::cell::Cell` or equivalent interior mutability, including thread-local test fault switches | KEEP. |
| `honeycrisp/unimem/src/grid.rs`; `bita/crates/rbtc-hash5/src/echo.rs`; `foculus/specs/vec.md`; `hemera/rs/src/tree.rs` | Memory/grid cells, ECHO hash blocks, availability sampling and tree coordinates | KEEP; no subject identity. |
| `nu/crates/nu-{term-grid,table,protocol,command}/*`; visual cell-grid prose in `cyb/reference/rendering.md`; `radio/iroh-blobs/docs/img/get_machine.drawio` | Upstream table/path/grid APIs, rendering units and draw.io nodes | KEEP; author-owned adapters already use GraphSession/typed routes where relevant. |
| `trident/catalog/os/{nervos,ton}/*`, `trident/catalog/vm/tvm/*`; foreign Cosmos/Neptune/CKB/TON structures and vendor trees | Foreign protocol/storage conventions | KEEP their exact types, strings and encodings. |

## Repository move, source aliases and generated copies

- The canonical runtime checkout is `~/cyber/neuron`; `~/cyber/cell` is absent.
  Its retained Git HEAD is `34bbff9dbf36421a45c5c283e6c1336dcafc997a`, with dirty
  migration sources. This is the former repository's continuation, not a new
  empty identity wrapper. `neuron/.git/config` points to
  `https://github.com/cyberia-to/neuron.git`; this review verifies local
  configuration, not remote publication or a remote rename operation.
- `cyberia-blog/subgraphs.toml` now names `neuron`, parent `soft3`, producing
  `/soft3/neuron` from local `~/cyber/neuron`. The publisher resolves `repo`
  override or `name`; no old cell declaration remains. All declared names and
  derived mounts are unique. Existing `tok`→remote `plumb` is an intentional
  source alias, unrelated to this merge, and must remain.
- `cyberia-blog/subgraphs.lock.toml` is an older organization census, dated
  2026-08-12, with neither cell nor neuron. It is not an immutable release
  revision manifest. Do not silently treat it as proof of a published neuron
  revision. The local source declaration and current origin agree; remote
  clone/publication readiness still needs its normal read-only release check.
- `.worktrees/soft3-thread-publication` is a separate clean branch snapshot:
  `docs/thread-publication` at `4fc77ad335712915bd39714d72856ada1da0a52f`.
  Its old site package strings belong to that branch. It was not edited,
  rebased or reset; a future publication from it must first intentionally
  integrate the accepted current source revisions.
- `cyber/cyber/build/**` and `cyberia-blog/build/**` are generated snapshots,
  including six generated `.git` marker trees in the scan. They are not source
  checkouts to mass-edit. Rebuild from corrected owners when that output profile
  is part of the release. The review did not rebuild or deploy them.
- `cyb/cell` contains only `target/`, no manifest or source; it is an old ignored
  build-cache path, not a parallel component. `honeycrisp/.claude/worktrees/strata`
  resolves inside the workspace to the existing `strata` checkout; no cell rename
  conflict was found there.
- Baseline `dispositions.json` still keys the original repository as `cell`.
  For the final G01 map, carry that repository's implementation disposition to
  `neuron`; preserve the original baseline record rather than pretending a
  second subject repository was discovered.

## Gates versus observed evidence

This is a closure checklist against the plan, not a new reduction of its scope.

| Gate | Evidence or remaining condition |
|---|---|
| G01 | Scan: 118 entries = 104 repositories, 7 worktrees, 6 generated markers and one broken Git checkout (`nu`), plus 6 non-Git source groups. Preserve the named gaps: 12,397 unavailable sparse-index blobs in `omi/refs/omi-upstream`; its 12,478 missing/gitlink paths are not reviewed authored source. Final dispositions must include the local cell→neuron move and explicit snapshot/vendor/generated exclusions. |
| G02 | Current cyb/neuron/soft3 contracts have one subject plus program/work records. Historical “no CellId” explanations and graph aliases are not conflicting definitions. |
| G03 | Ledger records old suite vectors and new record validation; legacy tag paths above are intentionally retained. Final manifest must identify the exact tested suite/reader revisions. |
| G04 | Ledger/consumer audit records identity-only builds and vectors; minimal ID/API reexports remain separate from engine/GUI. Do not infer this from the broad unified workspace dependency tree alone. |
| G05 | Shared storage and graph-session evidence exists; the separately repaired API atomicity has eight regressions and default/all-features owner runs. Later TextArchive edits require their own final results, not the older 57/93 suite totals. |
| G06 | Ledger records multiple prog/task execution, immutable bindings, resource recovery and stale-result tests. No new naming defect was found in those current engine paths. |
| G07 | Ledger records current grant/revocation, captured network/subject, stale-writer and custody checks. The supported profile is fenced local publication, not a distributed multi-writer guarantee. Final shell/fleet results must correspond to the final integration sources. |
| G08 | [Retained-source migration audit](legacy-cutover.md) records original-binary fixtures, three origins into one subject, unknown-attempt preservation, real process loss at four durable transfer boundaries and backend fault tests. Its bounded archive/application profile and observational disk headroom remain explicit. |
| G09 | Registry/process/typed-route evidence records two keys/networks, watch-only identities and device bindings. Exact compatibility routes above do not select or mint an account. Final graphical integration validation remains the root owner's closing run. |
| G10 | Latest ledger entries supersede earlier “not implemented” checkpoints and describe actual Soma tool/restart/result, parent/child, steering/context/model and scheduling scenarios. Use those final composition logs, not an isolated engine rename test. |
| G11 | [Consumer disposition](consumers.md) records the supported native/foreign vectors and seven isolated WASM identity consumers; foreign bytes remain unchanged. |
| G12 | Canonical domain duties are preserved. The newly found Foculus mode defect is now repaired and its five owner scenarios pass; ordinary biological/VM/memory/foreign cells remain unchanged. |
| G13 | CLI/package/build-closure evidence exists in `cyb/audit/neuron-packaging.md`; source aliases and compatibility paths are reviewed above. Context input hashes were stale at inspection and must be regenerated; final output/profile fingerprints still need recorded closure. The two current metadata/guidance findings above were fixed and reread. |
| G14 | Open until the final per-gate matrix and compatible source/features manifest are recorded. Explicitly separate supported public direct execution/native openings from unsupported recursive TensorMerkle, and local endpoint acceptance from consensus finality. No remote Linux/Windows release or distributed writer result was produced by this review. |

The complete scan's counts, chronological ledger and this residual review have
different purposes. None substitutes for the final profile-specific evidence
manifest. No large suites, remote builds, SSH, publishing, commits or deployments
were performed for this review; the only additional test run was the five-case
Foculus owner check after the authorized source correction.
