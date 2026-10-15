#!/usr/bin/env nu
# chain-fleet — three soft3 root-chain nodes on one machine, each with its own home,
# HTTP bind, radio UDP port and bootstrap-peers file; the local rehearsal of
# deploy/chain/README.md before anything touches the three cybernode machines.
# Modeled on cyb/harness/fleet.sh.
#
#   nu scripts/chain-fleet.nu --check                 # validate the config and the binary, start nothing
#   nu scripts/chain-fleet.nu                         # run 3 nodes for 60 s and assert they converge
#   nu scripts/chain-fleet.nu --bin ~/.cargo/bin/soft3 --n 3 --secs 120 --genesis ./genesis.json --keep
#
# Until the phase-1 node exists (soft3/specs/chain.md is being written) this script stops at
# the binary probe with the list of flags it requires — that is the expected result today.

const REQUIRED_FLAGS = ["--chain" "--home" "--bind" "--radio" "--peers" "--moniker"]
const HTTP_BASE = 17780
const RADIO_BASE = 17790

def main [
  --check                      # validate config + binary only
  --bin: string                # node binary; default $SOFT3_BIN, then `soft3` on PATH
  --n: int = 3                 # nodes
  --secs: int = 60             # how long the fleet lives once every node answers
  --chain: string = "pussy"    # chain id written into genesis and peers files
  --genesis: string            # genesis.json copied into every home; default: a placeholder naming the spec
  --keep                       # keep the work dir on success
] {
  let bin = (resolve-bin $bin)
  print $"chain-fleet: ($n) nodes · chain ($chain) · binary ($bin)"

  # ── 1. the binary must exist and speak the phase-1 flags ───────────────────
  if not ($bin | path exists) {
    fail 2 $"no node binary at ($bin). The phase-1 node is not built yet (spec: soft3/specs/chain.md); set --bin or $env.SOFT3_BIN when it exists."
  }
  let help = (do { ^$bin node --help } | complete)
  let help_text = ($help.stdout + $help.stderr)
  let missing = ($REQUIRED_FLAGS | where {|f| not ($help_text | str contains $f) })
  if ($missing | is-not-empty) {
    print $"chain-fleet: `($bin) node --help` lacks ($missing | str join ' ')"
    print "chain-fleet: this binary is not the phase-1 root-chain node. Required: soft3 node --chain <pussy|bostrom> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME"
    print "chain-fleet: see soft3/deploy/chain/README.md § assumptions"
    fail 2 "binary probe failed"
  }
  print "  ok    binary speaks the phase-1 flags"

  # ── 2. ports must be free ──────────────────────────────────────────────────
  let plan = (0..<$n | each {|i|
    {
      i: ($i + 1)
      http: ($HTTP_BASE + $i)
      radio: ($RADIO_BASE + $i)
      moniker: $"fleet-($i + 1)"
    }
  })
  for node in $plan {
    for port in [$node.http $node.radio] {
      let busy = (do { ^lsof -nP -i $":($port)" } | complete | get exit_code) == 0
      if $busy { fail 2 $"port ($port) is in use; free it or change HTTP_BASE/RADIO_BASE" }
    }
  }
  print $"  ok    ports free: http ($plan | get http | str join ',') radio ($plan | get radio | str join ',')"

  # ── 3. homes, genesis, peers ───────────────────────────────────────────────
  let work = (^mktemp -d /tmp/chain-fleet.XXXXXX | str trim)
  let genesis_text = (if ($genesis | is-empty) {
    # placeholder until specs/chain.md fixes the genesis schema; the launch set is pussy | bostrom
    $"{\n  \"chain_id\": \"($chain)\",\n  \"genesis_time\": (date now | format date '%s'),\n  \"engine\": \"cybergraph+bbg\",\n  \"protocol\": \"soft3/($chain)/v1\"\n}\n"
  } else {
    open --raw $genesis
  })
  for node in $plan {
    let home = $"($work)/node-($node.i)"
    mkdir $home
    $genesis_text | save $"($home)/genesis.json"
    let others = ($plan | where i != $node.i)
    let peers = (
      [$"version = 1" $"chain = \"($chain)\"" ""]
      | append ($others | each {|o|
          [ "[[peer]]" $"name = \"($o.moniker)\"" "id = \"\"" $"addrs = [\"127.0.0.1:($o.radio)\"]" "relay = \"\"" "" ]
        } | flatten)
      | str join "\n"
    )
    $peers | save $"($home)/peers.toml"
  }
  print $"  ok    homes + genesis + peers under ($work)"

  if $check {
    print "chain-fleet: --check passed; nothing started"
    rm -rf $work
    return
  }

  # ── 4. launch ──────────────────────────────────────────────────────────────
  mut pids = []
  for node in $plan {
    let home = $"($work)/node-($node.i)"
    let cmd = $"exec ($bin) node --chain ($chain) --home ($home) --bind 127.0.0.1:($node.http) --radio 127.0.0.1:($node.radio) --peers ($home)/peers.toml --moniker ($node.moniker) > ($home)/stdout.log 2>&1 & echo $!"
    let pid = (^bash -c $cmd | str trim | into int)
    $pids = ($pids | append $pid)
  }
  print $"chain-fleet: launched pids ($pids | str join ' ')"

  # ── 5. every node must answer /status within 60 s ─────────────────────────
  let deadline = ((date now) + 60sec)
  mut up = []
  while ((date now) < $deadline) and (($up | length) < $n) {
    $up = ($plan | where {|node| (status-of $node.http) != null } | get i)
    sleep 1sec
  }
  let result = (if (($up | length) == $n) {
    print $"  ok    all ($n) nodes answer /status"
    sleep ($secs * 1sec)
    converge $plan
  } else {
    print $"  FAIL  only ($up | length)/($n) nodes answered within 60 s; logs in ($work)"
    false
  })

  # ── 6. cleanup — nothing we spawned outlives the fleet ────────────────────
  for pid in $pids { do -i { ^kill $pid } }
  sleep 1sec
  for pid in $pids { do -i { ^kill -9 $pid } }
  if $result and (not $keep) { rm -rf $work } else { print $"chain-fleet: artifacts kept in ($work)" }
  if $result { print "chain-fleet: GREEN" } else { fail 1 "chain-fleet: RED" }
}

# the fleet converges when every node reports the same height within a short window
def converge [plan: list<record>] {
  let samples = ($plan | each {|node| {i: $node.i, status: (status-of $node.http)} })
  let heights = ($samples | each {|s| height-of $s.status } | uniq)
  if ($heights | length) == 1 and ($heights | first) != null {
    print $"  ok    heights agree: ($heights | first)"
    true
  } else {
    print $"  FAIL  heights differ or unreadable: ($samples | each {|s| $'node-($s.i)=(height-of $s.status)' } | str join ' ')"
    false
  }
}

def status-of [port: int] {
  let r = (do { ^curl -s -m 3 $"http://127.0.0.1:($port)/status" } | complete)
  if $r.exit_code == 0 and ($r.stdout | is-not-empty) { $r.stdout } else { null }
}

def height-of [status] {
  if $status == null { return null }
  let m = ($status | parse -r '"?height"?[ :=]+(?<h>\d+)')
  if ($m | is-empty) { null } else { $m | first | get h | into int }
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
