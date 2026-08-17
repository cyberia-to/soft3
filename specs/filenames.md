---
title: filenames
tags: cyber, soft3, spec, tooling
crystal-type: spec
crystal-domain: cyber
status: active
alias: file names, NTFS, Windows paths, path safety
---
# filenames

cross-platform path rules for every repo under the cyber workspace. source trees and git-tracked paths must open on macOS, Linux, and **Windows NTFS** without rename.

companion: run `nu soft3/scripts/check-filenames.nu` (or the Python fallback) before commit.

## forbidden in any path component

characters (Win32 / NTFS):

```text
< > : " / \ | ? *
```

also forbidden:

- ASCII control characters (`U+0000`–`U+001F`)
- trailing **space** or trailing **dot** (NTFS strips or rejects them)
- leading space
- Windows reserved basenames (case-insensitive), with or without extension:
  `CON` `PRN` `AUX` `NUL` `COM1`–`COM9` `LPT1`–`LPT9`

## preferred form

| prefer | avoid |
|--------|--------|
| `nox - frozen provable computer.md` | `nox: frozen….md` |
| `salmonella-spp` (slug) | `salmonella-spp.` |
| `foo-bar` | `foo\|bar` `foo?bar` |
| `con_` (if title is CON) | `CON` as path |

use ASCII hyphens for word breaks. wiki **titles** may keep punctuation; **paths and slugs** must be sanitized (see [[optica]] `slugify_page_name`).

## length

- keep each path component well under 255 bytes (APFS/NTFS limit)
- optica slugs truncate around 200 so `…/index.html` still fits

## case

Windows paths are case-**insensitive**. do not rely on two files that differ only by case in the same directory (`Readme.md` vs `readme.md`).

## generated output

build directories (`build/`, `dist/`, `target/`) must use the same sanitization as sources. generators (optica slugify, export tools) strip NTFS-illegal characters so a Windows clone can rebuild the site.

## check

from any repo root:

```bash
# workspace-wide (from ~/cyber)
nu soft3/scripts/check-filenames.nu --root ~/cyber

# single repo
nu soft3/scripts/check-filenames.nu
```

CI and agents: fail the change if the checker reports any path.

## agents

when creating or renaming files:

1. never put `:` `\|` `?` `*` `<` `>` `"` `\\` in a filename
2. never end a filename or directory with `.` or space
3. never use Windows reserved device names as basenames
4. prefer `kebab-case` or spaced words with hyphens over exotic punctuation
