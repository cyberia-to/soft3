---
title: canonical file identity decision
tags: soft3, storage, hemera, file, roadmap
status: decision-required
---
# S1 — canonical file identity

Decide one `file → particle` construction and the verification relation for its
parts before freezing storage descriptors and the public transfer protocol.
The [storage contract](../../specs/storage.md) fixes ownership. This record
defines the remaining decision and its closure criteria.

## primitive versus construction

A shared sponge primitive does not make these inputs equivalent:

| candidate | what determines the particle | consequence for partial verification |
|---|---|---|
| Flat sponge | Complete canonical content stream, with the specified length/encoding rules | A full stream can be rehashed; independent ranges require an additional verified proof relation to that digest |
| Structural commitment built from the sponge | Canonical leaves, ordered structure and root binding | Openings authenticate parts if structure, position and length are bound by that same construction |
| Sponge over a polynomial commitment | Canonical data-to-polynomial mapping and canonical Lens commitment encoding | Openings need the commitment-to-particle binding and the selected polynomial proof assumptions |

These are design alternatives, not interchangeable calls to `hash`. Radio's
BAO root cannot become a second public particle. A proof encoding can change
without changing identity only when its verifier proves the same committed
statement. A checksum conversion or a shared permutation establishes no such
equivalence.

Recommended direction for evaluation: keep Hemera's sponge as the primitive and
make the universal structural commitment the candidate for file identity, with
`.cyb` as an adapter. Compare it against the existing flat-sponge identity and
the Lens path before selection. This preserves the earlier structural design
intent while making the compatibility and proof costs explicit. It remains a
recommendation until the criteria below are met.

## decisions to close

| question | required decision |
|---|---|
| What is the file? | Exact bytes for opaque data; precise semantic equivalence and source-byte preservation for structured adapters |
| What is hashed? | Complete normative encoding/construction, byte-to-field packing, length binding, empty value and root rule; reconcile [types](../../specs/types.md) and Hemera's one-pure-hash proposal |
| Which structure is canonical? | Fixed rules for ordering, leaf/node distinction and any canonical chunk boundaries; distinguish semantic parts from physical storage segments |
| What does a range prove? | Bytes, offset/membership and total length against the same particle; explicit proof for any auxiliary commitment |
| What changes identity? | Backend, transport and physical chunking leave it invariant; define effects of semantic edits and noncanonical `.cyb` encodings |
| What security is assumed? | Selected Hemera parameters and proof-system assumptions, unresolved analysis and production qualification criteria |
| How do existing references survive? | Enumerate formats, reverify original content, record legacy-to-canonical relation and preserve provenance; specify handling of missing source bytes |

Relevant owner documents:
[[hemera/roadmap/one-pure-hash]],
[[hemera/specs/structural-commitments]],
[[hemera/specs/README]], [[file/specs/README]] and
[[lens]]. Reconcile their claims in the owning specs and link the resulting
decision here. [Coordinated migration](../migration.md) owns the wider value-model
sequence; this workstream owns the file/transfer acceptance criteria.

FS path bindings are external to the target file's content. A rename preserves
that target's particle. When an enclosing directory manifest or `.cyb` container
includes entry names, those names are part of its own content: renaming changes
the enclosing particle. Include both cases in the canonical vectors so path
metadata and semantic container fields cannot be confused.

## closure evidence

1. One normative owner document defines the construction and its relation to
   structured adapters. Every old formula is reconciled or explicitly superseded.
2. A reference implementation and independently checked vectors cover empty
   files, arbitrary binary data, trailing zeroes, boundary lengths, reordered/duplicated parts,
   truncated streams and malformed structure. Add exact-byte versus semantic
   equivalence cases for `.cyb` and at least one other structured consumer.
3. File, Cybergraph, BBG and Radio agree on the particle of each canonical case.
   Corrupt, spliced, omitted, replayed or wrongly positioned parts are rejected
   against that identity. One-shot and incremental computation agree. Backend choice and transport packetization do not
   change it. Format-specific chunking rules are tested rather than assumed.
4. Measure full hashing, first verified byte, range proof generation/verification,
   proof bytes and temporary memory over increasing file sizes and random ranges.
   Compare candidates under the same inputs; attach commands and revisions in
   `audit/`. Mathematical and security arguments accompany measurements.
5. Rehearse imports from the pinned baseline on copies. Record every mismatch,
   incomplete source and required reference rewrite. Preserve old data until a
   verified destination and recoverable mapping exist.

Compatibility vectors establish agreement. Security review establishes the
cryptographic confidence required for production; neither substitutes for the
other. S1 can select and implement a format with an explicit research status,
but S9 cannot label it production-qualified while its required security gate
remains open.
