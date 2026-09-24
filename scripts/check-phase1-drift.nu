#!/usr/bin/env nu
# Compare release/phase1.toml's pinned sibling revisions against each
# sibling's live origin/<branch> HEAD; fail on drift.
# Usage:
#   nu soft3/scripts/check-phase1-drift.nu
#   nu soft3/scripts/check-phase1-drift.nu --root ~/cyber --manifest release/phase1.toml

def main [
  --root: path = '~/cyber'        # parent directory holding sibling checkouts
  --manifest: path = 'release/phase1.toml'
] {
  let root = ($root | path expand)
  let manifest_path = ($manifest | path expand)
  if not ($manifest_path | path exists) {
    print $"FAIL — manifest not found: ($manifest_path)"
    exit 1
  }
  let siblings = (open $manifest_path | get sibling)

  mut rows = []
  mut drift = 0
  mut missing = 0

  for s in $siblings {
    if ($s.rev? | is-empty) {
      $rows = ($rows | append { name: $s.name, pin: '—', live: '—', status: $"skipped: ($s.note)" })
      continue
    }
    let dir = $"($root)/($s.name)"
    if not ($dir | path exists) {
      $missing += 1
      $rows = ($rows | append { name: $s.name, pin: $s.rev, live: '?', status: 'FAIL: no local checkout' })
      continue
    }
    let live = (try {
      ^git -C $dir rev-parse $"origin/($s.branch)" | str trim
    } catch { '' })
    if ($live | is-empty) {
      $missing += 1
      $rows = ($rows | append { name: $s.name, pin: $s.rev, live: '?', status: $"FAIL: origin/($s.branch) not resolvable" })
    } else if $live == $s.rev {
      $rows = ($rows | append { name: $s.name, pin: $s.rev, live: $live, status: 'OK' })
    } else {
      $drift += 1
      $rows = ($rows | append { name: $s.name, pin: $s.rev, live: $live, status: 'DRIFT' })
    }
  }

  print ($rows | table)

  if $drift > 0 or $missing > 0 {
    print $"FAIL — ($drift) drifted, ($missing) unresolvable, against ($manifest_path)"
    exit 1
  }
  print $"OK — ($siblings | where rev? != null | length) pinned siblings match ($manifest_path)"
}
