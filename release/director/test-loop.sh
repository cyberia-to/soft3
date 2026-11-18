#!/bin/bash
# the cases of loop.sh's should_skip — run: bash release/director/test-loop.sh
# sources the real loop.sh (the guard after should_skip stops the loop itself from running) and feeds it
# fake ticks in a temporary directory. no network, no goose, no tokens.
SRC=$(cd "$(dirname "$0")" && pwd)
. "$SRC/loop.sh"

WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
cd "$WORK" || exit 1; mkdir -p state
fail=0

# tick <stamp> <owner_activity> <head sha> [mergeable] [updatedAt]
tick() { jq -n --arg at "$1" --arg o "$2" --arg h "$3" --arg m "${4:-MERGEABLE}" --arg u "${5:-2026-10-27T11:00:00Z}" \
    '{at:$at, heads:{}, prs:[{n:384,t:"the idle skip",labels:[],mergeable:$m,branch:"fix/director-idle-skip",head:$h,updated:$u}], last_run:null, owner_activity:$o}' \
    > "state/tick-$1.json"; }
reset() { rm -f state/tick-*.json; t=; next="NEXT: idle"; last_start=$(date +%s); HEARTBEAT=10800; }
newest() { t=$(ls -1 state/tick-*.json 2>/dev/null | tail -1); }   # what loop.sh sets after each tick
want() { local name=$1 expect=$2 got; if should_skip; then got=skip; else got=run; fi
  if [ "$got" = "$expect" ]; then echo "ok   $name → $got"; else echo "FAIL $name: expected $expect, got $got"; fail=1; fi; }

A=2026-10-26T07:35:34Z; SHA=1111111111111111111111111111111111111111

# (a) two ticks identical but for at, the last session idle, started just now
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest
want "identical ticks, NEXT: idle, inside the heartbeat" skip

# (b) the owner commented, reviewed or pushed anywhere in the org
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z 2026-10-27T11:10:00Z $SHA; newest
want "owner_activity moved" run

# (c) the last session had more to do
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest; next="NEXT: more"
want "NEXT: more" run

# (d) the heartbeat expired since the last session started
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest; last_start=$(( $(date +%s) - 10801 ))
want "last session older than the heartbeat" run

# (e) the first iteration of the process: one tick, no session yet
reset; tick 20261027T110000Z $A $SHA; newest; unset next last_start
want "first iteration" run
next="NEXT: idle"; last_start=$(date +%s)

# (f) the session printed no NEXT line (logged as NEXT: ?)
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest; next=""
want "no NEXT line" run

# (g) a push to an open PR moves its head
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A 2222222222222222222222222222222222222222; newest
want "an open PR's head moved" run

# (h) a comment or a review moves the PR's updatedAt
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA MERGEABLE 2026-10-27T11:14:00Z; newest
want "an open PR's updatedAt moved" run

# (i) mergeable alone flaps and is deliberately ignored
reset; tick 20261027T110000Z $A $SHA UNKNOWN; tick 20261027T111500Z $A $SHA MERGEABLE; newest
want "mergeable alone flapped" skip

# (j) a truncated or malformed tick must never buy silence — one side unreadable
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest
head -c 20 "$t" > "$t.cut" && mv "$t.cut" "$t"
want "the newest tick is truncated" run 2>/dev/null

# (k) and neither when both are unreadable: two failing renderings are not two equal ticks
reset; tick 20261027T110000Z $A $SHA; tick 20261027T111500Z $A $SHA; newest
for f in state/tick-*.json; do head -c 20 "$f" > "$f.cut" && mv "$f.cut" "$f"; done
want "both ticks are truncated" run 2>/dev/null

# (l) the comparison above is blind unless tick.sh records the fields it compares
for f in head:.headRefOid updated:.updatedAt; do
  if grep -q -- "$f" "$SRC/tick.sh"; then echo "ok   tick.sh records $f"; else echo "FAIL tick.sh does not record $f"; fail=1; fi
done

[ "$fail" = 0 ] && echo "all cases pass" || echo "cases failed"
exit "$fail"
