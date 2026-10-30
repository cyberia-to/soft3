---
tags: soft3, task, proposal
alias: p1 assumptions registry, p1
kind: review
status: open
owner: master
serves: security · the assumptions of the stack
lane: contained
---
# p1 · the assumptions registry — eidos replaces lean

## claim

the owner (2026-10-27): a clear registry of the stack's scientific assumptions; lean is foreign, eidos is native, and whatever eidos lacks to replace lean must be built. the unit is one proposal page, `proposals/assumptions-registry.md`, and its index row. the pull request: [soft3#398](https://github.com/cyberia-to/soft3/pull/398).

## for

one list of everything the stack assumes, classed, with a response plan per entry; the list is the axiom file of a soft3 eidos corpus, so a theorem can only rest on a named assumption; 2026-10-05 to 10-26 (3SUM, APSP refuted 10-05; UGC proved 10-06; ETH, SETH, Gap-ETH, NSETH claimed refuted by a preprint of 10-17 rev. 10-25, under review) showed that nothing in consensus rested on a conjecture, by habit — this makes it a rule.

## against

eidos is small today and its record is not yet trustworthy (a Rust kernel, the nox kernel a draft spec, an FNV-1a certificate placeholder, `sorry` accepted as proved, one-file check, unary `Nat`, no `Int`, 139 tests); the ladder E0–E5 is work, and E2 is weeks; the registry's class-G and class-H rows are choices, not science, listed in the same table.

## review

node 5 of [[deliver]]: a second body, lenses charter alignment · technical truth · consistency · completeness · vocabulary and links · feasibility. verdicts and findings are on the pull request; each finding keeps its id until disposed.

### round 1 — disposed 2026-10-30

26 findings (F1–F26) checked against the repositories and disposed in four commits on `proposal/assumptions-registry` (`0c2b370` why now and sources, `0b39256` the registry, `09f640f` eidos and the ladder, `bcc167b` watch, launch and decisions). The observed results moved to [[soft3/audit/assumptions/2026-10-30|the receipt of 2026-10-30]].

- facts and dates. F1: campaign day 0 is not started; no invented dates. F2: pool digests are 64 bytes. F3: the sample entry is B-01. F4: section citations fixed. F5: A-02 follows the ledger over Fp3. F6: B-02 is the G3 criterion. F19, F20: sourced; the 3SUM paper's own text attributes the algorithm to Claude. F22: F-05 quotes the receipt; decision order fixed. F24: certificate per spec, README drift flagged.
- registry content. F7: E-01 covers every phase-1 signature and is red. F8: C-04 to C-06 and E-02 added; the privacy-frozen §1.1 contradiction flagged. F21: A′ sub-class. F23: A-09 added. Also found during disposal: C-03 MCA is proven at the shipped regime per the ledger; A-10 lists eidos's stdlib bootstrap set; chain.md's 66-minute recovery figure contradicts the measured 8,613 s.
- eidos. F9, F10, F11, F13, F14: stated as it is, with "checked" and "certified" separated, `sorry` as an error, the model/code gap linked to conformance, and the CLI work plus the eidos pin as gate prerequisites. F12 (blocker): E1 moved after genesis and cut to a symbolic conservation lemma, prerequisites listed.
- mechanisms. F15: keyword scan, cited constants, staleness informs and never reds, the watch run by the crypto domain's sensor. F16: one source; `Axioms.ei` and the mirror are generated; observations go to `audit/assumptions/`. F26: axioms for B and C only, F/G/H never; "same statement" is the particle of a canonical form.
- launch and decisions. F17: H-01 is due before 2026-11-02 00:00 UTC; what happens after the freeze is stated. F18: every decision has a link or "no link yet"; the Lean-evidence item is dropped; E5's bounty rule is an owner item. F25: `[[lean]]` is plain text; bold only on the status line; §4.1 phrased positively.

disagreements, recorded here. F8: the relation compiler is a correctness claim about our own compiler, not an oracle model, and program binding reduces to B-01. Both are listed in C, as asked, with that note, and C is widened to "proof-system conjectures". F11: `Nat.min` is a definition that reduces, not an axiom; `Nat.min_le_right` is the trusted one. F21: A-04 and A-08 are tested invariants too, so they are A′ alongside A-03, A-05, A-06 and A-07.

## verdict

—
