#!/usr/bin/env bash
# start.sh <chain> — start run-node.sh in a detached tmux session soft3-<chain>.
set -euo pipefail
chain="${1:-}"
[[ -n "$chain" ]] || { echo "usage: start.sh <chain>" >&2; exit 2; }
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="${CHAIN_ROOT:-/srv/forge/claude/chain}"
session="soft3-$chain"
if tmux has-session -t "$session" 2>/dev/null; then
  echo "start: tmux session $session already exists (tmux attach -t $session)" >&2
  exit 1
fi
mkdir -p "$root/log" "$root/run"
tmux new-session -d -s "$session" "CHAIN_ROOT='$root' bash '$here/run-node.sh' '$chain' 2>&1 | tee -a '$root/log/$chain.supervisor.log'"
echo "started tmux session $session; log $root/log/$chain.log"
