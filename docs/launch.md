---
title: launch spacepussy-test
tags: cyber, soft3, guide, launch
crystal-type: guide
crystal-domain: cyber
alias: launch network, soft3 node, start testnet, spacepussy-test guide
---
# launch spacepussy-test

how to run the **real soft3 network** — cybergraph + bbg as the chaosnet **spacepussy-test**.

this is not a status stub. the node processes cyberlinks into authenticated state, advances height on finalize, and serves product RPC.

## network identity

| field | value |
|-------|-------|
| name | spacepussy-test |
| chain_id | `spacepussy-test` |
| engine | cybergraph + bbg |
| protocol | `soft3/spacepussy-test/v1` |
| public rpc | `https://cyb.ai/spacepussy-test` |
| host | cyberproxy (cybernode edge) |
| local bind | `127.0.0.1:7780` |
| denom | `testpussy` |

cosmos **space-pussy** / **bostrom** on cybernode are bootloader chains (go-cyber). product tools reject those names.

## install

```bash
rustup update stable    # need rustc ≥ 1.85 (edition 2024 deps)
cargo install soft3     # operator / node
cargo install true-cyber  # product face (thin client)
```

## run a node (local)

```bash
soft3 node --home ~/.spacepussy-test --bind 127.0.0.1:7780 --moniker dev-1
```

what starts:

- cybergraph processor (intend/seal/link lifecycle)
- bbg authenticated state
- durable log at `$home/log` + block height at `$home/blocks`
- HTTP RPC on the bind address

### RPC

| method | path | role |
|--------|------|------|
| GET | `/status` | chain_id, moniker, height, bbg_root, signals, particles |
| GET | `/health` | `ok` |
| GET | `/root` | BBG root hex |
| GET | `/stats` | height, root, graph counts |
| POST | `/v1/link` | submit cyberlink JSON |
| POST | `/v1/finalize` | close block (advance height) |

### submit a cyberlink

labels are hemera-hashed; pure hex (≤64 chars) is left-padded identity.

```bash
curl -sS -X POST http://127.0.0.1:7780/v1/link \
  -H 'content-type: application/json' \
  -d '{"neuron":"01","from":"0a","to":"0b","amount":1,"valence":0,"finalize":true}'
```

response:

```json
{"ok": true, "height": 1, "root": "…", "signals": 1}
```

`finalize: true` (default) calls `bbg.finalize_block` after the link so height advances.

### probe

```bash
soft3 sync
# or, against local bind:
# soft3 does not yet take --rpc; use curl for local-only, public edge via soft3 sync
curl -sS http://127.0.0.1:7780/status | head
cyber sync                 # product client → public edge
```

## run the public chaosnet (cybernode)

host: **cyberproxy** (edge for cyb.ai).

```bash
# on an operator machine with soft3 0.6+
cargo install soft3 --force

# on cyberproxy
ssh cyberproxy
export PATH="$HOME/.cargo/bin:$PATH"
cargo install soft3 --force

# stop any legacy python scaffold
sudo systemctl stop spacepussy-test 2>/dev/null || true

# install systemd unit that runs soft3 node
sudo tee /etc/systemd/system/spacepussy-test.service >/dev/null <<'EOF'
[Unit]
Description=spacepussy-test soft3 node (cybergraph+bbg)
After=network.target

[Service]
Type=simple
User=cyber
Group=cyber
Environment=HOME=/home/cyber
Environment=PATH=/home/cyber/.cargo/bin:/usr/bin
ExecStart=/home/cyber/.cargo/bin/soft3 node --home /home/cyber/spacepussy-test/data --bind 127.0.0.1:7780 --moniker cyberproxy-spt
Restart=on-failure
RestartSec=3
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable --now spacepussy-test
sudo systemctl status spacepussy-test --no-pager | head -20
curl -sS http://127.0.0.1:7780/status | head
```

nginx (already on cyberproxy): `cyb.ai/spacepussy-test/` → `127.0.0.1:7780`.

```bash
# public
curl -sS https://cyb.ai/spacepussy-test/status | head
cyber sync
```

redeploy after soft3 upgrade:

```bash
ssh cyberproxy 'export PATH=$HOME/.cargo/bin:$PATH; cargo install soft3 --force; sudo systemctl restart spacepussy-test'
```

## multi-node (current limits)

v1 is **single-node cybergraph**. each `soft3 node` is an independent local processor with its own store.

not yet on the public chaosnet:

| capability | status |
|------------|--------|
| cybergraph link + bbg state | **live** |
| height / root / stats RPC | **live** |
| durable log | **live** |
| radio gossip between nodes | open (S4) |
| foculus multi-node φ* finality | open (S5) |
| seal STARK binding at commit | open (S3) |

next work is wire [[radio]] + [[foculus]] so two soft3 nodes exchange signals and share one tip — see [[cyber/launch]] S3–S6.

until then: one public node on cyberproxy is the product spacepussy-test; clients sync to it with `cyber sync` / `soft3 sync`.

## product client

```bash
cargo install true-cyber --force
export PATH="$HOME/.cargo/bin:$PATH"
cyber version    # cyber … (true-cyber)
cyber sync       # probes soft3 node on cybernode
```

true-cyber is a thin product face over the soft3 network. the network itself is the soft3 node.

## not soft3

| name | substrate | role |
|------|-----------|------|
| spacepussy-test | soft3 cybergraph+bbg | product chaosnet |
| space-pussy | cosmos go-cyber | bootloader experimental |
| bostrom | cosmos go-cyber | bootloader mainnet history |

## related

- [[soft3]] · [[soft3/status]] · [[install]] · [[bootloader]] · [[cyber/launch]]
