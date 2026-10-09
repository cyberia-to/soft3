#!/usr/bin/env nu
# stale-proof-claims.nu — the phase-5 gate of proposals/proof-system-repair.md
#
# greps the whole stack for statements that present the old, unsound zheng/lens
# design as current. exits 1 if any hit remains outside the allow-list.
#
# usage:  nu scripts/stale-proof-claims.nu            # from ~/cyber/soft3
#         nu scripts/stale-proof-claims.nu --root ~/cyber --json
#
# the ledger of every hit as of 2026-10-09 is proposals/proof-system-repair.md §C.

def main [
    --root: string = "~/cyber"   # the workspace that holds every repo
    --json                        # machine-readable output
] {
    let root = ($root | path expand)

    # patterns, one per family of stale claim (ERE, case-insensitive)
    let patterns = [
        '~ ?1\.3 KiB'                         # the old opening size
        '~ ?2(\.[0-9])? ?Ki?B (proof|per|at|regardless|constant)'  # the old proof size
        '~ ?2\.4 KiB'
        '~ ?200 ?(bytes|B)\b.*(accumulat|proof|opening|checkpoint)'
        '240[- ]byte'
        '~ ?75 (bytes|B)'
        '~ ?22 ?kb'
        '~ ?5 ?(μs|us|µs)'                    # the old verify time
        '10[-–]50 ?(μs|us|µs)'
        '~ ?50 ?(μs|us|µs)'
        '100 nanoseconds|~ ?100 ?ns|~ ?0\.1 ?(μs|us|µs)'
        '290 ?(μs|us|µs)'
        'Merkle[- ]free'
        'no Merkle (verification|paths|tree)'
        'HyperNova'
        '~ ?30 field op'
        '~ ?825'
        '~ ?89 (constraints|constraint)'
        '89 constraints'
        'algebraic Fiat'
        '8\.7×'
        '~ ?3 hemera calls'
        '144 ?K (to|→) 0'
        'accumulator IS the proof'
        '2\^-118|2⁻¹¹⁸'
        '2\^-512|2⁻⁵¹²'
        'kd/p negligible'
        '256-bit classical'
    ]

    # directories that are archives, vendored trees, build output, or the ledger itself
    let exclude_dirs = [target build dist node_modules .git .node-builds .worktrees .stands .rel04 zheng-022 cyb-014 .vendor refs]
    let allow_files = [
        'soft3/proposals/proof-system-repair.md'
        'soft3/scripts/stale-proof-claims.nu'
        'zheng/CHANGELOG.md'
        'joy/CHANGELOG.md'
        'cyberia-blog/blog/2026_03_24.md'
        'cyberia-blog/blog/2026_03_26.md'
        'cyberia-blog/blog/2026_03_27.md'
        'cyberia-blog/blog/2026_09_12.md'
        'warriors/audit/parano1d-vs-uhash-2026-09-27.md'
    ]

    let excl = ($exclude_dirs | each {|d| $"--exclude-dir=($d)" })
    let pat = ($patterns | str join '|')

    let raw = (
        ^grep -rInE --include='*.md' --include='*.rs' --include='*.toml' --include='*.nu' ...$excl -e $pat $root
        | complete
    )
    let hits = (
        $raw.stdout
        | lines
        | each {|l|
            let parts = ($l | split column ':' file line text -n 3)
            $parts | first
          }
        | where {|h| not ($h.text | str trim | str starts-with '> superseded:') }
        | where {|h|
            let rel = ($h.file | str replace $"($root)/" '')
            not ($allow_files | any {|a| $rel | str starts-with $a })
          }
        | update file {|h| $h.file | str replace $"($root)/" '' }
    )

    if $json {
        $hits | to json
    } else {
        if ($hits | is-empty) {
            print "stale-proof-claims: 0 hits — the ledger is closed"
        } else {
            print ($hits | select file line text | update text {|h| $h.text | str trim | str substring 0..140 } | table -e)
            print $"stale-proof-claims: ($hits | length) hits in ($hits | get file | uniq | length) files"
        }
    }

    if not ($hits | is-empty) { exit 1 }
}
