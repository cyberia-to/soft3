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

## historical publication audit — 2026-08-12

the stack is described in three places — the README triad table, the
site's triad panels, and the published graph — and each drifted on its
own. an audit of the 29 component links (2026-08-12) found three species
of address on production:

| species | address | components |
|---|---|---|
| subgraph under soft3 | `cyber.page/soft3/<name>/` | cybergraph · bbg · nox · zheng · tru · hemera · strata · lens · mudra · radio · tade · foculus · lytics · tok |
| subgraph under cyb | `cyber.page/cyb/<name>/` | honeycrisp · wysm · prysm · fs · ward |
| subgraph under neural | `cyber.page/neural/<name>/` | trident · eidos · rune · inf |
| concept page at root | `cyber.page/<name>/` | cell · soma · mir · glia · neural · cyb · kern |

The historical registry was `cyber/subgraphs.toml`; `parent` decided the namespace.
The first audit misread six of
these as unpublished when they were simply mounted under `cyb/` and
`neural/` rather than `soft3/` — the mess was link paths, never missing
pages. The recorded reconciliation linked those components at their then-current
addresses. These historical rows preserve the original `cell` concept name and
publication paths; they do not declare today's component identities or mounts.

## current source boundary — 2026-09-13

The full-project graph now belongs to
[cyberia-blog/subgraphs.toml](../../cyberia-blog/subgraphs.toml), with publishing
configuration in [publish.toml](../../cyberia-blog/publish.toml). The cyber repo
owns the protocol-only graph. A site's configured base URL and the selected
registry's `mount` / `parent` / `name` determine its target; a repository move
does not by itself establish a published URL.

The runtime repository is [neuron](../../neuron/README.md): subject identity and
optional durable progs. GraphSession belongs to the multi-neuron host;
[ward](../../cyb/specs/runtime-authority.md) and
[vault](../../cyb/specs/private-vault.md) currently have cyb host implementations.
Their rows must link to those real contracts until an independently published
component page exists. Old cell URLs may be compatibility landings with explicit
destinations; retaining them never reintroduces a second signing subject. Use
the [neuron contract](../specs/neuron.md) and the
[migration ledger](../audit/neuron-cell/implementation.md) for current semantics.

## the canon

1. A repository component's README is its source page. Publishing uses its
   declared registry mount and site base URL; the local path and remote URL are
   separate fields.
2. A concept or host-resident role links to its owning graph/spec page. A name
   in a component table alone establishes neither a repository nor a release.
3. until a component's page is published, its link points at the next
   thing that exists and is honest: the GitHub repo when public, the
   [[soft3]] stack page otherwise. a dead link is never shipped.
4. [site/check-links.nu](../site/check-links.nu) checks external HTTPS hrefs in
   `site/index.html` against live responses. Local checks must additionally
   validate relative files, diagram node/edge endpoints and compatibility
   destinations. Run publication checks against the intended build before deploy.

## historical reconciliation record

resolved 2026-08-12: every component link on the site points at its
canonical cyber.page address; zero GitHub fallbacks remain in the triad
panels. two registry drifts were fixed on the way — `lytics` mounted at
`soft3/lytics`, and `tok` flipped public with `repo = "plumb"` (the org
repo was renamed; publish.yml now honors the `repo` field when cloning).
`fs` stays a page inside the cyb subgraph (`cyb/fs`) while its own repo
remains private; `kern` has a root concept page and still needs its
repo.

The 2026-08-12 record also identified declaration drift in `scripts/sync-org.nu`.
Its current source is [cyberia-blog/scripts/sync-org.nu](../../cyberia-blog/scripts/sync-org.nu).
Any future apply must be assessed against the current workspace and registry;
the historical count of proposed stubs is not current execution evidence.

## the deeper cut (optional, later)

the site's triad panels restate the README table by hand. if the drift
returns, generate the panel markup from the table at deploy time — one
source, two renderings. deferred until the table stabilizes.
