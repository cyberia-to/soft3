---
tags: cyber, soft3, schema
crystal-type: spec
crystal-domain: cyber
---
# schema

canonical wire format definitions for the soft3 stack. all language SDKs generate types from here.

## scope

| message | format | status |
|---------|--------|--------|
| signal (binary) | fixed-width records | defined — see [[cybergraph/specs/graph]] |
| query request | TBD | pending |
| query response + proof | TBD | pending — blocked on lens serde |
| checkpoint | TBD | pending |
| cyberlink (7-tuple) | fixed-width | defined — see [[cybergraph/specs/cyberlink]] |

## signal binary format

from [[cybergraph/specs/graph]]:

```
signal header (44 bytes):
  [0..32]   ν   neuron (hemera hash, 32 B)
  [32..40]  t   unix timestamp, seconds (u64 little-endian)
  [40..44]  n   link count (u32 little-endian, n ≥ 1)

link record (105 bytes), repeated n times:
  [0..32]    p   from (hemera hash, 32 B)
  [32..64]   q   to (hemera hash, 32 B)
  [64..96]   τ   token (hemera hash, 32 B)
  [96..104]  a   stake amount (u64 LE)
  [104..105] v   valence (i8: -1, 0, +1)

total: 44 + 105·n bytes
```

## query wire format

not yet defined. will cover:
- request: `(dimension, key, bbg_root)` → 65 bytes fixed
- response: `(value_bytes, commitment, opening, point)` → variable

blocked on lens `Commitment`/`Opening` serde stabilisation.

## checkpoint format

```
checkpoint (232 bytes):
  [0..32]    bbg_root       (32 B)
  [32..224]  folding_acc    (192 B — zheng accumulator, TBD exact size)
  [224..232] height         (u64 LE)
```

accumulator size depends on zheng proof system stabilisation.
