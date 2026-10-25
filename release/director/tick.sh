#!/bin/bash
# the director's deterministic tick — sense, qualify, think, report. no tokens unless something changed.
# runs as the body user on agent-a; needs: git, gh (as cyberia-apparatus), jq, goose; DIR=$HOME/director
set -euo pipefail
DIR=${DIR:-$HOME/director}; STATE=$DIR/state; mkdir -p "$STATE" "$DIR/mirrors"; cd "$DIR"
ORG=cyberia-to; NOW=$(date -u +%Y%m%dT%H%M%SZ); TODAY=$(date -u +%Y-%m-%d)
log() { echo "$(date -u +%H:%M:%S) $*" | tee -a "$STATE/tick.log"; }
gh auth status >/dev/null 2>&1 || { log "gh is not signed in (the cyberia-apparatus token is missing) — sense only, no qualify, no think"; NOGH=1; }

# 1 sense: component HEADs from the registry, open PRs, last train run
MANIFEST=$DIR/mirrors/soft3/release/phase1.toml
[ -d "$DIR/mirrors/soft3/.git" ] || git clone -q --filter=blob:none https://github.com/$ORG/soft3 "$DIR/mirrors/soft3"
DEF=$(git -C "$DIR/mirrors/soft3" remote show origin | awk '/HEAD branch/ {print $NF}'); git -C "$DIR/mirrors/soft3" fetch -q origin "$DEF" && git -C "$DIR/mirrors/soft3" reset -q --hard "origin/$DEF"
heads="{}"
while read -r repo branch pin; do
  head=$(git ls-remote -q "https://github.com/$repo" "refs/heads/$branch" | cut -f1)
  heads=$(jq -c --arg r "$repo" --arg h "$head" --arg p "$pin" '. + {($r): {head:$h, pin:$p, drift: ($h != $p)}}' <<<"$heads")
done < <(python3 - "$MANIFEST" <<'PY'
import sys,tomllib
for s in tomllib.load(open(sys.argv[1],'rb'))['sibling']: print(s['repo'], s['branch'], s['rev'])
PY
)
prs="[]"; run="null"
[ -z "${NOGH:-}" ] && prs=$(gh pr list -R $ORG/soft3 --state open --json number,title,labels,mergeable,headRefName,updatedAt -q '[.[] | {n:.number,t:.title,labels:[.labels[].name],mergeable:.mergeable,branch:.headRefName}]')
[ -z "${NOGH:-}" ] && run=$(gh run list -R $ORG/soft3 --workflow "Release train" -L 1 --json databaseId,status,conclusion,createdAt -q '.[0]')
jq -n --argjson heads "$heads" --argjson prs "$prs" --argjson run "$run" --arg at "$NOW" '{at:$at, heads:$heads, prs:$prs, last_run:$run}' > "$STATE/tick-$NOW.json"
prev=$(ls -1 "$STATE"/tick-*.json 2>/dev/null | tail -2 | head -1)
changed=1; [ -n "$prev" ] && [ "$prev" != "$STATE/tick-$NOW.json" ] && diff -q <(jq -S 'del(.at)' "$prev") <(jq -S 'del(.at)' "$STATE/tick-$NOW.json") >/dev/null && changed=0
drift=$(jq '[.heads[] | select(.drift)] | length' "$STATE/tick-$NOW.json")
log "sense: drift on $drift siblings, $(jq '.prs|length' "$STATE/tick-$NOW.json") open PRs, last run $(jq -r '.last_run.conclusion // .last_run.status // "unknown"' "$STATE/tick-$NOW.json"), changed=$changed"

# 2 qualify: a HEAD moved since the last qualify → dispatch the gates without a cut, once
[ -n "${NOGH:-}" ] && exit 0
if [ "$changed" = 1 ] && [ "$(jq -r '.last_run.status' "$STATE/tick-$NOW.json")" = completed ]; then
  if [ ! -f "$STATE/qualified-heads" ] || ! diff -q <(jq -S .heads "$STATE/tick-$NOW.json") "$STATE/qualified-heads" >/dev/null; then
    gh workflow run release-train.yml -R $ORG/soft3 -f cut=false && jq -S .heads "$STATE/tick-$NOW.json" > "$STATE/qualified-heads" && log "qualify: dispatched the gates (cut=false)"
  fi
fi
# receipts of the last completed run
rid=$(jq -r '.last_run.databaseId' "$STATE/tick-$NOW.json"); if [ -n "$rid" ] && [ ! -d "$STATE/run-$rid" ] && [ "$(jq -r '.last_run.status' "$STATE/tick-$NOW.json")" = completed ]; then
  gh run download -R $ORG/soft3 "$rid" -D "$STATE/run-$rid" -p 'candidate-*' 2>/dev/null || true
  jq -s '[.[] | {result, red: [.gates[]? | select(.result!="green") | {name, detail: (.detail // .error // .reason // .command // "")}]}]' "$STATE"/run-$rid/*/receipt/release-validation.json > "$STATE/red-$rid.json" 2>/dev/null || true
  log "receipts: run $rid → $(jq -r '[.[].result] | unique | join(",")' "$STATE/red-$rid.json" 2>/dev/null)"
fi

# 3 think: the sessions are loop.sh's — the tick only senses, qualifies and pulls receipts
# 4 report: the first tick after the noon cut writes the day's report (the body does it, in the think above, when red-*.json for today's cut exists)
find "$STATE" -name 'tick-*.json' -mtime +7 -delete
