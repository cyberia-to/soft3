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
| subgraph under soft3 | `cyber.page/soft3/<name>/` | cybergraph · bbg · nox · zheng · tru · hemera · strata · lens · mudra · radio · tape · foculus |
| concept page at root | `cyber.page/<name>/` | cell · tok · soma · mir · glia · inf · neural · cyb |
| unpublished | 404 everywhere | honeycrisp · trident · rune · wysm · eidos · prysm · fs · ward · kern |

locally all 29 resolve (private repos included in the local build); the
publish set is narrower, so the site's links rotted silently.

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

## the reconciliation queue

to retire every fallback, publish these as subgraphs (public repos —
add to the cyber workspace publish set): honeycrisp, trident, rune,
wysm, eidos, prysm. then create the missing pages: `kern` (the shader
component that wraps wgpu — repo does not exist yet), `fs` and `ward`
(pages live in the cyb subgraph; publish or mirror them). each landing
turns a GitHub/stack-page fallback back into its canonical address —
`check-links.nu` stays green through every step.

## the deeper cut (optional, later)

the site's triad panels restate the README table by hand. if the drift
returns, generate the panel markup from the table at deploy time — one
source, two renderings. deferred until the table stabilizes.
