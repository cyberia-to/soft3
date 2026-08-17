#!/usr/bin/env nu
# Fail if any tracked (or filesystem) path is invalid on Windows NTFS.
# Usage:
#   nu soft3/scripts/check-filenames.nu
#   nu soft3/scripts/check-filenames.nu --root ~/cyber
#   nu soft3/scripts/check-filenames.nu --fs   # also scan untracked (skip build/target)

def is-reserved [stem: string] {
  let s = ($stem | str upcase)
  let reserved = [
    CON PRN AUX NUL
    COM1 COM2 COM3 COM4 COM5 COM6 COM7 COM8 COM9
    LPT1 LPT2 LPT3 LPT4 LPT5 LPT6 LPT7 LPT8 LPT9
  ]
  $reserved | any {|r| $r == $s }
}

def check-component [name: string] {
  mut problems = []
  let forbidden = ['<' '>' ':' '"' '/' '\' '|' '?' '*']
  for c in ($name | split chars) {
    if $c in $forbidden {
      $problems = ($problems | append $"forbidden char '($c)'")
    }
    let o = ($c | into binary | first)
    # control chars roughly: skip complex, check common
  }
  if ($name | str ends-with ' ') or ($name | str ends-with '.') {
    $problems = ($problems | append 'trailing space or dot')
  }
  if ($name | str starts-with ' ') {
    $problems = ($problems | append 'leading space')
  }
  # stem for reserved (strip one extension)
  let stem = (if ($name | str contains '.') {
    $name | split row '.' | first
  } else { $name })
  if (is-reserved $stem) {
    $problems = ($problems | append $"reserved device name '($stem)'")
  }
  $problems
}

def check-path [rel: string] {
  mut all = []
  for part in ($rel | split row '/') {
    if $part == '' or $part == '.' or $part == '..' { continue }
    let p = (check-component $part)
    if not ($p | is-empty) {
      $all = ($all | append { path: $rel, part: $part, problems: $p })
    }
  }
  $all
}

def main [
  --root: path = '.',   # workspace or repo root
  --fs                  # scan filesystem too (not only git ls-files)
] {
  let root = ($root | path expand)
  mut issues = []

  # git-tracked paths in this repo or every nested .git under root
  let gits = (glob $"($root)/**/.git" | where {|g| ($g | path type) == 'dir' })
  let git_dirs = if ($"($root)/.git" | path exists) {
    [$root]
  } else {
    $gits | each {|g| $g | path dirname }
  }

  for repo in $git_dirs {
    let tracked = (try {
      ^git -C $repo ls-files
      | lines
    } catch { [] })
    for f in $tracked {
      let found = (check-path $f)
      for row in $found {
        $issues = ($issues | append ($row | insert repo $repo))
      }
    }
  }

  if $fs {
    # light walk via find for untracked / build
    let found = (try {
      ^find $root
        -path '*/.git/*' -prune -o
        -path '*/node_modules/*' -prune -o
        -path '*/target/*' -prune -o
        -path '*/.vendor/*' -prune -o
        \( -name '*[:<>"|?*\\]*' -o -name '*.' \) -print
      | lines
    } catch { [] })
    for p in $found {
      let rel = (try { $p | path relative-to $root } catch { $p })
      $issues = ($issues | append {
        repo: $root
        path: $rel
        part: ($p | path basename)
        problems: ['filesystem NTFS-unsafe name']
      })
    }
  }

  if ($issues | is-empty) {
    print $"OK — no NTFS-unsafe paths under ($root)"
    return
  }

  print $"FAIL — ($issues | length) NTFS-unsafe path\(s\):"
  for i in $issues {
    print $"  ($i.repo) :: ($i.path)  [($i.part)]  ($i.problems | str join ', ')"
  }
  print "see soft3/specs/filenames.md"
  exit 1
}
