#!/bin/bash
# the director's pulse: tick, then sessions back to back while there is work; sleep when idle or out of window; sessions run only when the tick changed or the heartbeat is due
HEARTBEAT=${HEARTBEAT:-10800}
# mergeable flaps UNKNOWN on every push to a PR's branches; a real conflict is seen at the latest by the heartbeat
TICK_CMP='del(.at) | .prs |= map(del(.mergeable))'
# skip the session when the newest tick equals the previous one, the last session said NEXT: idle, and it started less than HEARTBEAT ago (uses t, next, last_start)
should_skip() { local prev; prev=$(ls -1 state/tick-*.json 2>/dev/null | tail -2 | head -1)
  [ -n "$prev" ] && [ "$prev" != "$t" ] && [ "$next" = "NEXT: idle" ] && [ $(( $(date +%s) - ${last_start:-0} )) -lt "$HEARTBEAT" ] && diff -q <(jq -S "$TICK_CMP" "$prev") <(jq -S "$TICK_CMP" "$t") >/dev/null; }
[ "${BASH_SOURCE[0]}" = "$0" ] || return 0
export PATH="$HOME/director:$HOME/.local/bin:$PATH" DIR="$HOME/director"
# the body's subscription token (claude setup-token, 1 year); never in the repository
[ -f "$HOME/.config/cyber/claude.env" ] && . "$HOME/.config/cyber/claude.env"
cd "$DIR"; mkdir -p state
while true; do
  bash tick.sh || echo "$(date -u +%H:%M:%S) tick failed rc=$?" >> state/tick.log
  t=$(ls -1 state/tick-*.json 2>/dev/null | tail -1); r=$(ls -t state/red-*.json 2>/dev/null | head -1)
  if should_skip; then echo "$(date -u +%H:%M:%S) no session: tick unchanged since an idle session (heartbeat ${HEARTBEAT}s)" >> state/tick.log; sleep 900; continue; fi
  last_start=$(date +%s)
  log=state/session-$(date -u +%Y%m%dT%H%M%SZ).log
  echo "$(date -u +%H:%M:%S) session start" >> state/tick.log
  goose run --recipe director.yaml --no-session --params tick="$t" --params red="${r:-}" > "$log" 2>&1
  rc=$?; next=$(grep -o 'NEXT: [a-z]*' "$log" | tail -1)
  echo "$(date -u +%H:%M:%S) session end rc=$rc ${next:-NEXT: ?}" >> state/tick.log
  if grep -qiE "usage limit|rate limit|limit reached|resets at" "$log"; then echo "$(date -u +%H:%M:%S) subscription window hit; sleeping 30 min" >> state/tick.log; sleep 1800
  elif [ "$next" = "NEXT: more" ]; then sleep 30
  else sleep 900; fi
  find state -name 'session-*.log' -mtime +14 -delete
done
