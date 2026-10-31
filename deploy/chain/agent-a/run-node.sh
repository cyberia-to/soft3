#!/usr/bin/env bash
# run-node.sh <chain> — supervisor for soft3 node (and optionally soft3 mine) on agent-a, no systemd.
# Mirrors soft3-chain@.service: pre-check gate, restart after 5 s, burst limit 5 starts / 300 s,
# SIGTERM forwarded to the child. Normally started by start.sh inside a tmux session.
#
# Limits: `ulimit -n 65536` (best effort). The unit's TasksMax=2048 has no reliable equivalent in a
# container (ulimit -u counts per uid across the whole container), so the thread count is watched
# by soft3-chain-check (DEGRADED above 512), not enforced.
set -euo pipefail

chain="${1:-}"
[[ -n "$chain" ]] || { echo "usage: run-node.sh <chain>" >&2; exit 2; }
root="${CHAIN_ROOT:-/srv/forge/claude/chain}"
env_file="$root/etc/$chain.env"
[[ -r "$env_file" ]] || { echo "run-node: missing $env_file" >&2; exit 2; }
# shellcheck disable=SC1090
source "$env_file"
: "${SOFT3_CHAIN:?}" "${SOFT3_BIND:?}" "${SOFT3_RADIO:?}" "${SOFT3_MONIKER:?}"
pidfile="${SOFT3_PIDFILE:-$root/run/$chain.pid}"
mine_pidfile="${SOFT3_MINE_PIDFILE:-$root/run/$chain.mine.pid}"
mkdir -p "$root/run" "$root/log" "$root/state/$chain"
export RUST_LOG="${RUST_LOG:-info}"

ulimit -n 65536 2>/dev/null || echo "run-node: could not raise nofile to 65536 (now $(ulimit -n))" >&2

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
check="${SOFT3_CHECK:-$(command -v soft3-chain-check || true)}"
[[ -n "$check" ]] || check="$here/../soft3-chain-check"
bin="$root/bin/soft3"
[[ -x "$bin" ]] || { echo "run-node: $bin is not executable" >&2; exit 2; }

log() { printf '%s run-node[%s]: %s\n' "$(date -u +%FT%TZ)" "$chain" "$*"; }

# pre-check gate: clock, disk, env — refuse to start otherwise
if ! SOFT3_ETC="$root/etc" SOFT3_STATE="$root/state" bash "$check" --pre "$chain"; then
  log "soft3-chain-check --pre failed; not starting"
  exit 1
fi

child=""; mchild=""; msup=""; stopping=0
on_term() {
  stopping=1
  log "SIGTERM: stopping children"
  [[ -n "$child" ]] && kill -TERM "$child" 2>/dev/null || true
  [[ -n "$mchild" ]] && kill -TERM "$mchild" 2>/dev/null || true   # the miner, in its own supervise subshell
  [[ -n "$msup" ]] && kill -TERM "$msup" 2>/dev/null || true
}
trap on_term TERM INT

node_cmd=("$bin" node --chain "$SOFT3_CHAIN" --home "$root/state/$chain" --bind "$SOFT3_BIND"
          --radio "$SOFT3_RADIO" --peers "$root/etc/$chain.peers.toml" --moniker "$SOFT3_MONIKER")
[[ -n "${SOFT3_LOG_NET:-}" && "${SOFT3_LOG_NET}" != 0 ]] && node_cmd+=(--log-net)

mine_cmd=()
if [[ -n "${SOFT3_MINE_ADDRESS:-}" ]]; then
  mine_cmd=("$bin" mine --node "http://$SOFT3_BIND" --chain "$SOFT3_CHAIN" --address "$SOFT3_MINE_ADDRESS" --backend cpu)
  [[ -n "${SOFT3_MINE_THREADS:-}" ]] && mine_cmd+=(--threads "$SOFT3_MINE_THREADS")
fi

# supervise <name> <pidfile> <logfile> cmd... — restart loop with the systemd burst limit
supervise() {
  local name="$1" pf="$2" lf="$3"; shift 3
  local -a starts=()
  trap on_term TERM INT   # a backgrounded call (the miner) starts with default traps
  local now t kept
  while (( ! stopping )); do
    now=$(date +%s)
    kept=()
    for t in "${starts[@]}"; do (( now - t < 300 )) && kept+=("$t"); done
    starts=("${kept[@]}" "$now")
    if (( ${#starts[@]} > 5 )); then
      log "$name: more than 5 starts in 300 s — giving up (like StartLimitBurst); fix and rerun start.sh"
      return 1
    fi
    log "$name: starting: $*"
    "$@" >> "$lf" 2>&1 &
    local pid=$!
    echo "$pid" > "$pf"
    [[ "$name" == node ]] && child=$pid || mchild=$pid
    local rc=0
    wait "$pid" || rc=$?
    # a trapped signal interrupts the first wait; wait again until the child is really gone
    while kill -0 "$pid" 2>/dev/null; do wait "$pid" || rc=$?; done
    rm -f "$pf"
    (( stopping )) && { log "$name: stopped (rc=$rc)"; return 0; }
    log "$name: exited rc=$rc; restart in 5 s"
    sleep 5 &
    wait $! || true
  done
}

if (( ${#mine_cmd[@]} )); then
  supervise miner "$mine_pidfile" "$root/log/$chain.mine.log" "${mine_cmd[@]}" &
  msup=$!
fi
rc=0
supervise node "$pidfile" "$root/log/$chain.log" "${node_cmd[@]}" || rc=$?
stopping=1
[[ -n "$msup" ]] && { kill -TERM "$msup" 2>/dev/null || true; wait "$msup" 2>/dev/null || true; }
exit "$rc"
