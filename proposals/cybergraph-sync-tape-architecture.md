---
tags: cyber, soft3, architecture, proposal
crystal-type: process
crystal-domain: cyber
status: implemented
date: 2026-06-07
---
# Architecture upgrade: cybergraph + sync + tape

Reshape the network layer of the soft3 stack so that cybergraph is a unified link processor with one API, sync owns the full structural-sync protocol, tape becomes a first-class stack member for wire framing, and intent is promoted to a first-class verb alongside signal.

## Goals

1. One API surface for soma — `declare / submit / subscribe / query` all through cybergraph.
2. No read/write split across repos.
3. sync owns *all* structural-sync mechanics (validity, ordering, availability, local merge) — not just availability.
4. tape becomes the wire substrate; radio carries tape frames; sync and cybergraph mint frames in their dialect.
5. intent is recognised everywhere — bbg persists it, sync validates it, cybergraph exposes `declare()` for it.

## Final architecture

```
                    soma (think)
                       │
                       ▼ ── ONE API: declare / submit / subscribe / query
                  cybergraph (link)
                       │
       ┌───────────────┼───────────────┐
       ▼               ▼               ▼
     bbg            sync             radio
   (store)         (sync)         (transmit)
                       │               │
                       └──── uses ─────┤
                                       ▼
                                     tape
                                    (frame)
```

Backend ↔ backend connections (no funnel through cybergraph):

- `sync ↔ bbg` — sync reads `BBG_root` and requests Lens openings for completeness verification.
- `sync ↔ radio` — sync chunks signals/intents, hands frames to radio for gossip; radio delivers peer frames to sync.
- `radio ↔ tape` — radio carries tape-framed bytes; tape is the codec.

Forbidden: `bbg ↔ radio` direct (bbg is pure state, never network), `cybergraph ↔ radio` direct (cybergraph only sees validated objects, never raw frames).

## Backend contracts

```rust
// bbg (store) — pure state library, no network
trait Bbg {
    fn apply(&mut self, links: &[Cyberlink], height: u64) -> StateUpdate;
    fn apply_intent(&mut self, intent: &Intent) -> StateUpdate;
    fn apply_signal_record(&mut self, s: &Signal) -> StateUpdate; // header only
    fn get(&self, dim: u8, key: &[u8; 32]) -> Option<&[Goldilocks]>;
    fn open(&self, dim: u8, key: &[u8; 32]) -> LensOpening;
    fn root(&self) -> BbgRoot;
}

// sync (sync) — structural-sync protocol; uses bbg + radio
trait Sync {
    fn validate_intent(&self, i: &Intent) -> Result<(), SyncError>;
    fn validate_signal(&self, s: &Signal) -> Result<(), SyncError>;
    fn order_and_chain(&mut self, s: &Signal) -> ChainPosition;
    fn broadcast_intent(&self, i: Intent);
    fn broadcast_signal(&self, s: Signal);
    fn on_peer_frame(&mut self, f: TapeFrame) -> Option<SignalOrIntent>;
    fn verify_completeness(&self, peer: PeerId, dim: u8) -> Result<(), SyncError>;
}

// radio (transmit) — bytes via tape frames
trait Radio {
    fn gossip(&self, f: TapeFrame);
    fn request(&self, peer: PeerId, f: TapeFrame) -> impl Future<Output = TapeFrame>;
    fn on_frame(&self, handler: impl Fn(TapeFrame));
}

// tape (frame) — pure codec, no runtime
trait Tape {
    fn encode(t: u8, data: &[u8]) -> TapeFrame;
    fn decode(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])>; // (type, data, remainder)
    fn register_dialect(urn: &str, decoder: DialectDecoder);
}
```

## Stack table updates

Add two rows; keep cybergraph at #6. Final ordering:

| # | repo | verb | release |
|---|---|---|---|
| 0 | strata | math | — |
| 1 | hemera | hash | v0.2.0 |
| 2 | lens | commit | — |
| 3 | trident | compile | v0.1.0 |
| 4 | nox | run | — |
| 5 | zheng | prove & verify | — |
| 6 | cybergraph | link | — |
| 7 | bbg | store | — |
| 8 | tru | converge | — |
| 9 | glia | infer | — |
| 10 | mir | render | — |
| 11 | mudra | encrypt | — |
| 12 | radio | transmit | — |
| 13 | tape | frame | — |
| 14 | sync | sync | — |
| 15 | foculus | agree | — |
| 16 | soma | think | — |
| + | rune | eval | — |

Renumbers foculus from 13→15 and soma from 14→16 to slot tape (13) and sync (14) into the network triplet.

## subgraphs.toml diff

```toml
# Add under soft3 parent block:

[[subgraph]]
name = "tape"
parent = "soft3"
visibility = "private"
```

`sync`, `radio`, `bbg`, `cybergraph`, `foculus`, `soma` are already present.

## Per-repo migrations

### bbg

| change | detail |
|---|---|
| add dimension | `dim::INTENTS = 13` (ephemeral was 12; intents persistent — re-number) |
| add method | `apply_intent(intent: &Intent) -> StateUpdate` |
| add method | `apply_signal_record(s: &Signal) -> StateUpdate` (header only) |
| spec update | `specs/state.md` — new intents dim, contract for apply_intent |

Commit: `feat: add intents dimension + apply_intent/apply_signal_record`

Note: collision with existing `EPHEMERAL = 12`. Resolve by `INTENTS = 13`, leaving room to grow. Update all backends to support the new dim count (mem, unimem, fjall, redb, tiered).

### cybergraph

| change | detail |
|---|---|
| move `src/chain.rs` | → `sync/src/chain.rs` |
| move `src/vdf.rs` | → `sync/src/vdf.rs` |
| move `specs/sync.md` | → `sync/specs/sync.md` |
| add `declare(intent)` | unsealed-signal API; calls `sync.validate_intent` + `bbg.apply_intent` + `sync.broadcast_intent` |
| keep `submit(signal)` | rewires to `sync.validate_and_order` + `bbg.apply` + `sync.broadcast_signal` |
| keep `subscribe(filter)` | local event bus |
| keep `query(inf_script)` | inf relations |
| update README | API now has 4 verbs; remove "manages conviction UTXOs" language already replaced |
| update tests | rewire integration tests to use sync's chain primitives |

Commits (atomic):
1. `refactor: move chain.rs and vdf.rs to sync repo`
2. `refactor: relocate sync.md cross-cutting spec to sync repo`
3. `feat: add declare() intent API; rewire submit through sync`

### sync

| change | detail |
|---|---|
| receive | `chain.rs`, `vdf.rs` from cybergraph |
| receive | `sync.md` cross-cutting protocol spec |
| add module | `src/protocol.rs` — top-level `validate_intent`, `validate_signal`, `order_and_chain`, `on_peer_frame` |
| add module | `src/broadcast.rs` — `broadcast_intent`, `broadcast_signal` via radio |
| add module | `src/completeness.rs` — `verify_completeness` via bbg Lens openings |
| update | `Cargo.toml` to depend on bbg, radio, tape |
| spec | `specs/protocol.md` explaining how the 5 layers compose |

Commits:
1. `feat: receive chain + vdf from cybergraph; add validate APIs`
2. `feat: add broadcast and completeness modules`
3. `docs: add structural-sync protocol spec`

### tape

| change | detail |
|---|---|
| add | `Cargo.toml` (if not already a Rust crate — currently has impl/) |
| reference | from radio's Cargo.toml |
| spec | `spec/0-overview.md` already exists — add cyber-dialect section pointing at sync's types |
| nothing destructive | tape already exists and is well-specced |

Commit: `docs: document cyber dialect type registry (signal, intent, chunk)`

### radio

| change | detail |
|---|---|
| add | dependency on tape |
| add | `on_frame(handler)` API for sync to register a callback |
| update spec | `specs/` (if exists) — note tape framing on the wire |

Commit: `feat: integrate tape framing; expose on_frame handler API`

### soma

| change | detail |
|---|---|
| update | `soma-spec.md` §7 "The cybergraph interface" — 4 verbs (declare/submit/subscribe/query) instead of 3 |
| update | diagram in soma-spec to show fan-out (bbg, sync, radio) under cybergraph rather than peer arrows |
| update | `soma.md` overview to mention intent declaration |

Commit: `docs: update cybergraph interface for declare() and corrected fan-out`

### soft3

| change | detail |
|---|---|
| update | `README.md` stack table — add sync (#14, verb `sync`) and tape (#13, verb `frame`); renumber foculus (15), soma (16) |
| update | `README.md` diagram showing soma → cybergraph → {bbg, sync, radio → tape} |
| add | `proposals/cybergraph-sync-tape-architecture.md` (this file) |

Commits:
1. `docs: add cybergraph-sync-tape architecture proposal`
2. `docs: update stack table and diagram for sync/tape/soma`

### cyber/subgraphs.toml

| change | detail |
|---|---|
| add | `tape` under `soft3` parent |

Commit: `chore: add tape to subgraphs under soft3 parent`

### inf

No changes. inf is already the query language; no impact.

### foculus

No changes in this proposal. foculus owns layer-5-global (finality), which is unaffected.

## Migration sequence (dependency-ordered)

Each phase is independent within itself; later phases depend on earlier ones.

### Phase 1 — paperwork (no code moves)

1. Commit this proposal to soft3/proposals/
2. Update subgraphs.toml (add tape)
3. Update soft3/README.md (stack table + diagram)

### Phase 2 — bbg gains intents dimension

1. Add `dim::INTENTS = 13`, expand all backends to 14 dimensions
2. Add `apply_intent` and `apply_signal_record` methods
3. Update specs/state.md
4. All tests pass

### Phase 3 — sync absorbs chain.rs + vdf.rs

1. Move chain.rs and vdf.rs from cybergraph to sync
2. Update sync's Cargo.toml dependencies
3. Update sync's lib.rs to export ChainPosition, SignalChain, VdfProof
4. Add `validate_intent`, `validate_signal`, `order_and_chain` as public API
5. Move `specs/sync.md` from cybergraph to sync
6. Cybergraph's lib.rs drops the chain/vdf modules and imports them from sync

### Phase 4 — cybergraph gets the 4-verb API

1. Add `declare(intent)` to cybergraph public API
2. Rewire `submit(signal)` through `sync::order_and_chain`
3. Update integration tests if they reach into chain.rs internals
4. Update specs and README

### Phase 5 — radio + tape integration

1. Radio depends on tape
2. Sync uses tape encoding/decoding for outbound/inbound frames
3. Document the cyber dialect type registry (frame type bytes)

### Phase 6 — documentation pass

1. Update soma-spec §7 cybergraph interface
2. Update soma.md overview
3. Sweep cybergraph specs/ for stale references

## Out of scope (explicitly NOT in this upgrade)

- foculus changes — layer-5-global stays as-is for now
- inf or query schema changes — already done in prior commit
- Moving cybergraph's data-structure specs (cyberlink.md, particle.md, neuron.md) — they stay
- Moving identity.md, model.md out of cybergraph — separate audit, not blocking
- Spec audit cleanup (delete axon.md, merge valence/amount) — separate task
- soma implementation work — soma stays in spec phase
- The tru ↔ cybergraph contract for focus/karma read-through — separate workstream

## Risks and mitigations

| risk | mitigation |
|---|---|
| Phase 3 breaks cybergraph's existing tests | Run the full `stack_*` integration suite after every phase commit |
| `dim::INTENTS = 13` collides with existing EPHEMERAL slot in bbg backends | This proposal explicitly numbers INTENTS = 13, EPHEMERAL stays at 12, expand arrays to 14 — safe |
| sync's `on_peer_frame` introduces a new ingress path that wasn't tested | Add a smoke test where a tape frame is decoded into a Signal and ingested |
| Cross-repo coordination (changes land in different repos at different times) | Use the migration sequence above; each phase is atomic per repo; no in-flight cross-repo breakage if order is followed |
| README and spec drift | Phase 6 explicit documentation pass; CI lint that flags stale "structural-sync" mentions in cybergraph |

## Acceptance criteria

1. `cargo test` passes across cybergraph, sync, bbg, radio, tape after Phase 5.
2. soma-spec §7 diagram matches the final architecture diagram in this proposal.
3. `soft3/README.md` stack table contains 17 rows (0–16 + rune), with sync at #14 verb `sync`, tape at #13 verb `frame`.
4. `subgraphs.toml` contains a `tape` entry under `soft3` parent.
5. Cybergraph's `src/` no longer contains chain.rs or vdf.rs; sync's `src/` contains them and exposes `SignalChain`, `VdfProof`, `validate_intent`, `validate_signal`, `order_and_chain`.
6. Cybergraph's public API has four verbs: `declare`, `submit`, `subscribe`, `query`.
7. bbg has `dim::INTENTS` and `apply_intent` + `apply_signal_record` methods.

## Estimated effort

- Phase 1 (paperwork): 1 pomodoro
- Phase 2 (bbg intents): 2 pomodoros
- Phase 3 (sync absorbs chain+vdf): 3 pomodoros
- Phase 4 (cybergraph 4-verb): 3 pomodoros
- Phase 5 (radio+tape): 2 pomodoros (mostly Cargo wiring)
- Phase 6 (docs pass): 2 pomodoros

Total: ~13 pomodoros = ~2 sessions of focused work.
