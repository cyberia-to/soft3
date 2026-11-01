#!/bin/bash
# no-network test of tick.sh's pull_receipts and loop.sh's pick_red; run: bash release/director/test_receipts.sh
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
mkdir -p "$W/bin"
cat > "$W/bin/gh" <<'STUB'
#!/bin/bash
echo "$*" >> "$GH_CALLS"
case "$1 $2" in
  "run download") while [ $# -gt 0 ]; do [ "$1" = -D ] && D=$2; shift; done
    for l in $GH_LEGS; do mkdir -p "$D/candidate-$l/receipt"; echo "{\"result\":\"green\",\"gates\":[]}" > "$D/candidate-$l/receipt/release-validation.json"; done
    for l in ${GH_NORECEIPT:-}; do mkdir -p "$D/candidate-$l/receipt/logs"; done
    exit "${GH_DL_RC:-0}";;
  api*) echo "$GH_WANT";;
esac
STUB
chmod +x "$W/bin/gh"; export PATH="$W/bin:$PATH" GH_CALLS="$W/calls"
export DIR=$W/dir; mkdir -p "$DIR"
. "$here/tick.sh"
fail=0; ok() { if eval "$2"; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi; }
fresh() { rm -rf "$STATE"/run-* "$STATE"/red-*; : > "$GH_CALLS"; export GH_LEGS="a b c d e" GH_WANT=5 GH_DL_RC=0 GH_NORECEIPT=""; }
leg() { mkdir -p "$STATE/run-9/candidate-$1/receipt"; echo '{"result":"green","gates":[]}' > "$STATE/run-9/candidate-$1/receipt/release-validation.json"; }

fresh; leg a; pull_receipts 9 >/dev/null
ok "(i) 5 legs after merge"  '[ "$(ls -d "$STATE"/run-9/candidate-* | wc -l)" = 5 ]'
ok "(i) red lists 5 results" '[ "$(jq length "$STATE/red-9.json")" = 5 ]'
ok "(i) no temp dir left"    '[ -z "$(ls -d "$STATE"/run-9.tmp.* 2>/dev/null)" ]'
fresh; GH_DL_RC=1; pull_receipts 9 >/dev/null
ok "(ii) rc 1 -> no red"     '[ ! -e "$STATE/red-9.json" ]'
fresh; GH_LEGS="a b c d"; pull_receipts 9 >/dev/null
ok "(iii) 4/5 -> no red"     '[ ! -e "$STATE/red-9.json" ]'
ok "(iii) incomplete logged" 'grep -q "run 9 incomplete (4/5 legs), retry next tick" "$STATE/tick.log"'
GH_LEGS="a b c d e"; pull_receipts 9 >/dev/null
ok "(iii) next tick completes" '[ "$(jq length "$STATE/red-9.json")" = 5 ]'
fresh; echo '[{"result":"green"}]' > "$STATE/red-9.json"; pull_receipts 9 >/dev/null
ok "(iv) red present -> gh not called" '[ ! -s "$GH_CALLS" ]'
fresh; GH_LEGS="a b c d" GH_NORECEIPT="e"; pull_receipts 9 >/dev/null
ok "(v) a leg without a receipt -> red from the other 4, no endless retry" '[ "$(jq length "$STATE/red-9.json")" = 4 ]'

. "$here/loop.sh"
cd "$W"; mkdir -p state; echo '{"last_run":{"databaseId":2}}' > state/tick-1.json; t=state/tick-1.json
echo '[]' > state/red-2.json; sleep 1; echo '[]' > state/red-1.json
ok "pick_red prefers last run's red over newer mtime" '[ "$(pick_red)" = state/red-2.json ]'
rm state/red-2.json
ok "pick_red falls back to newest" '[ "$(pick_red)" = state/red-1.json ]'
exit $fail
