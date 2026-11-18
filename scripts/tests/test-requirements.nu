#!/usr/bin/env nu
# test-requirements.nu — the fixtures of scripts/requirements.nu.
#
# builds a throwaway git repository whose spec, map and history produce one
# row of every outcome the counter reports — a clause with no case, a clause
# the map does not carry, a row with no test, a receipt that is not there, a
# receipt written and never committed, code touched after the receipt landed,
# and a row that stays proven — and checks the state and the reason the
# counter gives each one, and the note for a map row the spec does not carry.
#
#   nu scripts/tests/test-requirements.nu

const HERE = (path self | path dirname)
use ($HERE | path join .. requirements.nu) requirement-rows

const SPEC = '# the fixture specification

## one · FIX-PROVEN

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-PROVEN.a | positive | the blessed vector | regenerated | it is the same |

## two · FIX-STALE

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-STALE.a | positive | the receipt | read | it observed this code |

## three · FIX-NO-TEST

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-NO-TEST.a | positive | a clause | checked | nothing checks it |

## four · FIX-NO-RECEIPT

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-NO-RECEIPT.a | negative | a receipt that was never written | read | the state is open |

## five · FIX-UNCOMMITTED

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-UNCOMMITTED.a | negative | a receipt written and not committed | read | the state is open |

## six · FIX-UNMAPPED

| case | kind | given | when | then |
|---|---|---|---|---|
| FIX-UNMAPPED.a | positive | a clause the map forgot | counted | the state is open |

## seven · FIX-NO-CASE

This clause carries no case table at all.
'

const MAP = '# the fixture map

| requirement | code | test | proof | state |
|---|---|---|---|---|
| FIX-PROVEN | src/b.rs | b.rs::holds | r1.md | proven |
| FIX-STALE | src/a.rs | a.rs::holds | r1.md | proven |
| FIX-NO-TEST | src/a.rs | — | r1.md | simulated |
| FIX-NO-RECEIPT | src/b.rs | b.rs::holds | r9.md | proven |
| FIX-UNCOMMITTED | src/b.rs | b.rs::holds | r2.md | proven |
| FIX-NO-CASE | src/b.rs | b.rs::holds | r1.md | proven |
| FIX-GHOST | src/b.rs | b.rs::holds | r1.md | proven |
'

# the repository the counter is run in: four commits, and one receipt left
# out of the index.
def build [dir: string] {
  mkdir ($dir | path join "src") ($dir | path join "specs") ($dir | path join "audit" "fixture")
  $SPEC | save -f ($dir | path join "specs" "fixture.md")
  $MAP | save -f ($dir | path join "audit" "fixture" "map.md")
  "the code of FIX-STALE\n" | save -f ($dir | path join "src" "a.rs")
  "the code of FIX-PROVEN\n" | save -f ($dir | path join "src" "b.rs")
  ^git -C $dir init -q
  ^git -C $dir config user.email fixture@example.invalid
  ^git -C $dir config user.name fixture
  ^git -C $dir add -A
  ^git -C $dir commit -q -m "the spec, the map and the code"
  "the receipt of r1\n" | save -f ($dir | path join "audit" "fixture" "r1.md")
  ^git -C $dir add -A
  ^git -C $dir commit -q -m "the receipt r1 lands"
  "the code of FIX-STALE, changed after r1\n" | save -f ($dir | path join "src" "a.rs")
  ^git -C $dir add -A
  ^git -C $dir commit -q -m "the code of FIX-STALE changes"
  # written, never committed: git knows no commit that added it
  "the receipt of r2\n" | save -f ($dir | path join "audit" "fixture" "r2.md")
}

def main [] {
  let dir = ($nu.temp-dir | path join $"requirements-fixture-(random chars -l 8)")
  build $dir
  let count = (do { cd $dir; requirement-rows "specs/fixture.md" "audit/fixture/map.md" })
  rm -rf $dir

  let expected = [
    [requirement cases state why];
    [FIX-PROVEN 1 proven ""]
    [FIX-STALE 1 stale "1 commit(s) touched the code after r1.md"]
    [FIX-NO-TEST 1 open "no code or no test in the map"]
    [FIX-NO-RECEIPT 1 open "receipt missing"]
    [FIX-UNCOMMITTED 1 open "receipt missing"]
    [FIX-UNMAPPED 1 open "not in the map"]
    [FIX-NO-CASE 0 open "placeholder: no case"]
  ]
  mut bad = 0
  for e in $expected {
    let got = ($count.rows | where requirement == $e.requirement | get -o 0)
    if $got == $e { print $"ok   ($e.requirement) — ($e.state)" } else {
      $bad += 1
      print $"FAIL ($e.requirement): expected ($e) got ($got)"
    }
  }
  if ($count.rows | length) != ($expected | length) {
    $bad += 1
    print $"FAIL the counter read ($count.rows | length) requirements, the spec carries ($expected | length)"
  }
  if $count.extra != ["FIX-GHOST"] {
    $bad += 1
    print $"FAIL the map rows the spec does not carry: expected [FIX-GHOST] got ($count.extra)"
  } else { print "ok   FIX-GHOST — named by the map, not by the spec" }
  if $bad > 0 { error make -u {msg: $"($bad) check\(s\) failed"} }
  print $"(($expected | length) + 1) checks passed"
}
