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
# the manifest read and the receipt reduction are release/director_tick.py, tested by the release suite;
# the parser writes a file, so a sibling it cannot read fails the tick loudly instead of shortening the list
TICKPY=$DIR/mirrors/soft3/release/director_tick.py; SIBLINGS=$STATE/siblings.txt
python3 "$TICKPY" siblings "$MANIFEST" > "$SIBLINGS" 2> "$STATE/siblings.err"
if [ -s "$STATE/siblings.err" ]; then log "sense: $(cat "$STATE/siblings.err")"; fi
heads="{}"; unreadable=""
while read -r repo branch pin; do
  # ls-remote exits 0 and prints nothing when the branch is renamed or deleted: that is a sibling
  # that could not be sensed, never a sibling that moved — it is named in the log, and drift is false
  head=$(git ls-remote -q "https://github.com/$repo" "refs/heads/$branch" | cut -f1)
  [ -z "$head" ] && unreadable="$unreadable $repo#$branch"
  heads=$(jq -c --arg r "$repo" --arg h "$head" --arg p "$pin" '. + {($r): {head:$h, pin:$p, drift: ($h != "" and $h != $p), unreadable: ($h == "")}}' <<<"$heads")
done < "$SIBLINGS"
if [ -n "$unreadable" ]; then log "sense: no ref answered, the branch is renamed or deleted —$unreadable"; fi
prs="[]"; run="null"
[ -z "${NOGH:-}" ] && prs=$(gh pr list -R $ORG/soft3 --state open --json number,title,labels,mergeable,headRefName,updatedAt -q '[.[] | {n:.number,t:.title,labels:[.labels[].name],mergeable:.mergeable,branch:.headRefName}]')
[ -z "${NOGH:-}" ] && run=$(gh run list -R $ORG/soft3 --workflow "Release train" -L 1 --json databaseId,status,conclusion,createdAt -q '.[0]')
jq -n --argjson heads "$heads" --argjson prs "$prs" --argjson run "$run" --arg at "$NOW" '{at:$at, heads:$heads, prs:$prs, last_run:$run}' > "$STATE/tick-$NOW.json"
prev=$(ls -1 "$STATE"/tick-*.json 2>/dev/null | tail -2 | head -1)
changed=1; [ -n "$prev" ] && [ "$prev" != "$STATE/tick-$NOW.json" ] && diff -q <(jq -S 'del(.at)' "$prev") <(jq -S 'del(.at)' "$STATE/tick-$NOW.json") >/dev/null && changed=0
drift=$(jq '[.heads[] | select(.drift)] | length' "$STATE/tick-$NOW.json")
log "sense: drift on $drift of $(jq '.heads|length' "$STATE/tick-$NOW.json") siblings, $(jq '.prs|length' "$STATE/tick-$NOW.json") open PRs, last run $(jq -r '.last_run.conclusion // .last_run.status // "unknown"' "$STATE/tick-$NOW.json"), changed=$changed"

# 2 qualify: the sensed head set moved since the last dispatch → dispatch the gates without a cut, once.
# qualified-heads alone decides it: a dispatch that failed leaves it unwritten, so the next tick retries.
[ -n "${NOGH:-}" ] && exit 0
if [ "$(jq -r '.last_run.status' "$STATE/tick-$NOW.json")" = completed ]; then
  if [ ! -f "$STATE/qualified-heads" ] || ! diff -q <(jq -S .heads "$STATE/tick-$NOW.json") "$STATE/qualified-heads" >/dev/null; then
    if gh workflow run release-train.yml -R $ORG/soft3 -f cut=false; then
      jq -S .heads "$STATE/tick-$NOW.json" > "$STATE/qualified-heads"; log "qualify: dispatched the gates (cut=false)"
    else
      log "qualify: the dispatch failed — qualified-heads left unwritten, the next tick retries this head set"
    fi
  fi
fi
# receipts of the last completed run: the reduction is kept, the platform archives are not
rid=$(jq -r '.last_run.databaseId // empty' "$STATE/tick-$NOW.json")
if [ -n "$rid" ] && [ ! -f "$STATE/red-$rid.json" ] && [ "$(jq -r '.last_run.status' "$STATE/tick-$NOW.json")" = completed ]; then
  download=$(mktemp -d "$STATE/run-$rid.XXXXXX")
  if gh run download -R $ORG/soft3 "$rid" -D "$download" -p 'candidate-*' 2>/dev/null \
     && python3 "$TICKPY" receipts "$download"/*/receipt/release-validation.json > "$download/red.json"; then
    mv "$download/red.json" "$STATE/red-$rid.json"
    log "receipts: run $rid → $(jq -r '[.[].result] | unique | join(",")' "$STATE/red-$rid.json")"
  else
    log "receipts: run $rid — no receipt downloaded or none could be read; red-$rid.json not written, the next tick retries"
  fi
  rm -rf "$download"
fi

# 3 think: the sessions are loop.sh's — the tick only senses, qualifies and pulls receipts
# 4 report: the first tick after the noon cut writes the day's report (the body does it, in the think above, when red-*.json for today's cut exists)
find "$STATE" -maxdepth 1 \( -name 'tick-*.json' -o -name 'red-*.json' \) -mtime +7 -delete
find "$STATE" -maxdepth 1 -name 'run-*' -mtime +1 -exec rm -rf {} +
