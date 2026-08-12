---
tags: cyber, soft3, proposal
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-08-12
---
# One component, one page: the URL canon

> every component of the stack answers at one canonical address on
> [[cyber.page]]. the site, the README table, and the graph agree, and a
> link checker keeps them agreeing.

## the mess this crystallizes

the stack is described in three places — the README triad table, the
site's triad panels, and the published graph — and each drifted on its
own. an audit of the 29 component links (2026-08-12) found three species
of address on production:

| species | address | components |
|---|---|---|
| subgraph under soft3 | `cyber.page/soft3/<name>/` | cybergraph · bbg · nox · zheng · tru · hemera · strata · lens · mudra · radio · tape · foculus · lytics · tok |
| subgraph under cyb | `cyber.page/cyb/<name>/` | honeycrisp · wysm · prysm · fs · ward |
| subgraph under neural | `cyber.page/neural/<name>/` | trident · eidos · rune · inf |
| concept page at root | `cyber.page/<name>/` | cell · soma · mir · glia · neural · cyb · kern |

the registry (`cyber/subgraphs.toml`) is the single source of the mount
map: `parent` decides the namespace. the first audit misread six of
these as unpublished when they were simply mounted under `cyb/` and
`neural/` rather than `soft3/` — the mess was link paths, never missing
pages. the site now links every component at its registry-derived
address; the only GitHub links left are the soft3 repo's own header and
footer.

## the canon

1. a component that is a repo publishes as a subgraph and answers at
   `cyber.page/soft3/<name>/`. its README is the page.
2. a component that is a concept page of the cyber graph answers at
   `cyber.page/<name>/`.
3. until a component's page is published, its link points at the next
   thing that exists and is honest: the GitHub repo when public, the
   [[soft3]] stack page otherwise. a dead link is never shipped.
4. `site/check-links.nu` enforces this: it curls every external href in
   the site and fails on any non-200. run it before every deploy.

## the reconciliation record

resolved 2026-08-12: every component link on the site points at its
canonical cyber.page address; zero GitHub fallbacks remain in the triad
panels. two registry drifts were fixed on the way — `lytics` mounted at
`soft3/lytics`, and `tok` flipped public with `repo = "plumb"` (the org
repo was renamed; publish.yml now honors the `repo` field when cloning).
`fs` stays a page inside the cyb subgraph (`cyb/fs`) while its own repo
remains private; `kern` has a root concept page and still needs its
repo.

known tooling drift, deliberately not applied: `scripts/sync-org.nu`
still reads per-repo `subgraphs/*.md` declarations and proposes ~50
"adopt" stubs that would duplicate `subgraphs.toml`. the registry moved
to the single toml; sync-org needs the same migration before its
--apply is safe again.

## the deeper cut (optional, later)

the site's triad panels restate the README table by hand. if the drift
returns, generate the panel markup from the table at deploy time — one
source, two renderings. deferred until the table stabilizes.
