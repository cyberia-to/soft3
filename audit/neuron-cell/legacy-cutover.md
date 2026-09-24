# P07 retained-source migration audit — 2026-09-13

The operator path is implemented across BBG ApplicationArchive/TransferSource,
cybergraph Archive/Transfer, neuron ArchiveGraph and the CLI. Read-only inspection
has no mutation handle and performs no temporary application/index transaction.
Source validation and sealing share the same exclusive backend owner. Logical
export reuses the existing staged BBG transfer; activation and semantic mapping
remain explicit. See [operator guide](../../../neuron/docs/legacy-cutover.md).

| Boundary or failure | Executed evidence | Result |
|---|---|---|
| Original bytes, checkpoints and unknown attempts | Original compiled cell binary fixture; legacy_capture + legacy_integrity | Original v1 history remains readable; missing historical artifacts reject |
| Three origins into one authenticated target | neuron_migration, CLI process export/activation/import | All three progs, counter state 7, charges/reservations/claims and unknown outcome retained |
| Fresh and sealed inspection | application_transfer + CLI process | Same logical digest and last application transaction; no key creation or dispatch |
| Bounded partial pages and exact retry | transfer + application_transfer + process | Atomic cursor, resumable pages, idempotent complete retry, pinned target/key conflict |
| Real process loss | application_transfer child killed after seal, prepare, page and completion | Reopen source fenced; target cursor/head visibility agrees with phase; exact resume succeeds |
| Page validation failure | application_transfer callback panic caught before transaction | No cursor advance or partial page publication |
| Missing receipt/closure, limits, active owner | archive and migration tests | Errors propagate; malformed archive cannot complete/activate; busy source rejects |
| Bad destination before fresh seal | CLI process: existing file, same path | Existing bytes preserved; fresh source digest/transaction unchanged and unsealed |
| Source/target CAS and grant revocation | neuron_migration | No partial mappings/fences; denied activation does not release effects |
| Old writer entry points | actual legacy binary in node transfer/process tests | Retired source and new target reject original normal writer |
| Backend precommit read / write / sync / lost barrier acknowledgement | BBG native_tests with FaultFile and shared composed domains | Rollback or whole commit recovery; all views freeze after uncertain sync; exact retry deduplicates |

Logs under implementation-baseline:

- `p07-archive-process-tests.log`: 9 process tests passed, no ignored tests.
- `p07-archive-bbg-tests.log`: 7 harness cases passed, including one subprocess
  helper that is inert in the parent run; six substantive parent scenarios.
- `p07-archive-migration-tests.log`: 1 legacy capture + 1 integrity + 3 semantic
  migration + 2 separate-store transfer tests passed. NEURON_LEGACY_BINARY points
  to the preserved original executable, so the reader-fence checks actually ran.
- `p07-archive-storage-faults.log`: 4 backend fault tests passed.
- `p07-archive-check.log`: neuron-cli compilation passed.

The fault tests exercise the shared storage owner; they do not simulate literal
OS disk exhaustion during every export instruction. Real process loss covers
the four durable transfer boundaries. Disk sizing is observational headroom,
not reserved capacity. Read-only here means no logical application/seal/index
write; opening the SSD backend may perform its own recovery/maintenance.

This local profile is bounded: 256 source namespaces, 1M inspected rows, 8 GiB
logical archive, 512 rows per transfer page and 1–4096 pages per CLI call. Native
shards/record domains are rejected by this application-only archive reader.
Original cell/*/1 codecs remain immutable. No production store was migrated,
deleted or unsealed, and no process was left running by these tests.
