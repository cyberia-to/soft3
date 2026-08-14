---
title: launch spacepussy-test
tags: cyber, soft3, guide, launch
crystal-type: guide
crystal-domain: cyber
alias: launch network, soft3 node, start testnet, spacepussy-test guide
---
# launch spacepussy-test

how to stand up the soft3 product network — the chaosnet named **spacepussy-test**.

this is the day-one network for [[soft3]], [[true-cyber|cyber]], and [[cyb]]. tokens and state here are test. mainnet is a later gate; see [[cyber/launch]].

## network identity

| field | value |
|-------|-------|
| name | spacepussy-test |
| chain_id | `spacepussy-test` |
| role | soft3 chaosnet · product default |
| rpc | `http://127.0.0.1:7780` |
| lcd | `http://127.0.0.1:7781` |
| index | `http://127.0.0.1:7782` |
| denom | `testpussy` |

aliases accepted by the CLI: `spacepussy-test`, `test`, `soft3`, `sptest`.

```bash
cargo install true-cyber
cyber network
# network spacepussy-test
#   role     soft3 chaosnet (product default)
#   rpc      http://127.0.0.1:7780
```

## two chains with similar names

| name | substrate | what it is |
|------|-----------|------------|
| spacepussy-test | soft3 | product chaosnet — this guide |
| space-pussy | cosmos-sdk / [[go-cyber]] on cybernode | bootloader experimental chain — migration source |

`cyber sync -n space-pussy` and `soft3 sync -n bostrom` are rejected on purpose. bootloader history lives under [[bootloader]]; product tools never default to cybernode cosmos RPC.

## goal of a live network

two or more independent nodes converge on one [[cybergraph]]:

- signals carry proofs
- gossip moves signals between peers
- [[foculus]] orders and finalizes by φ*
- [[bbg]] holds authenticated state
- [[cyber sync]] / `soft3 sync` report the node reachable on port 7780

that package is the soft3-node + genesis work (milestone S6 in [[cyber/launch]]). pieces exist as crates today; the single binary that wires them is the remaining spine.

## prerequisites

```bash
# rust stable
rustc --version

# product CLI (probe + network presets)
cargo install true-cyber
# or stack facade
cargo install soft3

# workspace root for building components from source
export CYBER_ROOT="${CYBER_ROOT:-$HOME/cyber}"
# expected layout: $CYBER_ROOT/{soft3,cybergraph,foculus,radio,bbg,...}
```

optional advanced bootstrap of the full toolset:

```bash
cyber source                 # clone public stack repos into $CYBER_ROOT
cyber tools                  # list registered tools
# cyber install cybergraph   # when you need component CLIs on PATH
```

## status today

| surface | status |
|---------|--------|
| network presets (`spacepussy-test`, ports 7780–7782) | shipped in soft3 0.4 / true-cyber 0.3 |
| `cyber sync` / `soft3 sync` probe | shipped — reports reachable / offline |
| local [[cybergraph]] processor + store | usable — single-node link / query / root |
| [[foculus]] consensus tools + device sync daemon | usable — not yet the product RPC on 7780 |
| [[radio]] gossip | builds — signal membrane onto gossip open |
| soft3-node binary (one process, product ports) | not shipped — S6 |
| genesis tooling | not shipped — S6 |

when nothing listens on 7780:

```bash
cyber sync
# cyber sync · spacepussy-test
#   role             soft3 chaosnet (product default)
#   rpc              http://127.0.0.1:7780
#   reachable        no
#   detail           no soft3 node at http://127.0.0.1:7780 …
```

that is the correct offline signal. do not point product sync at cosmos endpoints to make the probe green.

## path A — local graph (available now)

run the cyberlink processor against a local store. this exercises intend / seal / link / query / root on one machine. it is not yet multi-node consensus and does not bind product ports 7780–7782 until soft3-node exists.

```bash
cd "$CYBER_ROOT/cybergraph"
cargo build --release -p cybergraph-cli   # package name may be cybergraph; see crate Cargo.toml

# ephemeral (in-memory)
cybergraph link --neuron alice --from cat --to dog

# durable store
mkdir -p /tmp/spacepussy-test-store
cybergraph --store /tmp/spacepussy-test-store link --neuron alice --from cat --to dog
cybergraph --store /tmp/spacepussy-test-store stats
cybergraph --store /tmp/spacepussy-test-store root
cybergraph --store /tmp/spacepussy-test-store chain alice
```

useful operators:

| command | role |
|---------|------|
| `cybergraph intend …` | declare an unsealed intent |
| `cybergraph seal …` | seal intent into a signal |
| `cybergraph link …` | one-shot submit (auto chain fields) |
| `cybergraph query '<datalog>'` | [[inf]] query over the graph |
| `cybergraph finalize` | advance root + height |
| `cybergraph root` | print BBG root |

seal binding and proof verification at the commit port are still open (S3). local processing is the scaffold the soft3-node will wrap.

## path B — foculus consensus tools (available now)

drive φ*, fork-choice, and finality by hand, or run the device-sync daemon (QUIC, default port 4200 — a foculus sync port, separate from product RPC 7780).

```bash
cd "$CYBER_ROOT/foculus"
cargo build --release

# core consensus CLI (no network)
foculus --help

# device sync daemon (feature `net` when required by the build)
foculus node --dir ~/.foculus-a --port 4200 daemon
# second peer on another port / dir, then add-peer + sync
```

foculus is the ordering and finality engine inside the future soft3-node. it is a component surface, not the product chaosnet endpoint.

## path C — soft3-node (target)

the product launch command once S6 lands:

```bash
# planned — not shipped yet
soft3 node --network spacepussy-test --home ~/.spacepussy-test
# listens: rpc :7780 · lcd :7781 · index :7782

cyber sync
# reachable yes · chain_id spacepussy-test
```

### what soft3-node wires

```text
                    soft3-node (one binary)
    ┌──────────────────────────────────────────────────┐
    │  rpc :7780   lcd :7781   index :7782             │
    │                                                  │
    │  cybergraph ──► bbg state                        │
    │       │              ▲                           │
    │       ▼              │                           │
    │  tape frames ──► radio gossip ──► peers          │
    │       │                                          │
    │       ▼                                          │
    │  foculus order + φ* finality                     │
    │       │                                          │
    │       ▼                                          │
    │  soma drive-loop: fetch → execute → prove → commit│
    └──────────────────────────────────────────────────┘
```

| layer | crate | job in the node |
|-------|-------|-----------------|
| process | [[cybergraph]] | intend / seal / link · signal chains |
| state | [[bbg]] | authenticated polynomial state |
| frame | [[tape]] | particle / cyberlink / signal on the wire |
| transport | [[radio]] | QUIC + gossip |
| order / finality | [[foculus]] | per-neuron chains · φ* · fork choice |
| runtime | [[soma]] | fetch → execute → prove → commit loop |
| prove | [[nox]] · [[zheng]] | execution + proof |
| identity | [[mudra]] | keys · signatures on signals |
| rank | [[tru]] | φ* over the live graph |

### genesis (planned)

```bash
# planned
soft3 genesis init --network spacepussy-test --out ~/.spacepussy-test
soft3 node --home ~/.spacepussy-test
```

genesis root starts the network. mainnet later migrates bootloader graph state (space-pussy rehearsal, then bostrom) onto soft3 — see R-milestones in [[cyber/launch]]. genesis of spacepussy-test itself is empty-or-seeded soft3 state, not a cosmos export.

### multi-node (planned)

```bash
# node A
soft3 node --home ~/.spt-a --rpc 7780 --p2p 7783

# node B
soft3 node --home ~/.spt-b --rpc 7790 --p2p 7793 --peer <A-multiaddr>

cyber sync                    # A
cyber sync --rpc http://127.0.0.1:7790   # B — when flag exists
```

gate: both nodes finalize the same root; a signal on A appears on B before finality.

## milestone ladder (network path)

from [[cyber/launch]] — only the network-relevant slice:

| stage | name | gate for launch |
|-------|------|-----------------|
| S1 | wire & framing | [[tape]] particle/cyberlink/signal round-trip, schema frozen |
| S2 | identity | signal signatures via [[mudra]] verified at order |
| S3 | proven processor | seal binding: unproven signal rejected |
| S4 | networking | signals on [[radio]] gossip · [[foculus]] order across peers |
| S5 | consensus v0 | multi-node φ* finality · no conflicting finals |
| S6 | node + genesis | soft3-node boots · genesis root · peers join |
| ★ | MVP testnet | multi-node public · 30 days · live economy |

spacepussy-test is the name of that MVP testnet as a product network. shipping soft3-node is the hard gate that turns `cyber sync` green by default.

## ports

| port | service | now |
|------|---------|-----|
| 7780 | product RPC (soft3-node) | reserved · offline until S6 |
| 7781 | product LCD / REST | reserved · offline until S6 |
| 7782 | product index | reserved · offline until S6 |
| 4200 | foculus device-sync default | component daemon only |

keep 7780–7782 free for the product node so `cyber sync` stays one command for everyone.

## operator checklist

1. install `true-cyber` (or `soft3`)
2. confirm presets: `cyber network` → chain_id `spacepussy-test`, rpc `127.0.0.1:7780`
3. run path A (cybergraph) and/or path B (foculus) while developing components
4. do not point product CLIs at cybernode cosmos RPC
5. when soft3-node ships: `soft3 node` then `cyber sync` → reachable yes
6. multi-node → public spacepussy-test → MVP gate in [[cyber/launch]]

## related

- [[soft3]] — stack entry
- [[soft3/stack]] — component registry
- [[soft3/docs|foundations]] — one mind · many languages · open world
- [[install]] — product install (`cargo install true-cyber`)
- [[bootloader]] — cosmos bostrom / space-pussy as migration sources
- [[cyber/launch]] — full milestone ladder to mainnet
