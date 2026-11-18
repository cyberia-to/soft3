---
title: receipt
tags: cyber, soft3, cybergraph, zheng, project
crystal-type: project
crystal-domain: cyber
alias: receipt, receipts, step boundary, inclusion receipt
status: proposed
date: 2026-11-16
---
# receipt

status: proposed · date: 2026-11-16 · origin: the survey of parano1d and dregg ([[zheng/docs/explanation/zippel|zippel]] round, 2026-10-24 to 11-16); the owner: "add a prop to soft3".

A proof is evidence. A receipt is the committed step boundary that evidence attests, bound to the step before it, foldable, and held by the party the step is about. Proof systems change (zheng profiles, a zippel-checked protocol, an eidos refinement); the receipt is the one interface that survives them. Both surveyed systems arrived at the same object under the same name: parano1d's payment receipt (inclusion against a header after bodies are pruned), dregg's turn receipt (the committed postcondition the light client folds). cyber has the contents, the [[zheng]] statement and its envelope, and nothing that chains them per neuron or that a holder keeps.

## 1. the object

```
receipt = ( prev,        hemera of the author's previous receipt (genesis sentinel for the first)
            statement,   program particle, input and output particles, focus bound  — zheng's Statement
            old_root,    the root the step read
            new_root,    the root the step wrote
            effects,     hemera of the effect list: links entered, boxes moved, rewards minted
            height,      the block that includes the step
            path )       the inclusion path from the step to that block's header
```

The receipt hash is hemera over the first six fields; `path` is carried, not hashed, so one receipt verifies against the header alone. A [[signal]] yields one receipt to each of its subjects; a transaction yields one to its author. The proof σ stays inside the step; the receipt names it by the statement.

## 2. what it gives

- the light client: a chain of receipts per neuron is a chain of statements, each continuing from the last root; folding receipts is folding statements, which is what the [[proven-present|proven present]] folds anyway;
- pruning: chain.md §3.6 drops bodies; a holder's receipt still verifies against the retained header — the evidence survives the data;
- portability: a receipt is a file like any other, a particle, linkable, exportable, the thing a wallet shows and a counterparty checks;
- the explicit claim: today the verifier of a link is the scope program behind its signal; the receipt states program, inputs and roots on the object a reader holds.

## 3. the moves

1. [[cybergraph]]: `receipt` beside [[signal]] in specs — the structure above, one per subject, emitted by `apply`; the receipt chain is the neuron's chain (order.md's `prev` becomes the previous receipt's hash, one object instead of two);
2. [[zheng]]: the Statement gains `old_root` and `new_root` explicitly (today the root is one field); the fold's public inputs are receipts;
3. soft3/specs/chain.md: a transaction's receipt and the inclusion path format (fixed depth over the block's transaction root, §1.7); the node serves `receipt(txid)`;
4. [[cy]] and [[cyb]]: the wallet keeps its receipts; a payment shows its receipt; a counterparty verifies one against the header it already has;
5. bbg: nothing; receipts are files, not a dimension.

## 4. acceptance

- a receipt verifies against the header with the body pruned, on two nodes, bit-exact;
- the fold of a neuron's receipt chain equals the fold of its steps (the proven-present vectors extended by receipts);
- a tampered `effects` or `new_root` fails verification; a receipt from one chain is rejected on the other (chain id in the statement);
- cyb shows a receipt for a payment and verifies an imported one.

## 5. what is cut from the sources

parano1d's fixed-depth Poseidon2b path (ours is hemera over the transaction root), dregg's executor signature and federation id (ours binds the block), dregg's capability consumption list (that belongs to the cap chain in σ, see the [[ward]] discussion).

related: [[proven-present|the proven present]] · [[signal]] · [[zheng/specs/api|zheng api]] · [[ward]]
