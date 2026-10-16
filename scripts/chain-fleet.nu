#!/usr/bin/env nu
# chain-fleet — three soft3 root-chain nodes on one machine, each with its own home,
# HTTP bind, radio UDP port and bootstrap-peers file, on the consensus kernel, mined by
# `soft3 mine --backend cpu` processes against their /template + /submit; the local
# rehearsal of deploy/chain/README.md before anything touches the three cybernode
# machines. Modeled on cyb/harness/fleet.sh.
#
#   nu scripts/chain-fleet.nu --check                 # validate the config and the binary, start nothing
#   nu scripts/chain-fleet.nu                         # run the fleet and assert the chain-wire scenarios
#   nu scripts/chain-fleet.nu --bin target/release/soft3 --n 3 --secs 20 --threads 1 --keep
#   nu scripts/chain-fleet.nu --chain pussy --genesis ./genesis.tade   # a real genesis file (no dev switches)
#
# Scenarios asserted (specs/chain-wire.md): peers connect · headers/blocks gossip and the
# heights converge · a late-joining node IBDs to the tip · a tx submitted to node 1 is mined
# by node 2 and visible on node 3 · a fork of depth 3 resolves to the heavier chain · a
# killed node restarts and recovers. The dev genesis sits at the floor target (2^-20 per hash)
# with genesis time two hours back, so ASERT stays at the floor for the whole run. Only the
# processes this script spawned are killed, by pid (recorded in $work/pids).

const REQUIRED_FLAGS = ["--chain" "--home" "--bind" "--radio" "--peers" "--moniker"]
const HTTP_BASE = 17780
const RADIO_BASE = 17790
# the harness wallet: a fixed seed, funded in the dev genesis; never a launch-chain key
const WALLET_SEED = "1111111111111111111111111111111111111111111111111111111111111111"
const BOB = "7777777777777777777777777777777777777777777777777777777777777777"

def main [
  --check                      # validate config + binary only
  --bin: string                # node binary; default $SOFT3_BIN, then `soft3` on PATH
  --n: int = 3                 # nodes in the fleet (a fourth joins late)
  --secs: int = 20             # how long the fleet mines before the convergence check
  --threads: int = 1           # CPU threads per `soft3 mine` process (1 ≈ a block per 4 s at the floor target)
  --chain: string = "dev"      # chain name; `dev` gets a generated genesis and the /dev switches
  --genesis: string            # genesis.tade copied into every home; default: generated dev genesis
  --keep                       # keep the work dir on success
] {
  let bin = (resolve-bin $bin)
  print $"chain-fleet: ($n) nodes · chain ($chain) · binary ($bin)"

  # ── 1. the binary must exist and speak the phase-1 flags ───────────────────
  if not ($bin | path exists) {
    fail 2 $"no node binary at ($bin). Build soft3 (cargo build in crate/) or set --bin / $env.SOFT3_BIN."
  }
  let help = (do { ^$bin node --help } | complete)
  let help_text = ($help.stdout + $help.stderr)
  let missing = ($REQUIRED_FLAGS | where {|f| not ($help_text | str contains $f) })
  if ($missing | is-not-empty) {
    print $"chain-fleet: `($bin) node --help` lacks ($missing | str join ' ')"
    print "chain-fleet: this binary is not the phase-1 root-chain node. Required: soft3 node --chain <pussy|bostrom> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME"
    fail 2 "binary probe failed"
  }
  print "  ok    binary speaks the phase-1 flags"

  # ── 2. ports must be free ──────────────────────────────────────────────────
  let plan = (0..$n | each {|i|
    { i: ($i + 1), http: ($HTTP_BASE + $i), radio: ($RADIO_BASE + $i), moniker: $"fleet-($i + 1)" }
  })
  for node in $plan {
    for port in [$node.http $node.radio] {
      let busy = (do { ^lsof -nP -i $":($port)" } | complete | get exit_code) == 0
      if $busy { fail 2 $"port ($port) is in use; free it or change HTTP_BASE/RADIO_BASE" }
    }
  }
  print $"  ok    ports free: http ($plan | get http | str join ',') radio ($plan | get radio | str join ',')"

  # ── 3. homes, genesis, identities, peers ───────────────────────────────────
  let work = (^mktemp -d /tmp/chain-fleet.XXXXXX | str trim)
  let wallet = (^$bin chain key --seed $WALLET_SEED | str trim)
  let genesis_file = $"($work)/genesis.tade"
  if ($genesis | is-empty) {
    if $chain != "dev" { fail 2 $"--chain ($chain) needs --genesis <file>; only `dev` gets a generated genesis" }
    ^$bin chain genesis-dev --name $chain --fund $"($wallet)=1000000000000" --out $genesis_file | ignore
  } else {
    cp $genesis $genesis_file
  }
  # every node's endpoint id first (the key is created on first call), then the peers files
  let plan = ($plan | each {|node|
    let home = $"($work)/node-($node.i)"
    mkdir $home
    $node | insert home $home | insert id (^$bin node id --home $home | str trim)
  })
  for node in $plan {
    let others = ($plan | where i != $node.i)
    peers-file $chain $others | save $"($node.home)/peers.toml"
  }
  print $"  ok    homes + genesis + peers under ($work); wallet ($wallet | str substring 0..8)…"

  if $check {
    print "chain-fleet: --check passed; nothing started"
    rm -rf $work
    return
  }

  # ── 4. launch: every node; `soft3 mine` works node 1 and node 2 (each its own coinbase address)
  let plan = ($plan | each {|node| $node | insert miner ($"($node.i)" | fill -c ($node.i | into string) -w 64) | insert work $work })
  "" | save -f $"($work)/pids"
  let pids = ($plan | first $n | each {|node| launch $bin $chain $genesis_file $node })
  print $"chain-fleet: launched pids ($pids | str join ' ')"

  let result = (do {
    # ── 5. every node answers /status within 60 s ──────────────────────────
    let fleet = ($plan | first $n)
    if not (wait-for 60 {|| ($fleet | all {|node| (status-of $node.http) != null }) }) {
      print $"  FAIL  not every node answered /status within 60 s; logs in ($work)"
      return false
    }
    print $"  ok    all ($n) nodes answer /status"
    let m1 = (miner-start $bin $chain ($fleet | get 0) $threads)
    let m2 = (miner-start $bin $chain ($fleet | get 1) $threads)
    print $"chain-fleet: soft3 mine --backend cpu --threads ($threads) on node-1 \(pid ($m1)\) and node-2 \(pid ($m2)\)"

    # peers connect
    if not (wait-for 30 {|| ($fleet | all {|node| (peer-count $node.http) >= 1 }) }) {
      print $"  FAIL  peers did not connect: ($fleet | each {|node| $'node-($node.i)=(peer-count $node.http)' } | str join ' ')"
      return false
    }
    print $"  ok    peers connected: ($fleet | each {|node| $'node-($node.i)=(peer-count $node.http)' } | str join ' ')"

    # headers/blocks gossip: the fleet mines for --secs then agrees on height
    sleep ($secs * 1sec)
    if not (wait-for 30 {|| (converged $fleet) }) {
      print $"  FAIL  heights differ: ($fleet | each {|node| $'node-($node.i)=(height-of (status-of $node.http))' } | str join ' ')"
      return false
    }
    let h = (height-of (status-of ($fleet | first | get http)))
    if $h < 3 { print $"  FAIL  only ($h) blocks mined in ($secs) s"; return false }
    let b1 = (^curl -s -m 3 $"http://127.0.0.1:(($fleet | get 1).http)/block/1" | from json)
    let ids = ($fleet | each {|node| ^curl -s -m 3 $"http://127.0.0.1:($node.http)/block/1" | from json | get id } | uniq)
    if ($ids | length) != 1 { print $"  FAIL  block 1 differs across nodes: ($ids | str join ' ')"; return false }
    print $"  ok    block 1 ($b1.id | str substring 0..12)… mined by ($b1.miner | str substring 0..8)… is block 1 on every node"
    print $"  ok    heights agree at ($h) across ($n) nodes"

    if $chain != "dev" {
      print "chain-fleet: launch chain — the dev scenarios (late join, tx, fork, restart) need /dev switches; stopping here"
      return true
    }

    # ── 6. a late-joining node IBDs to the tip ─────────────────────────────
    let late = ($plan | get ($n))
    let late_pid = (launch $bin $chain $genesis_file $late)
    print $"chain-fleet: late joiner node-($late.i) pid ($late_pid)"
    let first = ($fleet | first)
    if not (wait-for 60 {||
      let s = (status-of $late.http)
      ($s != null) and (not (catching-up $s)) and ((height-of $s) >= ((height-of (status-of $first.http)) - 1)) and ((height-of $s) > 0)
    }) {
      print $"  FAIL  late joiner did not reach the tip: ((status-of $late.http) | default '' | lines | where $it =~ '^(height|best-header|catching|peers)' | str join ' ')"
      return false
    }
    print $"  ok    late joiner synced to height (height-of (status-of $late.http)) \(IBD by ranges\)"
    let everyone = ($fleet | append $late)

    # ── 7. a tx submitted to node 1 is mined by node 2, visible on node 3 ───
    miner-stop $m1
    sleep 2sec
    let nonce = (^curl -s -m 3 $"http://127.0.0.1:($first.http)/nonce/($wallet)" | from json | get nonce)
    let tx = (^$bin chain tx --seed $WALLET_SEED --genesis $genesis_file --nonce $nonce --to $BOB --amount 5 | str trim)
    let txid = (^curl -s -m 3 -X POST -H "Content-Type: application/octet-stream" --data-binary $"($tx)" $"http://127.0.0.1:($first.http)/tx" | from json | get txid)
    let third = ($fleet | get 2)
    if not (wait-for 60 {|| (tx-height $third.http $txid) != null }) {
      print $"  FAIL  tx ($txid) not seen mined on node-3: (^curl -s -m 3 $'http://127.0.0.1:($third.http)/tx/($txid)')"
      return false
    }
    let mined_at = (tx-height $third.http $txid)
    let miner = (^curl -s -m 3 $"http://127.0.0.1:($third.http)/block/($mined_at)" | from json | get miner)
    let second = ($fleet | get 1)
    if $miner != $second.miner { print $"  FAIL  block ($mined_at) paid ($miner), not node-2's miner ($second.miner)"; return false }
    let bob = (^curl -s -m 3 $"http://127.0.0.1:($third.http)/balance/($BOB)" | parse -r 'balance: (?<b>\d+)' | first | get b | into int)
    if $bob != 5 { print $"  FAIL  bob holds ($bob) on node-3, expected 5"; return false }
    print $"  ok    tx ($txid | str substring 0..8)… submitted to node-1, mined at ($mined_at) by node-2's miner ($miner | str substring 0..8)…, bob=5 on node-3"

    # ── 8. a fork of depth 3 resolves to the heavier chain ─────────────────
    miner-stop $m2
    sleep 2sec
    if not (wait-for 30 {|| (converged $everyone) }) {
      print $"  FAIL  fleet did not settle before the fork: ($everyone | each {|node| $'node-($node.i)=(height-of (status-of $node.http))' } | str join ' ')"
      return false
    }
    let base = (height-of (status-of $first.http))
    isolate $late.http true
    let ml = (miner-start $bin $chain $late $threads)
    if not (wait-for 180 {|| (height-of (status-of $late.http)) >= ($base + 5) }) { print "  FAIL  isolated node did not mine its fork"; return false }
    miner-stop $ml
    let mm = (miner-start $bin $chain $first $threads)
    if not (wait-for 180 {|| (height-of (status-of $first.http)) >= ($base + 3) }) { print "  FAIL  main chain did not extend"; return false }
    miner-stop $mm
    sleep 2sec
    let main_h = (height-of (status-of $first.http))
    let fork_h = (height-of (status-of $late.http))
    let fork_tip = (tip-of $late.http)
    let main_work = (work-of $first.http)
    let fork_work = (work-of $late.http)
    if $fork_work <= $main_work { print $"  FAIL  fork \(($fork_h), work ($fork_work)\) is not heavier than main \(($main_h), work ($main_work)\)"; return false }
    isolate $late.http false
    let dial = ({ id: $first.id, addrs: [$"127.0.0.1:($first.radio)"] } | to json -r)
    ^curl -s -m 10 -X POST -d $dial $"http://127.0.0.1:($late.http)/dev/dial" | ignore
    if not (wait-for 120 {|| ($fleet | all {|node| (tip-of $node.http) == $fork_tip }) }) {
      print $"  FAIL  fleet did not reorg to the heavier fork: ($everyone | each {|node| $'node-($node.i)=(height-of (status-of $node.http))' } | str join ' ')"
      return false
    }
    print $"  ok    fork of depth ($main_h - $base) from ($base): main ($main_h) reorged to the heavier chain ($fork_h), tip ($fork_tip | str substring 0..8)… on every node"

    # ── 9. a killed node restarts and recovers ─────────────────────────────
    let victim = $third
    let before = (height-of (status-of $victim.http))
    ^kill -9 ($pids | get 2)
    sleep 1sec
    let mr = (miner-start $bin $chain $first $threads)
    let new_pid = (launch $bin $chain $genesis_file $victim)
    print $"chain-fleet: node-3 killed and relaunched as pid ($new_pid)"
    if not (wait-for 120 {||
      let s = (status-of $victim.http)
      ($s != null) and ((height-of $s) > $before) and ((height-of $s) >= ((height-of (status-of $first.http)) - 1))
    }) {
      print $"  FAIL  node-3 did not recover: before ($before), now ((status-of $victim.http) | default '' | parse -r 'height: (?<h>\d+)')"
      return false
    }
    miner-stop $mr
    let after = (height-of (status-of $victim.http))
    let conserved = ((status-of $victim.http) | str contains "conserved: true")
    if not $conserved { print "  FAIL  conservation check failed after restart"; return false }
    print $"  ok    node-3 recovered from disk: height ($before) before the kill, ($after) after restart \(replayed, then synced\), conserved"
    true
  })

  # ── cleanup — every pid this script spawned (nodes, the late joiner, the relaunch, miners), by pid only
  let spawned = (open $"($work)/pids" | lines | where $it != "" | into int)
  for pid in $spawned { do -i { ^kill $pid } | complete | ignore }
  sleep 1sec
  for pid in $spawned { do -i { ^kill -9 $pid } | complete | ignore }
  if $result and (not $keep) { rm -rf $work } else { print $"chain-fleet: artifacts kept in ($work)" }
  if $result { print "chain-fleet: GREEN" } else { fail 1 "chain-fleet: RED" }
}

def launch [bin: string, chain: string, genesis: string, node: record] {
  let cmd = $"exec ($bin) node --chain ($chain) --home ($node.home) --bind 127.0.0.1:($node.http) --radio 127.0.0.1:($node.radio) --peers ($node.home)/peers.toml --genesis ($genesis) --moniker ($node.moniker) --log-net >> ($node.home)/stdout.log 2>&1 & echo $!"
  let pid = (^bash -c $cmd | str trim | into int)
  $"($pid)\n" | save -a $"($node.work)/pids"
  $pid
}

# `soft3 mine --backend cpu` against one node's /template + /submit, paying the node's miner address
def miner-start [bin: string, chain: string, node: record, threads: int] {
  let cmd = $"exec ($bin) mine --chain ($chain) --node http://127.0.0.1:($node.http) --address ($node.miner) --backend cpu --threads ($threads) >> ($node.home)/mine.log 2>&1 & echo $!"
  let pid = (^bash -c $cmd | str trim | into int)
  $"($pid)\n" | save -a $"($node.work)/pids"
  $pid
}

def miner-stop [pid: int] {
  do -i { ^kill $pid }
}

# chainwork of a node's tip (decimal)
def work-of [port: int] {
  ^curl -s -m 3 $"http://127.0.0.1:($port)/tip" | from json | get chainwork | into int
}

def peers-file [chain: string, others: list<record>] {
  [$"version = 1" $"chain = \"($chain)\"" ""]
  | append ($others | each {|o|
      [ "[[peer]]" $"name = \"($o.moniker)\"" $"id = \"($o.id)\"" $"addrs = [\"127.0.0.1:($o.radio)\"]" "relay = \"\"" "" ]
    } | flatten)
  | str join "\n"
}

# poll `cond` every half second for up to `secs`
def wait-for [secs: int, cond: closure] {
  let deadline = ((date now) + ($secs * 1sec))
  mut ok = false
  while (not $ok) and ((date now) < $deadline) {
    $ok = (do $cond)
    if not $ok { sleep 500ms }
  }
  $ok
}

# the fleet converges when every node reports the same height
def converged [nodes: list<record>] {
  let heights = ($nodes | each {|node| height-of (status-of $node.http) } | uniq)
  (($heights | length) == 1) and (($heights | first) != null)
}

def status-of [port: int] {
  let r = (do { ^curl -s -m 3 $"http://127.0.0.1:($port)/status" } | complete)
  if $r.exit_code == 0 and ($r.stdout | is-not-empty) { $r.stdout } else { null }
}

def height-of [status] {
  if $status == null { return null }
  let m = ($status | parse -r '(?m)^"?height"?[ :=]+(?<h>\d+)')
  if ($m | is-empty) { null } else { $m | first | get h | into int }
}

def catching-up [status] {
  ($status | str contains "catching-up: true")
}

def tip-of [port: int] {
  let r = (do { ^curl -s -m 3 $"http://127.0.0.1:($port)/tip" } | complete)
  if $r.exit_code == 0 and ($r.stdout | is-not-empty) { $r.stdout | from json | get id } else { null }
}

def peer-count [port: int] {
  let r = (do { ^curl -s -m 3 $"http://127.0.0.1:($port)/peers" } | complete)
  if $r.exit_code == 0 and ($r.stdout | is-not-empty) { $r.stdout | from json | get peers | length } else { 0 }
}

def tx-height [port: int, txid: string] {
  let r = (do { ^curl -s -m 3 $"http://127.0.0.1:($port)/tx/($txid)" } | complete)
  if $r.exit_code != 0 or ($r.stdout | is-empty) { return null }
  let v = ($r.stdout | from json)
  let h = ($v | get -o height)
  if ($h == null) or ($h == "mempool") { null } else { $h }
}

def isolate [port: int, on: bool] {
  ^curl -s -m 3 -X POST -d ({ on: $on } | to json -r) $"http://127.0.0.1:($port)/dev/isolate" | ignore
}

def resolve-bin [bin] {
  if ($bin | is-not-empty) { return ($bin | path expand) }
  if ("SOFT3_BIN" in $env) { return ($env.SOFT3_BIN | path expand) }
  let found = (which soft3)
  if ($found | is-empty) { "soft3" } else { $found | first | get path }
}

def fail [code: int, msg: string] {
  print $"chain-fleet: ($msg)"
  exit $code
}
