# deploy/chain — the phase-1 root chain on two hosts

Runbook for running `soft3 node` as the proof-of-work root chain (decision
2026-10-15, cyber/launch.md; genesis 2026-11-05) on two hosts, cyberproxy and
agent-a (owner decision 2026-10-31: no new hardware; deimos and jupiter are
retired, see § E). Node-to-node over cyber-radio (QUIC/UDP), the HTTP API
behind the existing nginx on cyberproxy. A third peer may be any cyb body
(no file needed). Prepared 2026-10-15, reshaped 2026-10-31 while the chain
spec (`soft3/specs/chain.md`) is still being written; § assumptions lists
what this runbook takes from the spec and must be re-checked when it lands.
Nothing here starts a service by itself.

Honest risk: two nodes on two hosts, and only one of them (cyberproxy) is
public. It is the single point for new peers: while it is down nobody new can
join, and agent-a only reconnects.

Machine facts behind cyberproxy: `cybernode/servers/cyberproxy/inventory-2026-10-15.md`.

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
| `agent-a/run-node.sh` | `/srv/forge/claude/chain/bin/` (or run in place) | agent-a supervisor: pre-check, pidfile, restart after 5 s, burst limit 5/300 s, optional CPU miner |
| `agent-a/start.sh`, `agent-a/stop.sh` | run by hand on agent-a | detached tmux session `soft3-<chain>`; stop = SIGTERM via pidfile, waits 60 s |
| `agent-a/agent-a.env.example` | `/srv/forge/claude/chain/etc/<chain>.env` | agent-a parameters, incl. `SOFT3_PIDFILE`, `SOFT3_MINE_ADDRESS`, `SOFT3_MINE_THREADS` |
| `agent-a/agent-a.peers.toml.example` | `/srv/forge/claude/chain/etc/<chain>.peers.toml` | agent-a's peers: cyberproxy only, dial-out |
| `../../scripts/chain-fleet.nu` | dev machine | three nodes on one machine, the local rehearsal |

## layout

| | cyberproxy | agent-a |
|--|--|--|
| role | public edge (cyb.ai TLS), the only dialable node | dial-out peer + CPU miner; no public HTTP, no inbound UDP assumed |
| host | systemd system unit, sudo | container (hostname agent-0), 12 cores, 47 GB RAM, no systemd (PID 1 is sshd), no sudo, user `claude`, tmux + python3, no `ss` |
| public IPv4 | 167.235.28.94 · v6 `2a01:4f8:c0c:7c49::1` | none inbound; egress 66.188.16.11 behind NAT |
| state dir | `/var/lib/soft3/<chain>` on `/` (18 GB free — watch it) | `/srv/forge/claude/chain/state/<chain>` (~470 GB free on /srv/forge) |
| home layout | `/etc/soft3`, `/var/log/soft3-chain` | `/srv/forge/claude/chain/{bin/soft3,etc,state,log,run}` |
| HTTP bind | 127.0.0.1:7781 (pussy) · 7782 (bostrom) · 7783 (pussy-rc) | same, loopback only |
| radio UDP | 7791 (pussy) · 7792 (bostrom) · 7793 (pussy-rc) | same ports, listening but dialing out |
| HTTP public | `https://cyb.ai/<chain>/` | none |
| supervisor | `soft3-chain@<chain>.service` | `agent-a/start.sh` → tmux `soft3-<chain>` → `run-node.sh` |
| limits | MemoryMax 2G (drop-in) | `ulimit -n 65536` in the runner; no MemoryMax, thread count only watched by the check |
| env overrides for the check | none (defaults) | `SOFT3_ETC=/srv/forge/claude/chain/etc SOFT3_STATE=/srv/forge/claude/chain/state` (run-node.sh sets them) |
| clock check | `timedatectl` | kernel `adjtimex` via python3 (state 5 = unsynchronized = DOWN) |
| miner | none | `soft3 mine --backend cpu` when `SOFT3_MINE_ADDRESS` is set |

7780 stays with the 0.10.0 chaosnet node (`spacepussy-test.service`); it is untouched.

## step by step

### A. per machine, any day before genesis (nothing starts)

cyberproxy as below. agent-a: see the variant after step 8.

1. Binary: `cargo install soft3 --version <phase-1 release> --locked` as `cyber`
   (or copy the release binary to `/home/cyber/.cargo/bin/soft3`). Confirm
   `soft3 node --help` lists `--chain --radio --peers`; the unit will not work
   with the 0.10.0 flag set.
2. Directories: `sudo mkdir -p /etc/soft3 /var/log/soft3-chain && sudo chown cyber:adm /var/log/soft3-chain`.
3. Parameters: `sudo cp chain.env.example /etc/soft3/pussy.env`, edit bind/radio/moniker/MemoryMax
   from the table; `sudo chmod 0640 /etc/soft3/pussy.env && sudo chgrp cyber /etc/soft3/pussy.env`.
4. Check script and unit: `sudo install -m 0755 soft3-chain-check /usr/local/bin/`,
   `sudo cp soft3-chain@.service /etc/systemd/system/`, the MemoryMax drop-in from `limits.conf.example` (value from the table), `sudo systemctl daemon-reload`.
   Run `soft3-chain-check --pre pussy` — clock, disk and env must be OK.
5. Logs: `sudo cp logrotate-soft3-chain /etc/logrotate.d/soft3-chain`,
   `sudo mkdir -p /etc/systemd/journald.conf.d && sudo cp journald-soft3-chain.conf /etc/systemd/journald.conf.d/ && sudo systemctl restart systemd-journald`.
6. nginx (cyberproxy only): declare the zone once in `http{}`
   (`limit_conn_zone $server_name zone=soft3_chain_pussy:1m;`), render the
   snippet (`sed` line in the template), include it in the 443 server block,
   `sudo nginx -t`. Do **not** reload yet: the upstream is down and the
   location would answer 502 — reload on genesis day.
7. Identity and peers: `soft3 node id --home /var/lib/soft3/pussy` prints the
   endpoint id (64 hex). Cyberproxy's file is `peers.toml.example` →
   `/etc/soft3/pussy.peers.toml` (cyberproxy listed, it skips itself; agent-a
   is never listed — no inbound address, an id without addrs is a parse
   error). Agent-a's file lists cyberproxy only, with the id printed on cyberproxy.
8. Do not `systemctl enable` anything yet.

**agent-a variant** (user `claude`, no sudo, no cargo install):

1. Binary: the release executable from the build service — the candidate
   workflow's artifact of a green rehearsal on `x86_64-unknown-linux-gnu`.
   `mkdir -p /srv/forge/claude/chain/{bin,etc,state,log,run}`, copy it to
   `/srv/forge/claude/chain/bin/soft3`, `chmod 0755`, record
   `sha256sum bin/soft3` next to the artifact id (a line in
   `/srv/forge/claude/chain/bin/soft3.sha256`). Confirm
   `bin/soft3 node --help` lists `--chain --radio --peers`.
2. Scripts: copy `soft3-chain-check` to `/srv/forge/claude/chain/bin/` and
   `agent-a/*.sh` next to it (`SOFT3_CHECK` overrides where run-node.sh looks
   for the check; it falls back to `PATH`, then to `../soft3-chain-check`).
3. Parameters: `agent-a.env.example` → `etc/<chain>.env`, `agent-a.peers.toml.example`
   → `etc/<chain>.peers.toml` with cyberproxy's id filled in.
4. `SOFT3_ETC=/srv/forge/claude/chain/etc SOFT3_STATE=/srv/forge/claude/chain/state soft3-chain-check --pre <chain>`:
   clock (adjtimex), disk (>= 20 GiB), env and peers must be OK.
5. Miner (optional): set `SOFT3_MINE_ADDRESS` (neuron, 64 hex) and
   `SOFT3_MINE_THREADS` in the env file; empty address = no miner.
6. Nothing starts until `start.sh`.

### B. rehearsal (dev machine, then the testnet)

1. `nu scripts/chain-fleet.nu --check` then `nu scripts/chain-fleet.nu` with
   the release binary: three nodes on one machine must answer `/status` within
   60 s and agree on height; on the dev chain the script goes on to assert a
   late joiner's IBD, a transaction crossing the fleet, a depth-3 fork reorg
   and a kill-and-restart recovery, all on the consensus kernel with
   `soft3 mine --backend cpu` processes doing the work (receipts:
   `audit/chain/net-2026-10-15.md`, `audit/chain/slice-2026-10-16.md`).
2. Dress rehearsal on the real two hosts with `SOFT3_CHAIN=pussy-rc` (the
   launch allow-list has it), ports 7783/7793:
   - cyberproxy: system unit `soft3-chain@pussy-rc` (env file with 7783/7793,
     moniker `cyberproxy-pussy-rc`), `sudo ./ufw-chain.sh open pussy-rc`,
     `sudo systemctl start soft3-chain@pussy-rc`.
   - agent-a: `agent-a/start.sh pussy-rc` (peers file → cyberproxy:7793).
   - verify: `/status` on both shows the other as a peer and heights agree;
     `soft3-chain-check pussy-rc` OK on both; with a miner address set on
     agent-a, height advances. Then kill cyberproxy's node: agent-a must keep
     retrying and reconnect after the restart.
   - tear down: `sudo systemctl disable --now soft3-chain@pussy-rc` +
     `sudo ./ufw-chain.sh close pussy-rc`; `agent-a/stop.sh pussy-rc`.

### C. genesis day (2026-11-05)

Order: cyberproxy first — it is the only dialable node, agent-a dials it.

| T | where | command | verify |
|--|--|--|--|
| T−60 min | cyberproxy | `sudo ./ufw-chain.sh open pussy` | `ufw status` shows `7791/udp ALLOW IN Anywhere` |
| T−30 min | both | `soft3-chain-check --pre pussy` (agent-a with the `SOFT3_ETC`/`SOFT3_STATE` overrides) | all OK; clocks synchronized |
| T−15 min | both | put the final `genesis.json` in the state dir (`/var/lib/soft3/pussy/`, `/srv/forge/claude/chain/state/pussy/`) | identical sha256 on both |
| T | cyberproxy | `sudo systemctl enable --now soft3-chain@pussy` | `journalctl -u soft3-chain@pussy -f` shows listening + radio bound; `soft3-chain-check pussy` |
| T+1 min | agent-a | `agent-a/start.sh pussy` | `tail -f log/pussy.log`; cyberproxy `/status` shows agent-a as a peer |
| T+5 min | cyberproxy | `sudo nginx -t && sudo systemctl restart nginx` (restart, not reload — see servers/CLAUDE.md) | `curl -s https://cyb.ai/pussy/status` |
| T+10 min | dev machine | `cy net add pussy https://cyb.ai/pussy` on a cyb body | the body's networks row shows the height advancing |
| T+1 h | both | `soft3-chain-check pussy` | `progress` OK — height moved |
| T+1 day | — | repeat for `bostrom` on 7782/7792 | |

nginx runs on cyberproxy only; agent-a publishes no HTTP.

Rollback at any step: cyberproxy `sudo systemctl disable --now soft3-chain@pussy`,
`sudo ./ufw-chain.sh close pussy`, remove the nginx include and restart
nginx; agent-a `agent-a/stop.sh pussy`. The state directories are kept for post-mortem.

### D. operating

- `soft3-chain-check pussy` by hand or from cron every 5 min, output to
  `/var/log/soft3-chain/check-pussy.log` (rotated daily × 30).
- Logs: `journalctl -u soft3-chain@pussy` (journal capped at 1 GB); agent-a: `/srv/forge/claude/chain/log/<chain>.log`, `<chain>.mine.log`, `<chain>.supervisor.log` (no rotation yet — truncate by hand or `logrotate` from the owner).
- agent-a: `tmux attach -t soft3-<chain>`; `start.sh`/`stop.sh <chain>`; the supervisor gives up after 5 starts in 300 s and says so in the supervisor log.
- Restart: `sudo systemctl restart soft3-chain@pussy` — the unit refuses to
  start when the clock is not NTP-synchronized (`ExecStartPre`).
- Peers file edits need a restart; the file is read at start.
- Thread count is a check: the 0.10.0 chaosnet node reached 1,800 threads on
  2026-10-15 because each request took the state mutex for ~12 s and nginx let
  abandoned clients keep their upstream thread (incident:
  `cybernode/servers/cyberproxy/spacepussy-test.md`). The snippet's
  `limit_conn 32` and the unit's `TasksMax=2048` are the guards; agent-a has no equivalent, `soft3-chain-check` reports DEGRADED above 512 threads.

### E. retiring deimos and jupiter

The owner's hand — the director has no SSH to either host.

- deimos: `sudo systemctl disable --now soft3-chain@pussy-rc soft3-mine@pussy-rc`
  (and the same for `pussy` if it was ever enabled), `sudo ./ufw-chain.sh close pussy-rc`
  (and `pussy`), remove the nginx include for `deimos.cybernode.ai/<chain>`, `sudo nginx -t && sudo systemctl restart nginx`.
- jupiter (user units): `systemctl --user disable --now soft3-chain@pussy-rc soft3-mine@pussy-rc`.
- keep journals and state on both for post-mortem until the hosts are cancelled.
- then remove deimos and jupiter from every peers file (cyberproxy's too) and
  restart the nodes — the file is read at start only.

## assumptions taken from the spec in progress

Re-verify every row against `soft3/specs/chain.md` when it merges; each one
is a parameter or a line in one file above.

| assumption | where it lands |
|--|--|
| CLI: `soft3 node --chain <id> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME`; `soft3 node id --home DIR` prints the radio endpoint id — **implemented on the kernel** (soft3#187, slice 2026-10-16): `--home` is the chain directory (`radio.key`, `genesis.tade`, `blocks/` = the kernel's store, `anchors`); `--genesis FILE` installs the genesis file into it (required for `pussy`/`bostrom` until the file is embedded). The node does not mine: miners run `soft3 mine --node http://BIND --address NEURON` against `/template` + `/submit` | unit `ExecStart`, harness probe, step A7 |
| chain ids in the launch set are `pussy`, `bostrom` (and `pussy-rc` for rehearsal) — from `crate/src/node/genesis.rs` on branch `launch/45-genesis-chain-allowlist`; protocol `soft3/<chain>/v1`, engine `cybergraph+bbg` | env `SOFT3_CHAIN`, harness placeholder genesis |
| the radio endpoint id is the node's ed25519 public key, printed as 64 lowercase hex (radio `iroh-base` `PublicKey: Display`) | peers format `id` |
| radio listens on one UDP port given by `--radio`; direct dial (agent-a → cyberproxy), no relay | ufw rule, peers `relay = ""` |
| HTTP API keeps `GET /health` and `GET /status` with `height` and the bbg root (cyb bodies poll `/status` every 15 s with a 10 s timeout, `cyb/shell/src/worlds/body/networks.rs`), bodies limited to 8 MiB (`specs/native-node.md`) | nginx snippet, check script |
| block cadence: ASERT targets 30 s (chain.md §1.5, half-life 600 s); `SOFT3_STALE_SECS=600` = 20 missed blocks, keep unless launch hashrate says otherwise | env, check `progress` |
| PoW timestamp rule tolerates clock error of seconds; timesyncd (SNTP, ms-level) is enough, chrony is optional | unit `time-sync.target`, check `clock` |
| state is `$home/blocks/` (headers, bodies, undo journals) + `$home/genesis.tade` (a tade file, chain.md §4.1 — not `genesis.json`); a restart replays every main-chain body on the genesis state and verifies the roots (§3.6) — measured locally in seconds for tens of blocks; budget it for the real height before genesis day | state dir, `TimeoutStopSec`, step C T−15 |
| the genesis file is produced by `soft3 genesis build` (specs/genesis.md) and copied byte-identical; `scripts/chain-fleet.nu` generates a test genesis (`soft3 chain genesis-dev`: chain `dev`, floor target `POW_LIMIT_BITS`, genesis time two hours back) when none is given — never a launch chain | step C T−15 |
| agent-a sits behind NAT; it dials cyberproxy, inbound to it is not assumed; a two-node stand has one dialable node, so cyberproxy restarting leaves agent-a reconnecting every `DIAL_INTERVAL` (3 s, `discovery.rs`) | peers files (agent-a is never listed), step C order, § layout |
| `parse_peers`: an entry with an empty `id` is skipped, one with an id but no parsable `addrs` is a hard error; a node skips itself; `relay` is ignored by the code | `peers.toml.example`, `agent-a.peers.toml.example` |
