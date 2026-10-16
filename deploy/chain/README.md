# deploy/chain — the phase-1 root chain on three cybernode machines

Runbook for running `soft3 node` as the proof-of-work root chain (decision
2026-10-15, cyber/launch.md; genesis 2026-11-05) on cyberproxy, deimos and
jupiter, node-to-node over cyber-radio (QUIC/UDP) and the HTTP API behind
the existing nginx. Prepared 2026-10-15 while the chain spec
(`soft3/specs/chain.md`) is still being written; § assumptions lists what
this runbook takes from the spec and must be re-checked when it lands.
Nothing here starts a service by itself.

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
| `../../scripts/chain-fleet.nu` | dev machine | three nodes on one machine, the rehearsal |

## layout

| | cyberproxy | deimos | jupiter |
|--|--|--|--|
| role | public edge (cyb.ai TLS), third node | best-provisioned node | validator box, third node |
| public IPv4 | 167.235.28.94 | 195.201.105.229 | 88.198.18.156 |
| state dir | `/var/lib/soft3/<chain>` on `/` (18 GB free — watch it) | `/var/lib/soft3` → symlink to `/mnt/raid4tb/soft3` (root disk is 88 % full) | `/var/lib/soft3` → symlink to `/mnt/nvme4tb/soft3` |
| HTTP bind | 127.0.0.1:7781 (pussy) · 7782 (bostrom) | same | same |
| radio UDP | 7791 (pussy) · 7792 (bostrom) | same | same |
| HTTP public | `https://cyb.ai/<chain>/` | `https://deimos.cybernode.ai/<chain>/` | none (no sudo; local only) |
| MemoryMax | 2G | 16G | 8G (40 GB already in use) |
| sudo | yes | yes | **no** — owner installs the sudoers line first, or runs the root steps by hand |

7780 stays with the 0.10.0 chaosnet node (`spacepussy-test.service`); it is untouched.

## step by step

### A. per machine, any day before genesis (nothing starts)

1. Binary: `cargo install soft3 --version <phase-1 release> --locked` as `cyber`
   (or copy the release binary to `/home/cyber/.cargo/bin/soft3`). Confirm
   `soft3 node --help` lists `--chain --radio --peers`; the unit will not work
   with the 0.10.0 flag set.
2. Directories: `sudo mkdir -p /etc/soft3 /var/log/soft3-chain && sudo chown cyber:adm /var/log/soft3-chain`.
   On deimos/jupiter first `sudo mkdir -p /mnt/<big>/soft3 && sudo chown cyber:cyber /mnt/<big>/soft3 && sudo ln -s /mnt/<big>/soft3 /var/lib/soft3`.
3. Parameters: `sudo cp chain.env.example /etc/soft3/pussy.env`, edit bind/radio/moniker/MemoryMax
   from the table; `sudo chmod 0640 /etc/soft3/pussy.env && sudo chgrp cyber /etc/soft3/pussy.env`.
4. Check script and unit: `sudo install -m 0755 soft3-chain-check /usr/local/bin/`,
   `sudo cp soft3-chain@.service /etc/systemd/system/`, the MemoryMax drop-in from `limits.conf.example` (value from the table), `sudo systemctl daemon-reload`.
   Run `soft3-chain-check --pre pussy` — clock, disk and env must be OK.
5. Logs: `sudo cp logrotate-soft3-chain /etc/logrotate.d/soft3-chain`,
   `sudo mkdir -p /etc/systemd/journald.conf.d && sudo cp journald-soft3-chain.conf /etc/systemd/journald.conf.d/ && sudo systemctl restart systemd-journald`.
6. nginx (cyberproxy, deimos): declare the zone once in `http{}`
   (`limit_conn_zone $server_name zone=soft3_chain_pussy:1m;`), render the
   snippet (`sed` line in the template), include it in the 443 server block,
   `sudo nginx -t`. Do **not** reload yet: the upstream is down and the
   location would answer 502 — reload on genesis day.
7. Identity and peers: `soft3 node id --home /var/lib/soft3/pussy` prints the
   endpoint id (64 hex). Collect the three ids, fill `peers.toml.example` →
   `/etc/soft3/pussy.peers.toml` on every machine (each file lists all three;
   a node skips itself).
8. Do not `systemctl enable` anything yet.

### B. rehearsal (dev machine, then the testnet)

1. `nu scripts/chain-fleet.nu --check` then `nu scripts/chain-fleet.nu` with
   the release binary: three nodes on one machine must answer `/status` within
   60 s and agree on height; on the dev chain the script goes on to assert a
   late joiner's IBD, a transaction crossing the fleet, a depth-3 fork reorg
   and a kill-and-restart recovery (receipt: `audit/chain/net-2026-10-15.md`).
2. Same three steps on the machines with `SOFT3_CHAIN=pussy-rc` (the launch
   allow-list has it) on ports 7783/7793 if a dress rehearsal on real hosts is
   wanted; tear down with `systemctl disable --now soft3-chain@pussy-rc`
   and `ufw-chain.sh close pussy-rc`.

### C. genesis day (2026-11-05)

Order matters only for the peers' first dial; start the best box first.

| T | where | command | verify |
|--|--|--|--|
| T−60 min | all | `sudo ./ufw-chain.sh open pussy` | `ufw status` shows `7791/udp ALLOW IN Anywhere` |
| T−30 min | all | `soft3-chain-check --pre pussy` | all OK; clocks within ms (timesyncd offsets were −2.8 ms / −71 µs / −19 µs on 10-15) |
| T−15 min | all | put the final `genesis.json` in `/var/lib/soft3/pussy/` | identical sha256 on the three machines |
| T | deimos | `sudo systemctl enable --now soft3-chain@pussy` | `journalctl -u soft3-chain@pussy -f` shows listening + radio bound; `soft3-chain-check pussy` |
| T+1 min | jupiter | `systemctl --user` is not enough for a system unit — root runs `systemctl enable --now soft3-chain@pussy` | check as above; `/status` must show deimos as a peer |
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
  start when the clock is not NTP-synchronized (`ExecStartPre`).
- Peers file edits need a restart; the file is read at start.
- Thread count is a check: the 0.10.0 chaosnet node reached 1,800 threads on
  2026-10-15 because each request took the state mutex for ~12 s and nginx let
  abandoned clients keep their upstream thread (incident:
  `cybernode/servers/cyberproxy/spacepussy-test.md`). The snippet's
  `limit_conn 32` and the unit's `TasksMax=2048` are the guards.

## assumptions taken from the spec in progress

Re-verify every row against `soft3/specs/chain.md` when it merges; each one
is a parameter or a line in one file above.

| assumption | where it lands |
|--|--|
| CLI: `soft3 node --chain <id> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME`; `soft3 node id --home DIR` prints the radio endpoint id — **implemented** (soft3#187): `--home` is the chain directory (`radio.key`, `genesis.tade`, `blocks.tade`, `anchors`); `--genesis FILE` installs the genesis file into it (required for `pussy`/`bostrom` until the file is embedded) | unit `ExecStart`, harness probe, step A7 |
| chain ids in the launch set are `pussy`, `bostrom` (and `pussy-rc` for rehearsal) — from `crate/src/node/genesis.rs` on branch `launch/45-genesis-chain-allowlist`; protocol `soft3/<chain>/v1`, engine `cybergraph+bbg` | env `SOFT3_CHAIN`, harness placeholder genesis |
| the radio endpoint id is the node's ed25519 public key, printed as 64 lowercase hex (radio `iroh-base` `PublicKey: Display`) | peers format `id` |
| radio listens on one UDP port given by `--radio`; direct dial between the three machines, no relay | ufw rule, peers `relay = ""` |
| HTTP API keeps `GET /health` and `GET /status` with `height` and the bbg root (cyb bodies poll `/status` every 15 s with a 10 s timeout, `cyb/shell/src/worlds/body/networks.rs`), bodies limited to 8 MiB (`specs/native-node.md`) | nginx snippet, check script |
| block cadence: unknown — `SOFT3_STALE_SECS=600` is a placeholder until the PoW target interval is fixed | env, check `progress` |
| PoW timestamp rule tolerates clock error of seconds; timesyncd (SNTP, ms-level) is enough, chrony is optional | unit `time-sync.target`, check `clock` |
| state is `$home/bbg` + `$home/blocks.tade` + `$home/genesis.tade` (a tade file, chain.md §4.1 — not `genesis.json`); a restart replays from disk (budget the replay time — the 0.10.0 node replays 41k signals in hours, see the incident) | state dir, `TimeoutStopSec`, step C T−15 |
| the genesis file is produced by `soft3 genesis build` (specs/genesis.md) and copied byte-identical; `scripts/chain-fleet.nu` generates a dev genesis (`soft3 chain genesis-dev`) when none is given | step C T−15 |
