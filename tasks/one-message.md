---
tags: soft3, task, chain, signal
kind: task
status: doing
owner: claude
serves: pussy
lane: wide
date: 2026-11-03
---

# one message — the chain carries signals of cyberlinks, as soft3 specifies

## the claim

the root chain of pussy carries one kind of message, the signal of `cybergraph/specs/signal.md` and `intent.md`, with nothing added that the soft3 specifications do not already define; every action the chain needs today (transfer, claim, stake, unlock, payout owner, address announcement, coinbase, pool, revert) is expressed through it, its meaning is neural's (`neural/specs/`), and the proposal says so with a trace to the clause behind every sentence.

## the owner's corrections (2026-11-03, translated)

- the signal format of the first draft (soft3#677: `version ‖ neuron ‖ step ‖ fee ‖ n ‖ link × n ‖ payload ‖ scheme ‖ auth`) is not what the specifications say; the specification is cluttered and must be cleaned; nothing superfluous.
- "a bundle of cyberlinks committed in one step under one authorization" is false to the philosophy of the signal: one neuron creates the signal as an intent with provable conditions of completion; then any neurons may fulfil it, even in different blocks; a signal fits in one block only when it is minimal.
- the chain is implemented as if soft3 had no specification; the first draft piled noise on top.
- every open question of the first draft is already answered in the soft3 specifications; the work is to read them fully, make several passes and checks, and follow the agreed process.

## outputs

1. `proposals/one-message.md` rewritten from scratch on this branch (soft3#677).
2. a trace table in the proposal: every rule → the spec clause (repo, file, heading or line) it comes from; a rule with no clause is either removed or listed as a gap for the owner with the clause it would amend.
3. the list of what in `cybernode/specs/chain.md`, `chain-economy.md`, `chain-wire.md` contradicts or duplicates the soft3/cybergraph/neural specifications and must be removed.

## checks

- every sentence of the proposal that states a rule carries its clause; a reviewer of another model re-reads each cited clause and confirms it (review node, lenses of `cyberia/quality.md`, at most two rounds, findings with ids, each disposed).
- no format, field, scheme, word or number appears that no spec defines, unless listed as a gap with the reason.
- the signal is described as the specs describe it: an intent by one neuron with provable conditions, fulfilled by any neurons, possibly across blocks (intent → seal), minimal signals in one block.
- the proposal is shown to the owner only after the review is green.

## log

- 2026-11-03 · task opened after the owner rejected the first draft.
