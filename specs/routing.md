---
title: routing — the wire obeys the graph
tags: cyber, soft3, spec
crystal-type: spec
crystal-domain: cyber
alias: routing, the wire obeys the graph, locus routing, greedy routing, hyperbolic routing
---

# routing — the wire obeys the graph

How a soft3 node decides whom to dial and whom to forward to. The rule is a
composition: the address record comes from [cybergraph](https://github.com/cyberia-to/cybergraph/blob/master/specs/address.md)
(FOLLOW, ANTENNA, SOCKET, LOCUS), the coordinate from
[tru](https://github.com/cyberia-to/tru/blob/master/specs/locus.md) (locus),
the transport from [radio](https://github.com/cyberia-to/radio). Radio never
sees a coordinate: it receives an endpoint to dial and a stream to open. Tru
never dials. The node is the only place where the three meet, so the rule lives
here.

There is no second network. The follow graph a neuron publishes is at once its
subscription list, its transport neighbourhood and its routing table. A neuron
earns a place near the centre of the disk by being followed; a sybil sits on
the rim, where nobody forwards to it.

## what routes

| request | target | carried by |
|---|---|---|
| sync a book | the neuron that owns it | FOLLOW; the book's signals stream over radio |
| fetch a file by particle | the neuron whose book references the particle | radio blobs (cyber-bao verified streaming) |
| settle gossip, reconciliation | peers of the epoch's cluster | foculus chooses peers by locus sector |
| a signal for a neuron not yet followed | that neuron | greedy forwarding below |

Client-to-node traffic (HTTP ingress) does not route: a client knows its node.

## the rule

Given a target neuron $t$ and the node's own neuron $s$:

1. **direct.** If $s$ holds an ANTENNA or SOCKET link of $t$, dial it. Done.
2. **greedy.** Otherwise forward to the live neighbour $n \in \mathrm{FOLLOW}(s)$
   that minimises $d(n, t)$ by locus, provided $d(n, t) < d(s, t)$. The neighbour
   applies the same rule. Each hop carries the hop count and the minimum
   distance seen so far.
3. **local minimum.** If no neighbour is nearer than $s$, switch the request to
   pressure mode (gravity-pressure routing, Cvetkovski and Crovella 2009): forward
   to the neighbour with the fewest visits recorded in the request, until some
   node is nearer than the recorded minimum; then return to greedy. This turns
   dead ends into detours and keeps delivery near complete on scale-free graphs.
4. **bound.** A request dies after $H = \lceil 2 \log_2 N \rceil$ hops or after
   the same node is visited three times in pressure mode. The origin then asks
   its followed neurons for $t$'s address record directly (one hop) and gives up
   if none has it.
5. **unknown locus.** If $t$ has no LOCUS link the node holds, step 4 applies
   immediately: address records propagate by the same gossip as every other link,
   so a reachable neuron's LOCUS arrives with the first book that follows it.

Distances are compared in the fixed-point form tru defines; the ordering
surrogate $x = r_n + r_t + 2 \ln \sin(\Delta\theta / 2)$ is admitted wherever
only the comparison matters. No float enters the decision.

## what the node keeps

- the address records of every neuron it follows and of every neuron whose book
  it holds (they are links in those books, nothing extra);
- the liveness of each followed neighbour (last successful radio round trip);
- nothing else. There is no routing table, no bucket, no identifier space.

## privacy

Only address records enter the decision, and a neuron publishes them by choice.
Knowledge links, whose author is private, never influence a route. A request in
flight carries the target's name, the hop count, the minimum distance and the
visited set in pressure mode; it does not carry the origin beyond the first hop's
knowledge of who handed it over.

## gate

The property in `cyber/launch.md` (row 42): greedy forwarding over FOLLOW links
reaches at least 97 % of targets within $H$ hops, with path stretch at most 1.1
over the shortest follow path, measured on the cyb fleet harness and on a replay
of the bostrom follow graph from the burial snapshot; radio's interface is
unchanged by the feature. Until measured, the rule is a specification and the
three-node proof of 2026-09 is the only evidence.

## references

- Krioukov, Papadopoulos, Kitsak, Vahdat, Boguñá. Hyperbolic geometry of complex networks. Phys. Rev. E 82, 036106 (2010).
- Boguñá, Papadopoulos, Krioukov. Sustaining the internet with hyperbolic mapping. Nat. Commun. 1, 62 (2010).
- Papadopoulos, Psomas, Krioukov. Network mapping by replaying hyperbolic growth. IEEE/ACM ToN 23 (2015).
- Cvetkovski, Crovella. Hyperbolic embedding and routing for dynamic graphs. INFOCOM (2009).
