# pussy-rc — the phase-1 root chain rehearsed on cyberproxy, deimos and jupiter — 2026-10-16

The vertical slice of `audit/chain/slice-2026-10-16.md` (kernel #189, miner
#186, wire #187), deployed by the runbook `deploy/chain/README.md` to the
three cybernode machines as chain `pussy-rc`: one block from genesis to a
second machine, over the internet, then a transaction, restarts, a fork and
a late joiner. Requirements: `specs/chain.md`, `specs/chain-wire.md`.
Machines: `cybernode/servers/{cyberproxy,deimos,jupiter}/inventory-2026-10-15.md`;
what was installed where: `cybernode/servers/<host>/pussy-rc-2026-10-16.md`.
Genesis of the real chain: 2026-11-05.

## result

| | |
|--|--|
| network | 3 nodes, full mesh, **left running**: cyberproxy (system unit), deimos (system unit + CPU miner), jupiter (user units: node + CPU miner) |
| chain | `pussy-rc`, chain id `77bb25fb11eb4a5ac81d4dc2ddf74713f0563abe379605dd7290e5c11ddd22fd`, genesis block `6d6e6612448aa295f26f3ce1b648dca607bb45a8fae48a86bb64c78882d867f4`, genesis time 1792152780 (12:13:00 UTC) |
| block 1 | mined on deimos 12:13:37.879 UTC; header on jupiter +1.1 ms, on cyberproxy +4.7 ms |
| propagation | @@PROP@@ |
| block times | @@TIMES@@ |
| tx from the Mac | `cyber tx transfer` (true-cyber main) signed on the Mac, submitted to cyberproxy, mined by deimos at height 4, visible with the same block on all three in 17.4 s — through a hex shim, because the node rejected the wallet's raw body (§ broke 3; fixed in soft3#196, verified live without the shim at height 16) |
| restart | cyberproxy back from disk in 1.4 s at its height, synced after 12 s; deimos back in 4.9 s, a 1-block fork at 47 resolved by reorg at 48 |
| late joiner | @@LATE@@ |
| first attempt | **failed**: on the full pussy genesis (29,113 links) a block template takes 26.6 s on deimos, the miner times out at 15 s, no block 1 in 23 min (§ broke 1) |
| crashes | restarting cyberproxy aborted deimos and jupiter in the same millisecond (quinn `CidQueue` index past its ring, § broke 2; fix cyberia-to/radio#42); systemd brought both back in 5 s |

## build

Stand `~/cyber/.stands/RC` (worktrees, detached): soft3 `136721a` (main =
merge of #187), siblings at the slice's working pins — bbg `1d39975`,
cybergraph `0eae7ae`, foculus `9e32aa6`, mudra `4bfe3e8`, zheng `0c882e7`,
joy `06de02c`, lens `e11eaa8`, nox `2f09ca3`, trident `a3cef15f`, hemera
`d5f0a09`, radio `50600cc1`, tade `c7d404a`, file `e29b4c7`, honeycrisp
`9cbd480`, inf `5054c93`, neuron `fd24e76`, strata `494df7a`, tok
`e27da0d`, tru `44725bd` (the slice receipt's "pin deviation": bbg master
lacks the vendored fjall; bbg `1d39975` needs zheng's
`feat/public-certificate-v3` API). By the end of the day soft3 main had
moved to `2d55fdb`, which needs a newer bbg (`BbgState::root_version`); the
deployed binary is `136721a`.

Linux x86_64, cross-compiled on the Mac (M4 Max) with zig:

```
cd crate
PATH=~/.cargo/bin:$PATH cargo zigbuild --release --locked --bins \
  --no-default-features --features webgpu \
  --target x86_64-unknown-linux-gnu.2.31 --target-dir …/target-linux
    Finished `release` profile [optimized] target(s) in 5m 04s
```

| | |
|--|--|
| binary | `soft3` 27,722,792 B, ELF x86-64 PIE, dynamically linked, glibc ≥ 2.31, stripped; `soft3 version` → `soft3 0.10.0` |
| sha256 | `38abc7b2abf422f6c72c1b79abe3388a0834fa178cea1ac2503d5a08d430d46a` (identical on the three hosts) |
| features | node + `mine` with `cpu` and `webgpu` (wgpu → Vulkan); no `metal` |
| installed | `/home/cyber/soft3-chain/bin/soft3-136721a`, symlink `soft3` |
| warnings | 3 from bbg's vendored fjall (`journal_recovery_mode`, `outstanding_flushes`, `write_tx` lifetime) — the ones the slice receipt records; 1 in soft3's bin |

The first attempt failed in 5 s: `cargo` on PATH is Homebrew's (rustc 1.95
without the linux std) — `can't find crate for std`; rustup's toolchain has
the target. Mac builds for the late joiner and the wallet: `soft3`
(`cargo build --release --locked --bins`, 3 m 55 s) and `cyber` (true-cyber
main `580528f`, 2 m 23 s) — true-cyber's committed `Cargo.lock` does not
resolve against soft3 main (`--locked` refuses; 350 packages re-locked), so
`cyber` was built without `--locked`.

### miner throughput on the boxes

`soft3 mine bench --seconds 10`, this binary:

| host | backend | H/s |
|--|--|--|
| deimos (i7-7700) | cpu, 1 thread | 66.8 k |
| deimos | cpu, 4 threads (box busy with bostrom_arch) | 201.6 k |
| deimos | webgpu, GTX 1080 (Vulkan) | 3.21 M |
| jupiter (i7-7700) | cpu, 1 thread | 66.4 k |
| cyberproxy (Skylake vCPU) | cpu, 1 thread | 42.5 k |
| deimos | cpu, 1 thread, same commit built with `RUSTFLAGS=-C target-cpu=skylake` (sha256 `8a941658…2c4c`, not deployed) | 106.2 k (vs 63.0 k baseline in the same minute) |

M4 Max does 275 kH/s per thread (slice receipt); the baseline x86-64 build
leaves 1.7× on these CPUs — worth a tuned miner build for genesis day,
the node can stay portable.

## genesis

Bits: two deimos threads ≈ 130 kH/s; 30 s × 130 kH/s ≈ 3.9 M hashes ≈ 2^22
→ target 2^234 → compact `0x1e040000` (`GENESIS_BITS` 0x1e010000 is 2^24
hashes, 2 min per block on that miner; `POW_LIMIT_BITS` 0x1e100000 is
2^20, 8 s).

```
soft3 genesis build --chain pussy --snapshot ~/cyber/space-pussy/snapshot --out genesis/pussy.tade
genesis pussy · particle 2a83e54fe395ffbd924fc74a54a5c805eeec5396f97bf4c73fced8631a9e61b5
  file       genesis/pussy.tade (3249582 bytes, sha256 461fbfb4db6d8760b4460702275af5c28b06659510df78cd6df72cf179ee2ab7)
  S0         1099389810279114036  subsidy 10458426657
  state_root f2296834490e79d855673b18b0f9eb5c8242e4f48761e85b645d60a8cbeb5827
  block0_id  b863ce2877524e718bb51ea6dc7c5aba9540860f08d70162348d8e84b3eba304
  b_frames 1385 · l_frames 29113 · elapsed 7 s
```

The blessed vectors (slice receipt) — the pipeline is reproducible on this
build. `genesis build` takes only `pussy|bostrom` and writes `GENESIS_BITS`
and the launch name, so the rehearsal file is derived by
`scripts/rehearsal-genesis.py` (added with this receipt): it rewrites the
params frame (name `pussy-rc`, time, bits, subsidy) and inserts the funded
rehearsal neuron in canonical order, every other frame byte for byte. A run
with pussy's own name/time/bits and no `--fund` reproduces `pussy.tade`
exactly (sha256 `461fbfb4…` both). The rehearsal neuron
`3e0dd5b3efc947d4e4bcbd9548aa7b555522796e34e9ba3dc73ea566db4e86d3`
(`pussy1gun28ae3nx0k9u2pfrvlje2rgg9u4q867e39xy`) was minted with
`cyber neuron new --spell ~/.cyber-rc/spell`; its spell exists on the Mac only.

| | attempt 1 (full state) | attempt 2 (running) |
|--|--|--|
| command | `rehearsal-genesis.py pussy.tade pussy-rc.tade --name pussy-rc --time 1792151100 --bits 0x1e040000 --fund 3e0dd5b3…86d3=1000000000000` | same, `--time 1792152780 … --drop-links` |
| file | 3,249,661 B, sha256 `148d5ea5af23c553c7670cf0f911fdb17da8a5b68caae6319543e80f75988efc` | 105,457 B, sha256 `1a9d2bf073080d9c8c95bd3dc7324040d0001328942a9c889734390eb7544b36` |
| chain id | `e91a3b28f08a6dc1d09ea95cf09d730d81e6e38cd096243ee5bf1e852c9fbcd7` | `77bb25fb11eb4a5ac81d4dc2ddf74713f0563abe379605dd7290e5c11ddd22fd` |
| S0 / subsidy | 1,099,390,810,279,114,036 / 10,458,436,170 | same |
| state root (block 0) | `2e076504…193b` | `7c9113fc…e2f2` |
| genesis block (node `/status`) | `d8306bf9…7dd5` | `6d6e6612…67f4` |
| block0_id (`genesis verify --root`) | `df8389bf…7928` | `c205711d…5b51` |
| balances / links | 1,386 / 29,113 | 1,386 / 0 |
| on the hosts | identical sha256 on all three | identical sha256 on all three |

`soft3 genesis verify --root` prints a block-0 id that is not the node's:
`genesis::file::block0_header` hard-codes `GENESIS_BITS`, the kernel's
`Genesis::block0` writes `params.bits`. Equal for a launch file, different
for any rehearsal file.

## install (runbook A1–A8 as pussy-rc; scripts in the cybernode records)

| | cyberproxy | deimos | jupiter |
|--|--|--|--|
| unit | `soft3-chain@pussy-rc` (system) + drop-ins `limits.conf` (2G/1536M), `rehearsal.conf` (ExecStart from `~/soft3-chain/bin`, `--log-net`) | same, 16G/12G | `~/.config/systemd/user/soft3-chain@.service` (Linger=yes) |
| env / peers | `/etc/soft3/pussy-rc.env`, `/etc/soft3/pussy-rc.peers.toml` | same | `~/.config/soft3/pussy-rc.{env,peers.toml}` |
| home | `/var/lib/soft3/pussy-rc` (on `/`) | `/var/lib/soft3` → `/mnt/raid4tb/soft3` | `/mnt/nvme4tb/soft3/pussy-rc` (`~/soft3-chain/state` → `/mnt/nvme4tb/soft3`) |
| HTTP / radio | 127.0.0.1:7783 / 0.0.0.0:7793 | same | same |
| endpoint id | `54471c984e8949e006b3119c93291250930b743b1f200d69d39711af8d99ba64` | `352cf88c6de02f3193b0a6c6c84977ad5daac9989021e26981bfe3076f1ef6d6` | `5d4399810277b44674eccc5c9f33de600fc5e77e78c208dca84b5fcacd6eeb72` |
| ufw | `ufw-chain.sh open pussy-rc` → 7793/udp v4+v6 | 7793/udp v4 | not possible (no sudo) |
| miner | — | `soft3-mine@pussy-rc` (system), cpu 2 threads, Nice 10, coinbase → rehearsal neuron | `soft3-mine@pussy-rc` (user), cpu 1 thread, Nice 19, coinbase → `220ae6f9…14c7` |
| nginx | none: runbook B2 keeps HTTP on loopback for the rehearsal | none | none |
| journald drop-in | skipped (machine-wide retention) | skipped | — |

peers file sha256 `94fd265ce851cb3a89820c9f512e91fdd52935244e1eae4ba7ff9a587aa6d5a9`,
the same on all three. `soft3-chain-check --pre pussy-rc`: clock, disk, env,
peers OK on cyberproxy and deimos (clock offsets −997 µs / −19 µs). The full
check reported `DOWN radio: nothing bound on udp/7793` on both while
`ss` showed `soft3` on `0.0.0.0:7793`: it read `ss`'s peer column (fixed in
this PR). spacepussy-test on cyberproxy stayed `active` on 127.0.0.1:7780
throughout (its binary `~/.cargo/bin/soft3`, Sep 7, untouched).

## attempt 1 — the full pussy state (11:39–11:58 UTC)

Nodes started 11:39:46 (deimos), 11:40:05 (jupiter), 11:40:25 (cyberproxy);
each loaded the genesis in ~10–16 s; full mesh (2 peers each) by 11:40:51.
Miner started at the genesis time 11:45:05 (journal times are CEST, UTC+2):

```
13:45:06 soft3 mine · chain pussy-rc · node http://127.0.0.1:7783 · cpu (2 threads) · to 3e0dd5b3…
13:45:21 GET /template: … timed out reading response; retry in 1s
13:45:37 … retry in 2s · 13:45:54 … 4s · 13:46:13 … 8s · 13:46:36 … 16s · then every 45 s until 13:54:38
```

By hand on deimos: `curl /template` → 200 after **26.6 s**; `curl -m5
/status` → nothing in 5 s; the node at 125 % CPU. Peers: `header sync with
352cf88c: stream timed out` (cyberproxy 13:46:16, 13:47:01; jupiter twice),
then the deimos↔jupiter link flapping (`already connected`, `connection
lost`). Same file on the Mac (M4 Max): `/template` 17.3 s and 17.7 s.
`sample` of the node during a template:

```
KernelChain::template → engine_tx::header_template → BbgState::refresh_root
  → BbgState::root_leaves → bbg::dim::commit_fields → Brakedown::commit
  → Brakedown::build_matrix → cyber_hemera::permutation::permute_profile
```

The same balances without the link frames: `/template` 0.18–0.25 s on the
Mac, 0.31 s on deimos. Stopped at 11:58 (journals saved), homes cleared
except `radio.key`, attempt 2 with `--drop-links`.

## attempt 2 — running

Light genesis installed 12:08, nodes started 12:08:40 (full mesh at 12:08:55,
height 0, same tip and root on all three), deimos miner started at the
genesis time:

```
14:13:04.105 soft3 mine · chain pussy-rc · node http://127.0.0.1:7783 · cpu (2 threads) · to 3e0dd5b3…
14:13:04.419 template height 1 prev 6d6e6612448a… bits 0x1e040000 (2^22 expected hashes)
14:13:37.775 found nonce 0xd4c0271131496024000000000005aa95 after 393216 hashes, 3.0s
14:13:37.879 block 1 accepted: 5595a7aa1bbddccd8b69204c6d16c758feb2040bd20bc6ef2516c1aa40030000
jupiter     14:13:37.880089  net · header 5595a7aa at 1 from 352cf88c
cyberproxy  14:13:37.883722  net · header 5595a7aa at 1 from 352cf88c
```

(miner times CEST.) Jupiter's miner joined at 12:16:18 (1 thread, user unit).

### propagation

Method: each miner's `block N accepted: ID` line against the other nodes'
`net · header ID at N` lines, journal µs timestamps, clocks within ±1 ms
(timesyncd).

@@PROPTABLE@@

The header arrives within the round trip: deimos and jupiter share Hetzner
FSN1, cyberproxy is in NBG. `/status` on the three agreed on tip and bbg
root at every poll except one 1-block lag inside a poll window.

### block times

@@TIMETABLE@@

### a transaction from the Mac

```
ssh -f -N -L 17783:127.0.0.1:7783 -p 8333 cyber@167.235.28.94          # cyberproxy's loopback API
CYBER_SPELL=~/.cyber-rc/spell CYBER_RPC=http://127.0.0.1:17783 cyber tx transfer 220ae6f9…14c7 1000000
cyber: rejected: HTTP 400: expected hex or JSON
```

The node's `POST /tx` parsed hex or JSON only; the wallet posts
`application/octet-stream` as chain-wire §7 says. Through a local shim that
re-encodes the body as hex (`tx-hex-shim.py`, stand tools) the same command:

```
12:14:44.99  cyber tx transfer 220ae6f9e8c3cdc94f2f993deb3f19a5d0ad910cfe50b4a4a00a9e39300a14c7 1000000 --wait 600 --json
12:15:02.38  status included · height 4 · block f54471790b0ce068df04e23eb667ac0356125b76a4c45906708ab9caed000000
             txid 1b43e1b51baf994b329a61fd96dc38e71163ba98ded74dff77d6c9d30b675c14 · fee 224 · nonce 0
```

Submitted to cyberproxy, relayed over radio, mined by deimos' miner; `GET
/tx/1b43e1b5…` answers the same block, height 4, index 1 on all three, and
`/balance/220ae6f9…` is 1,000,000 on all three. With the fix (soft3#196)
built into a second Mac node that joined the network, `cyber tx transfer
220ae6f9… 2000000` straight at it (no shim): included at height 16, txid
`af0a87596652acc9c7c2d59ba07d91d7ff96d27fea9f77e459047501fa4a031b`, 34 s.

### restarts

- cyberproxy, 12:15:26: `systemctl restart` → `/health` in 1.4 s at height
  4, conserved, root `a4bd584f…` = the others'. Its two peers still held the
  old connection and refused the new one (`closed by peer: already
  connected (code 2)`, re-dial every 3 s); at the third re-dial
  (12:15:33.98) **deimos and jupiter aborted** in the same millisecond:

  ```
  thread 'chain-net' panicked at radio/quinn/quinn-proto/src/cid_queue.rs:117:13:
  index out of bounds: the len is 5 but the index is 5
  … PoisonError … panic in a destructor during cleanup … aborting.
  soft3-chain@pussy-rc.service: Main process exited, code=killed, status=6/ABRT
  ```

  systemd restarted both 5 s later; at 12:15:39.8 cyberproxy had both
  inbound, `applied 1 blocks, tip 5`. Same panic killed a Mac node at
  12:20. `CidQueue::next` cleared `buffer[cursor + j]` without `% LEN`;
  cyberia-to/radio#42 adds the modulo and a test that reproduces the field
  panic line for line.
- deimos, 12:32:55 (the miner's node): `/health` in 4.9 s at height 46
  (replayed from disk, conserved). Its peers refused it for ~12 s as above
  (no abort this time); its own miner mined a 47 (`9bd5cf90…`) while jupiter's
  47 (`43657fe0…`) went to the others — a 1-block fork; jupiter's 48
  (`dfc85006…`) on top of its 47 made deimos reorg; tips equal again by
  12:34:01.

### late joiner

@@LATETEXT@@

### resources (12:31 UTC, height ~45)

| | node RSS | node CPU | threads | home |
|--|--|--|--|--|
| cyberproxy | 25.8 MB (cgroup 11 MB) | 18 % | 6 | 268 KB |
| deimos | cgroup 23 MB | — | 15 | 268 KB |
| jupiter | 35.3 MB | 24 % | 15 | 264 KB |

An idle node at 18–24 % of a core with a 105 KB genesis is worth a look
(dial retry loop to the unreachable / sync tick); the miners hold 2 and 1
cores.

## what broke

1. **Template cost on the real genesis (launch blocker).** 26.6 s per
   `/template` on deimos, 17 s on an M4 Max, for the pussy genesis with its
   29,113 links: `refresh_root` re-commits every bbg dimension per template.
   `soft3 mine`'s 15 s HTTP timeout never sees a template; the node's lock
   starves `/status` and the wire (header syncs time out, links flap). Every
   block a peer validates recomputes the same root. Nothing in the slice saw
   it: the fleet's dev genesis has two balances. 30 s blocks on the real
   genesis need the root incremental (or cached per dimension) before
   2026-11-05; the rehearsal runs without the links.
2. **Remote abort on re-dial (launch blocker).** quinn-proto `CidQueue::next`
   out of bounds → abort. One restarting peer took down the other two.
   Fix radio#42 (code, not merged here); soft3 needs the radio pin bumped.
3. **`POST /tx` refused the wallet's bytes.** Fix soft3#196 (code, not
   merged here). true-cyber's tests pass against a mock node, so nothing
   caught it.
4. Reconnect after a restart takes ~12 s: peers keep the dead QUIC
   connection until its idle timeout and refuse the new one as `already
   connected`; a restarted node mines on its own tip meanwhile (the fork at 47).
5. `soft3-chain-check` radio probe read the wrong `ss` column (fixed here).
6. The runbook would have overwritten spacepussy-test's binary on
   cyberproxy (A1, unit `ExecStart`) and named the genesis `genesis.json`
   (C T−15); no miner unit; journald drop-in is machine-wide (all fixed
   or flagged in `deploy/chain/README.md`).
7. `soft3 genesis verify --root` prints a block-0 id computed with
   `GENESIS_BITS`, not the file's bits; `soft3 mine --chain pussy-rc` has no
   default node URL; `/balance` prints `denom: pussy-rc` for the `pussy`
   token; true-cyber's lockfile does not resolve against soft3 main. Small,
   unfixed.
8. jupiter: no inbound UDP (ufw needs root) — reachable only through the
   connections it dials; the NAT'd late joiners never reached it.

## what needs the owner

jupiter (no sudo for `cyber`), in order:

1. `echo 'cyber ALL=(ALL) NOPASSWD: ALL' | sudo tee /etc/sudoers.d/cyber && sudo chmod 440 /etc/sudoers.d/cyber`
   (servers/CLAUDE.md) — or run 2–5 as root by hand;
2. `sudo ./ufw-chain.sh open pussy-rc` (and `pussy` on genesis day) — after `/etc/soft3/pussy-rc.env` exists, or `sudo ufw allow proto udp to any port 7793 comment soft3-chain-pussy-rc-radio`;
3. move to the system unit like deimos: `/etc/soft3/*`, `/usr/local/bin/soft3-chain-check`,
   `/etc/systemd/system/soft3-chain@.service` + drop-ins (MemoryMax 8G),
   `ln -s /mnt/nvme4tb/soft3 /var/lib/soft3`; then `systemctl --user disable --now soft3-mine@pussy-rc soft3-chain@pussy-rc`
   and `sudo systemctl enable --now soft3-chain@pussy-rc soft3-mine@pussy-rc` — the state
   directory (`/mnt/nvme4tb/soft3/pussy-rc`) is the same, nothing to copy;
4. logrotate and (if wanted) the journald drop-in;
5. the reboot the inventory asks for (1,036 days up), before genesis.

All hosts: decide the journald drop-in per host. Merge radio#42 and
soft3#196 (code) and the bbg pin convergence before the release that
genesis runs; fix § broke 1 before genesis.

## running now and how to stop it

| host | units | stop |
|--|--|--|
| cyberproxy | `soft3-chain@pussy-rc` (enabled) | `sudo systemctl disable --now soft3-chain@pussy-rc && cd ~/soft3-chain && sudo ./ufw-chain.sh close pussy-rc` |
| deimos | `soft3-chain@pussy-rc`, `soft3-mine@pussy-rc` (enabled) | `sudo systemctl disable --now soft3-mine@pussy-rc soft3-chain@pussy-rc && cd ~/soft3-chain && sudo ./ufw-chain.sh close pussy-rc` |
| jupiter | user `soft3-chain@pussy-rc`, `soft3-mine@pussy-rc` (enabled, Linger) | `systemctl --user disable --now soft3-mine@pussy-rc soft3-chain@pussy-rc` |

Status from the Mac: `ssh -p 8333 cyber@<host> curl -s 127.0.0.1:7783/status`.
The Mac late-joiner nodes were stopped (own pids); the rehearsal spell
`~/.cyber-rc/spell` and the jupiter miner seed `~/.cyber-rc/jupiter-miner.seed`
stay on the Mac only.
