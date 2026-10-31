#!/usr/bin/env bash
# stop.sh <chain> — SIGTERM the supervisor's node via its pidfile, wait up to 60 s (TimeoutStopSec), then kill the tmux session.
set -euo pipefail
chain="${1:-}"
[[ -n "$chain" ]] || { echo "usage: stop.sh <chain>" >&2; exit 2; }
root="${CHAIN_ROOT:-/srv/forge/claude/chain}"
session="soft3-$chain"
# the supervisor's pid is the tmux pane's process; the node and miner pidfiles hold the children
pane_pid=$(tmux list-panes -t "$session" -F '#{pane_pid}' 2>/dev/null | head -1 || true)
pids=()
for pf in "$root/run/$chain.pid" "$root/run/$chain.mine.pid"; do
  [[ -r "$pf" ]] && pids+=("$(cat "$pf")")
done
if [[ ${#pids[@]} -eq 0 && -z "$pane_pid" ]]; then
  echo "stop: nothing running for $chain"; exit 0
fi
# the supervisor first: it forwards SIGTERM to its children and exits
sup=$(pgrep -f "run-node.sh $chain\$" | head -1 || true)
[[ -n "$sup" ]] && kill -TERM "$sup" 2>/dev/null || true
for p in "${pids[@]}"; do kill -TERM "$p" 2>/dev/null || true; done
for _ in $(seq 60); do
  alive=0
  for p in "${pids[@]}"; do kill -0 "$p" 2>/dev/null && alive=1; done
  [[ -n "$sup" ]] && kill -0 "$sup" 2>/dev/null && alive=1
  (( alive )) || break
  sleep 1
done
if (( alive )); then
  echo "stop: still alive after 60 s — SIGKILL" >&2
  for p in "${pids[@]}" ${sup:+"$sup"}; do kill -KILL "$p" 2>/dev/null || true; done
fi
tmux kill-session -t "$session" 2>/dev/null || true
echo "stopped $chain"
