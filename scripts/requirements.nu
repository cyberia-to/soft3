#!/usr/bin/env nu
# requirements.nu — the counter of cyberia/dev § requirements.
#
# reads a specification whose normative headings end with ` · ID` and carry a
# cases table, and the map audit/<product>/map.md (requirement · code · test ·
# proof · state); prints one row per requirement with the map's state,
# downgraded to `open` when the clause has no case or the row names no code or
# test, and to `stale` when any commit touched the code after the receipt
# landed. run from the repository root.
#
#   nu scripts/requirements.nu [--spec specs/chain.md] [--map audit/chain/map.md] [--dots]

def main [--spec: string = "specs/chain.md", --map: string = "audit/chain/map.md", --dots] {
  let spec_lines = (open --raw $spec | lines)
  let ids = ($spec_lines | each {|l| $l | parse -r '^#{1,6} .*· (?<id>[A-Z][A-Z0-9-]+)\s*$' } | flatten | get id)
  let cases = ($spec_lines | each {|l| $l | parse -r '^\| (?<id>[A-Z][A-Z0-9-]+)\.[a-z] \| (?<kind>positive|negative) \|' } | flatten | group-by id | transpose id rows | each {|g| {id: $g.id, n: ($g.rows | length)}})
  let rows = (open --raw $map | lines | where ($it | str starts-with "| ") | each {|l|
      let c = ($l | str trim | str trim -c '|' | split row '|' | each { str trim })
      if (($c | length) >= 5) and ($c.0 =~ '^[A-Z][A-Z0-9-]+$') { {id: $c.0, code: $c.1, test: $c.2, proof: $c.3, state: $c.4} } else { null }
    } | compact)
  let proof_dir = ($map | path dirname)
  let states = ["open" "stale" "simulated" "measured" "proven"]
  let out = ($ids | each {|rid|
    let n = ($cases | where id == $rid | get -o 0.n | default 0)
    let r = ($rows | where id == $rid | get -o 0)
    mut state = "open"
    mut why = ""
    if $n == 0 { $why = "placeholder: no case" } else if ($r == null) { $why = "not in the map" } else if ($r.code in ["—" ""]) or ($r.test in ["—" ""]) { $why = "no code or no test in the map" } else {
      $state = (if ($r.state in $states) { $r.state } else { "open" })
      if ($r.proof in ["—" ""]) { if $state != "open" { $state = "open"; $why = "no receipt" } } else {
        let receipt = ($proof_dir | path join $r.proof)
        let landed = (^git log --format=%H --diff-filter=A -- $receipt | lines | last | default "")
        if (not ($receipt | path exists)) or ($landed | is-empty) { $state = "open"; $why = "receipt missing" } else {
          let paths = ($r.code | split row ';' | each { str trim } | where ($it | is-not-empty))
          let missing = ($paths | where (not ($it | path exists)))
          if ($missing | length) > 0 { $state = "open"; $why = $"code path missing: ($missing | str join ', ')" } else {
            let changed = (^git log --oneline $"($landed)..HEAD" -- ...$paths | lines | where ($it | is-not-empty))
            if ($changed | length) > 0 { $state = "stale"; $why = $"($changed | length) commit\(s\) touched the code after ($r.proof)" }
          }
        }
      }
    }
    {requirement: $rid, cases: $n, state: $state, why: $why}
  })
  let counts = ($states | each {|s| {state: $s, n: ($out | where state == $s | length)}})
  let red = (($counts | where state in ["open" "stale"] | get n | math sum))
  let yellow = ($counts | where state == "simulated" | get n | math sum)
  let green = ($counts | where state in ["measured" "proven"] | get n | math sum)
  if $dots { print $"🔴 ($red) 🟡 ($yellow) 🟢 ($green) of ($ids | length)"; return }
  print ($out | table -e)
  print ""
  let summary = ($counts | each {|c| $"($c.state) ($c.n)" } | str join '  ')
  let ncases = ($cases | get n | math sum)
  print $"($summary)  of ($ids | length) requirements, ($ncases) cases"
  print $"dots: 🔴 ($red) 🟡 ($yellow) 🟢 ($green)"
  for r in $rows { if not ($r.id in $ids) { print $"note: map names ($r.id), which the spec does not carry" } }
}
