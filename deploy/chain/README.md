# deploy/chain — the phase-1 root chain on three cybernode machines

Runbook for running `soft3 node` as the proof-of-work root chain (decision
2026-10-15, cyber/launch.md; genesis 2026-11-05) on cyberproxy, deimos and
jupiter, node-to-node over cyber-radio (QUIC/UDP) and the HTTP API behind
the existing nginx. Prepared 2026-10-15 while the chain spec
(`soft3/specs/chain.md`) is still being written; § assumptions lists what
this runbook takes from the spec and must be re-checked when it lands.
Nothing here starts a service by itself.

Rehearsed on the three machines on 2026-10-16 as chain `pussy-rc` (receipt
`soft3/audit/chain/rehearsal-2026-10-16.md`); every correction that run
forced is folded into the steps below and listed in § what the rehearsal
changed. The blockers it found (two for launch, one for the wallet) are in § blockers.

Machine facts behind every number: `cybernode/servers/<host>/inventory-2026-10-15.md`.

## files

| file | installs to | role |
|--|--|--|
| `soft3-chain@.service` | `/etc/systemd/system/` | one unit per chain instance (`soft3-chain@pussy`, `soft3-chain@bostrom`) |
| `chain.env.example` | `/etc/soft3/<chain>.env` | the instance parameters (bind, radio port, moniker, thresholds) |
| `limits.conf.example` | `/etc/systemd/system/soft3-chain@<chain>.service.d/limits.conf` | per-machine MemoryMax drop-in |
| `peers.toml.example` | `/etc/soft3/<chain>.peers.toml` | bootstrap peers, format v1 |
| `nginx-chain-api.conf.template` | `/etc/nginx/snippets/soft3-chain-<chain>.conf` | HTTP API behind TLS with an in-flight cap |
| `ufw-chain.sh` | run by hand | open/close the radio UDP port, logged |
| `soft3-chain-check` | `/usr/local/bin/` | health check: ExecStartPre gate, cron probe, JSON |
| `logrotate-soft3-chain` | `/etc/logrotate.d/soft3-chain` | rotation of the check/ufw logs |
| `journald-soft3-chain.conf` | `/etc/systemd/journald.conf.d/` | journal cap for the node's own logs |
| `soft3-mine@.service` | `/etc/systemd/system/` | the miner beside a node (`soft3 mine` against the local `/template` + `/submit`) |
| `chain.mine.env.example` | `/etc/soft3/<chain>.mine.env` | miner parameters: node URL, coinbase neuron, backend, threads |
| `user/soft3-chain@.service`, `user/soft3-mine@.service` | `~/.config/systemd/user/` (jupiter) | the same two units as systemd **user** units, for a host without sudo (needs `Linger=yes`) |
| `../../scripts/chain-fleet.nu` | dev machine | three nodes on one machine, the rehearsal |
| `../../scripts/rehearsal-genesis.py` | dev machine | a `pussy-rc` genesis from a `soft3 genesis build` file: name, time, bits, funded neurons |

## layout

| | cyberproxy | deimos | jupiter |
|--|--|--|--|
| role | public edge (cyb.ai TLS), third node | best-provisioned node | validator box, third node |
| public IPv4 | 167.235.28.94 | 195.201.105.229 | 88.198.18.156 |
| binary | `/home/cyber/soft3-chain/bin/soft3` → `soft3-<commit>` (never `~/.cargo/bin/soft3`: spacepussy-test runs that file) | same | same |
| state dir | `/var/lib/soft3/<chain>` on `/` (18 GB free — watch it) | `/var/lib/soft3` → symlink to `/mnt/raid4tb/soft3` (root disk is 88 % full) | without sudo: `~/soft3-chain/state` → symlink to `/mnt/nvme4tb/soft3` (user unit); with root: `/var/lib/soft3` → `/mnt/nvme4tb/soft3` |
| HTTP bind | 127.0.0.1:7781 (pussy) · 7782 (bostrom) · 7783 (pussy-rc) | same | same |
| radio UDP | 7791 (pussy) · 7792 (bostrom) · 7793 (pussy-rc) | same | same; inbound UDP stays closed until root runs `ufw-chain.sh` — jupiter is then reachable only over the connections it dials itself (cyberproxy and deimos were; a NAT'd late joiner never was) |
| HTTP public | `https://cyb.ai/<chain>/` | `https://deimos.cybernode.ai/<chain>/` | none (no sudo; local only) |
| MemoryMax | 2G | 16G | 8G (40 GB already in use) |
| sudo | yes | yes | **no** — owner installs the sudoers line first, or runs the root steps by hand; until then the user units of `user/` run the node and the miner |
| CPU miner (measured) | 42.5 kH/s per vCPU | 66.8 kH/s per thread (i7-7700, x86-64 baseline build; 106 kH/s with `-C target-cpu=skylake`) · GTX 1080 webgpu 3.21 MH/s | 66.4 kH/s per thread · GTX 1080 as deimos |

7780 stays with the 0.10.0 chaosnet node (`spacepussy-test.service`); it is untouched.

## step by step

### A. per machine, any day before genesis (nothing starts)

1. Binary: build once on the dev machine and copy — the Mac cross-compiles in
   5 min (`cargo zigbuild --release --locked --bins --no-default-features
   --features webgpu --target x86_64-unknown-linux-gnu.2.31`, glibc 2.31 runs
   on the 20.04 hosts and on 22.04; `PATH=~/.cargo/bin:$PATH` so the rustup
   toolchain with the linux target is used, not Homebrew's rustc). Copy it to
   `/home/cyber/soft3-chain/bin/soft3-<commit>` and `ln -sfn soft3-<commit>
   soft3` there; record its sha256. **Never** `cargo install` or copy onto
   `/home/cyber/.cargo/bin/soft3`: on cyberproxy that file is the 0.10.0
   binary `spacepussy-test.service` runs, and it would pick up the new one at
   its next restart. Confirm `soft3 node --help` lists `--chain --radio --peers`.
2. Directories: `sudo mkdir -p /etc/soft3 /var/log/soft3-chain && sudo chown cyber:adm /var/log/soft3-chain`.
   On deimos/jupiter first `sudo mkdir -p /mnt/<big>/soft3 && sudo chown cyber:cyber /mnt/<big>/soft3 && sudo ln -s /mnt/<big>/soft3 /var/lib/soft3`
   (systemd's `StateDirectory=` accepts the symlinked parent — checked on deimos, systemd 245).
   Then the chain home itself, before step 7 reads its key: `sudo mkdir -p /var/lib/soft3/<chain> && sudo chown cyber:cyber /var/lib/soft3/<chain> && sudo chmod 0750 /var/lib/soft3/<chain>`.
3. Parameters: `sudo cp chain.env.example /etc/soft3/pussy.env`, edit bind/radio/moniker/MemoryMax
   from the table; `sudo chmod 0640 /etc/soft3/pussy.env && sudo chgrp cyber /etc/soft3/pussy.env`.
4. Check script and unit (`soft3-chain-check` before 2026-10-16 probed the wrong `ss` column and reported `DOWN radio` on a bound port — install this version):
   `sudo install -m 0755 soft3-chain-check /usr/local/bin/`,
   `sudo cp soft3-chain@.service /etc/systemd/system/`, the MemoryMax drop-in from `limits.conf.example` (value from the table), `sudo systemctl daemon-reload`.
   Run `soft3-chain-check --pre pussy` — clock, disk and env must be OK.
5. Logs: `sudo cp logrotate-soft3-chain /etc/logrotate.d/soft3-chain`,
   `sudo mkdir -p /etc/systemd/journald.conf.d && sudo cp journald-soft3-chain.conf /etc/systemd/journald.conf.d/ && sudo systemctl restart systemd-journald`.
   The journald drop-in is **machine-wide** (`SystemMaxUse=1G`, `MaxRetentionSec=30day` cut every
   unit's journal, bostrom's and ipfs' included): the owner decides it per host; the rehearsal skipped it.
   `cyber` is not in `adm`/`systemd-journal` on these hosts — read the node's log with `sudo journalctl`.
6. nginx (cyberproxy, deimos): declare the zone once in `http{}`
   (`limit_conn_zone $server_name zone=soft3_chain_pussy:1m;`), render the
   snippet (`sed` line in the template), include it in the 443 server block,
   `sudo nginx -t`. Do **not** reload yet: the upstream is down and the
   location would answer 502 — reload on genesis day.
7. Identity and peers: `soft3 node id --home /var/lib/soft3/pussy` prints the
   endpoint id (64 hex). Collect the three ids, fill `peers.toml.example` →
   `/etc/soft3/pussy.peers.toml` on every machine (each file lists all three;
   a node skips itself).
8. Miner: `sudo cp soft3-mine@.service /etc/systemd/system/`, fill `chain.mine.env.example` →
   `/etc/soft3/<chain>.mine.env` (node URL = `http://` + `SOFT3_BIND`; `soft3 mine` has no
   built-in node URL for `pussy-rc`, so the env always names it), `sudo systemctl daemon-reload`.
   Without at least one miner at genesis, block 1 never comes.
9. Do not `systemctl enable` anything yet.

### B. rehearsal (dev machine, then the testnet)

1. `nu scripts/chain-fleet.nu --check` then `nu scripts/chain-fleet.nu` with
   the release binary: three nodes on one machine must answer `/status` within
   60 s and agree on height; on the dev chain the script goes on to assert a
   late joiner's IBD, a transaction crossing the fleet, a depth-3 fork reorg
   and a kill-and-restart recovery, all on the consensus kernel with
   `soft3 mine --backend cpu` processes doing the work (receipts:
   `audit/chain/net-2026-10-15.md`, `audit/chain/slice-2026-10-16.md`).
2. The dress rehearsal on the real hosts (done 2026-10-16, receipt
   `audit/chain/rehearsal-2026-10-16.md`): steps A1–A8 with `SOFT3_CHAIN=pussy-rc`,
   bind 127.0.0.1:7783, radio 0.0.0.0:7793, no nginx (HTTP stays on loopback;
   reach a node with `ssh -L 17783:127.0.0.1:7783`).
   Genesis: `soft3 genesis build --chain pussy --snapshot ~/cyber/space-pussy/snapshot --out pussy.tade`,
   then `scripts/rehearsal-genesis.py pussy.tade pussy-rc.tade --name pussy-rc --time <T0>
   --bits 0x1e040000 --fund <neuron>=<amount> --drop-links` (2^22 expected hashes = ~30 s at
   ~130 kH/s, two i7-7700 threads; `--drop-links` until § blockers 1 is fixed), the same file
   into every `/var/lib/soft3/pussy-rc/genesis.tade` (0640 cyber), sha256 compared. Start the
   nodes before T0, the first miner at T0. Tear down with
   `sudo systemctl disable --now soft3-mine@pussy-rc soft3-chain@pussy-rc` and
   `sudo ./ufw-chain.sh close pussy-rc` (jupiter: `systemctl --user disable --now …`).

### C. genesis day (2026-11-05)

Order matters only for the peers' first dial; start the best box first.

| T | where | command | verify |
|--|--|--|--|
| T−60 min | all | `sudo ./ufw-chain.sh open pussy` | `ufw status` shows `7791/udp ALLOW IN Anywhere` |
| T−30 min | all | `soft3-chain-check --pre pussy` | all OK; clocks within ms (timesyncd offsets were −2.8 ms / −71 µs / −19 µs on 10-15) |
| T−15 min | all | put the final genesis file at `/var/lib/soft3/pussy/genesis.tade` (a tade stream, not `genesis.json`; `install -m 0640` as cyber) — the node reads `<home>/genesis.tade` when `--genesis` is absent | identical sha256 on the three machines; `/status` `genesis` = the chain id after start |
| T−10 min | deimos | `sudo systemctl enable --now soft3-chain@pussy` (nodes may start before the genesis time; they idle at height 0) | `sudo journalctl -u soft3-chain@pussy -f` shows the endpoint, radio bound, `dialed …`; `soft3-chain-check pussy` |
| T | deimos | `sudo systemctl enable --now soft3-mine@pussy` (at or after the genesis time: block 1's time must exceed it) | the miner logs `template height 1`, then `block 1 accepted`; the other nodes log `net · header … at 1` |
| T+1 min | jupiter | root runs `systemctl enable --now soft3-chain@pussy` (system unit); without root the user unit of `user/` works (`systemctl --user enable --now soft3-chain@pussy`, Linger=yes) — that is how the rehearsal ran | check as above; `/status` must show deimos as a peer |
| T+2 min | cyberproxy | `sudo systemctl enable --now soft3-chain@pussy` | three peers each |
| T+5 min | cyberproxy, deimos | `sudo nginx -t && sudo systemctl restart nginx` (restart, not reload — see servers/CLAUDE.md) | `curl -s https://cyb.ai/pussy/status` |
| T+10 min | dev machine | `cy net add pussy https://cyb.ai/pussy` on a cyb body | the body's networks row shows the height advancing |
| T+1 h | all | `soft3-chain-check pussy` | `progress` OK — height moved |
| T+1 day | — | repeat for `bostrom` on 7782/7792 | |

Rollback at any step: `sudo systemctl disable --now soft3-chain@pussy`,
`sudo ./ufw-chain.sh close pussy`, remove the nginx include and restart
nginx. The state directory is kept for post-mortem.

### D. operating

- `soft3-chain-check pussy` by hand or from cron every 5 min, output to
  `/var/log/soft3-chain/check-pussy.log` (rotated daily × 30).
- Logs: `journalctl -u soft3-chain@pussy` (journal capped at 1 GB).
- Restart: `sudo systemctl restart soft3-chain@pussy` — the unit refuses to
  start when the clock is not NTP-synchronized (`ExecStartPre`). A restarted
  node is back from disk in ~1.5–5 s, but its peers keep the dead connection
  and refuse the new one (`already connected`) for ~12 s, and until radio#42
  is deployed a re-dial can abort the peers (§ blockers 2); `Restart=always`
  is what brings them back.
- Peers file edits need a restart; the file is read at start.
- Thread count is a check: the 0.10.0 chaosnet node reached 1,800 threads on
  2026-10-15 because each request took the state mutex for ~12 s and nginx let
  abandoned clients keep their upstream thread (incident:
  `cybernode/servers/cyberproxy/spacepussy-test.md`). The snippet's
  `limit_conn 32` and the unit's `TasksMax=2048` are the guards.

## blockers (rehearsal 2026-10-16)

1. **A block template on the real pussy genesis takes 17 s (M4 Max) / 26.6 s
   (deimos).** `header_template` → `BbgState::refresh_root` →
   `bbg::dim::commit_fields` → `Brakedown::commit` re-commits every dimension
   per template; the 29,113 genesis links (41,866 particles, 26,805 axons)
   are what costs (the same file without links: 0.18–0.31 s). `soft3 mine`
   gives up on `/template` after 15 s, so no block 1 was ever mined on the
   full-state `pussy-rc` (attempt 1, genesis `e91a3b28…`); the node held its
   lock through each template, `/status` stopped answering within 5 s and
   peers' header syncs timed out. Block validation on the peers pays the same
   root. The rehearsal then ran on the same balances without links.
2. **A peer re-dial aborts the nodes it dials** — quinn-proto
   `CidQueue::next` indexed past its 5-slot ring (`cid_queue.rs:117`, "the len
   is 5 but the index is 5", then `status=6/ABRT`). Restarting cyberproxy took
   down deimos and jupiter in the same millisecond. Fix: cyberia-to/radio#42.
3. `cyber tx …` could not submit to a kernel node: `POST /tx` rejected the
   `application/octet-stream` body chain-wire §7 names (`400 expected hex or
   JSON`). Fix: soft3#196; the rehearsal submitted through a hex shim.

## what the rehearsal changed in this runbook

- binary path `~/soft3-chain/bin/soft3`, never `~/.cargo/bin/soft3` (A1, unit `ExecStart`);
- the chain home is created and chowned before `soft3 node id` (A2);
- `soft3-chain-check` radio probe reads `ss`'s local-address column (was the peer column: `DOWN radio` on every healthy node);
- journald drop-in is machine-wide — owner's call per host (A5);
- a miner unit and its env (A8, C T): the node does not mine;
- genesis is `genesis.tade`, not `genesis.json` (C T−15); nodes start before T0, the miner at T0;
- jupiter runs both as user units until root does the system install; its inbound UDP is closed;
- `pussy-rc` on 7783/7793 is a real path, with `scripts/rehearsal-genesis.py`.

## assumptions taken from the spec in progress

Re-verify every row against `soft3/specs/chain.md` when it merges; each one
is a parameter or a line in one file above.

| assumption | where it lands |
|--|--|
| CLI: `soft3 node --chain <id> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME`; `soft3 node id --home DIR` prints the radio endpoint id — **implemented on the kernel** (soft3#187, slice 2026-10-16): `--home` is the chain directory (`radio.key`, `genesis.tade`, `blocks/` = the kernel's store, `anchors`); `--genesis FILE` installs the genesis file into it (required for `pussy`/`bostrom` until the file is embedded). The node does not mine: miners run `soft3 mine --node http://BIND --address NEURON` against `/template` + `/submit` | unit `ExecStart`, harness probe, step A7 |
| chain ids in the launch set are `pussy`, `bostrom` (and `pussy-rc` for rehearsal) — from `crate/src/node/genesis.rs` on branch `launch/45-genesis-chain-allowlist`; protocol `soft3/<chain>/v1`, engine `cybergraph+bbg` | env `SOFT3_CHAIN`, harness placeholder genesis |
| the radio endpoint id is the node's ed25519 public key, printed as 64 lowercase hex (radio `iroh-base` `PublicKey: Display`) | peers format `id` |
| radio listens on one UDP port given by `--radio`; direct dial between the three machines, no relay — **held** on 2026-10-16: full mesh in seconds, header arrival median 0.4 ms deimos→jupiter, 1.5 ms deimos→cyberproxy; the `[v6]` entries are dialed by a v4-bound socket and never used | ufw rule, peers `relay = ""` |
| HTTP API keeps `GET /health` and `GET /status` with `height` and the bbg root (cyb bodies poll `/status` every 15 s with a 10 s timeout, `cyb/shell/src/worlds/body/networks.rs`), bodies limited to 8 MiB (`specs/native-node.md`) | nginx snippet, check script |
| block cadence: ASERT targets 30 s (chain.md §1.5, half-life 600 s); `SOFT3_STALE_SECS=600` = 20 missed blocks, keep unless launch hashrate says otherwise | env, check `progress` |
| PoW timestamp rule tolerates clock error of seconds; timesyncd (SNTP, ms-level) is enough, chrony is optional | unit `time-sync.target`, check `clock` |
| state is `$home/blocks/` (headers, bodies, undo journals) + `$home/genesis.tade` (a tade file, chain.md §4.1 — not `genesis.json`); a restart replays every main-chain body on the genesis state and verifies the roots (§3.6) — measured locally in seconds for tens of blocks; budget it for the real height before genesis day — on the light rehearsal genesis 46 blocks replayed in < 5 s on deimos; with the real links every replayed block pays § blockers 1 | state dir, `TimeoutStopSec`, step C T−15 |
| the genesis file is produced by `soft3 genesis build` (specs/genesis.md) and copied byte-identical; `scripts/chain-fleet.nu` generates a test genesis (`soft3 chain genesis-dev`: chain `dev`, floor target `POW_LIMIT_BITS`, genesis time two hours back) when none is given — never a launch chain | step C T−15 |
