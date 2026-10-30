---
tags: soft3, task, proposal
alias: p2 access declarations, p2
kind: review
status: open
owner: master
serves: fathership · the foundation of fast development
lane: contained
---
# p2 · access declarations

## claim

the owner (2026-10-30): give Joy, the person the owner names, and Alex the mayor access to the apparatus by declarations the owner proves with the machinery of soft3 — a statement signed from the owner's mnemonic that an agent verifies itself. the unit is one proposal page, `proposals/access-declarations.md`, and its index row. the pull request: [soft3#475](https://github.com/cyberia-to/soft3/pull/475); the apparatus's side: `apparatus/tasks/a1-access-declarations.md`.

## for

the signature the chain trusts (NSIG1, checked by `mudra::neuron::verify_statement`) grants host accounts, repositories and the bot; renders are written by the apparatus from the declarations in force, never by hand, and fail closed; an agent reads its own rights before it acts; delegation only attenuates; revocation is a declaration, permanent and cascading; Ward stays the authority over secrets.

## against

a private file store of declarations is a second record beside the chain until phase 1, kept honest only by the signed head; phase 0 waits for the cybernode C2 compile proof and for an offline root signing path; per-subject repository scope waits for a GitHub identity per subject; the root shares its key with the owner's balances and has no recovery quorum yet; the subjects' keys in custody are new surface.

## review

node 5 of [[deliver]]: a second body, lenses charter alignment · technical truth · consistency · completeness · vocabulary and links · security. verdicts and findings are on the pull request; each finding keeps its id until disposed.

### round 1 — disposed 2026-10-30

each finding checked against the code and the specs before the edit: cybernode `rs/src/wallet/{mod,tx}.rs` and `specs/chain.md` §2–3; mudra `src/neuron.rs`, `specs/{neuron-auth,identity}.md`; vault `specs/{authorization,secrets}.md`, README, `src/custody/uses.rs`; apparatus `estate.toml`, the cockpit's launchd period; cyberia `architecture.md`, `dev.md`, `people/README.md`; the home `AGENTS.md`.

| id | finding | disposed | commit |
|---|---|---|---|
| F1 | the signing path | `Signer::sign_statement` over `mudra::neuron::sign` (Signer signs only txs today); verifier `mudra::neuron::verify_statement`, the one `wallet::verify` calls; phase 1 a kind-2 `Tx::Cyberlink{p = particle, q = subject}`, amount 0, fee floor 257, a funded issuer; the C2 compile proof (cybernode `audit/move-2026-10-27.md`) a phase-0 gate — §3 | 9db9730 |
| F2 | canonical layout | fixed v1 layout with length-prefixed name and credential, strictly ascending verbs, mandatory 32-byte `revokes`, a `parent`; statement `hemera("cyberia/access/v1\0" ‖ estate_id ‖ bytes)`, particle `hemera(bytes)`; reject by re-encoding; golden vectors; bytes in the mirror because an amount-0 link writes no particle record (chain.md §3.2) — §2, §3 | 9db9730 |
| F3 | the owner's root path | offline ADR-036 (NSIG1 is ADR-036 under the `neuron` HRP) or vault `authorize_action` (a sketch, not built); the first grants wait, the default offline; `from_spell` forbidden in `cy access`, CI grep; non-exposure grep in acceptance — §3, §11 | 9db9730 |
| F4 | trust anchor, evaluator | ROOT, ESTATE_ID and the protected set pinned in the renderer; pure `in_force(set, now)`, `index.toml` ignored; revocation by issuer or ancestor, permanent, cascading; roles only by signed declarations (no role kind in v1) — §4 | acd50c7 |
| F5 | registrations | the `credential` field inside the issuer's declaration; the object names the account, `host:agent-a/user:caesar` — §2, §5 | 777c259 |
| F6 | renderer failure modes | fail closed on empty or unverifiable input; a 24 h lease; break-glass outside the render; monotone head plus a root-signed head; fixed templates, strict parsing; 15-minute tick (the cockpit's `StartInterval` 900); stated maximum delay; sessions ended, secrets listed for rotation (vault authorization lines 79–88) — §7 | 777c259 |
| F7 | GitHub | per-subject repo scope dropped for subjects sharing `cyberia-apparatus`; a GitHub identity per subject is decision 5; the renderer a GitHub App with Members only, not org admin — §6 | 777c259 |
| F8 | Joy's grant | depth 0; the protected set (apparatus, cyberia, .github) never matched by `*`; ssh only; a revocation drill in acceptance — §4, §10, §11 | 0641e06 |
| F9 | merge and bypass | no `merge` verb, no bypass render; the train rule cited (home `AGENTS.md` § release train); unprotected default branches alerted — §6 | 777c259 |
| F10 | sudo and budget | both named as the irreversible lane, each the owner's verdict per grant; budgets and `spend` dropped from phase 0 — §6 | 777c259 |
| F11 | Ward | Ward the sole authority; declarations its input; the apparatus writes no Ward grants; §0 narrowed to "the owner's source for the renders" — §0, §6 | 9db9730, 777c259 |
| F12 | subject/key, rotation | NSIG1 makes the key the subject: rotation by replacement; root transition signed by the old root; recovery quorum named as open, decision 1 — §4 | acd50c7 |
| F13 | Joy | a body (cyberia `dev.md`:93) and a genesis neuron (`architecture.md`:51); the `[[agent]]` row an a1 follow-up; the `[[joy]]` link (prysm's emotion page) now plain Joy — §5 | 777c259 |
| F14 | acceptance | the negative vectors, the tick, the non-exposure grep, the drill — §11 | 0641e06 |
| F15 | order | repo · move · gate per phase (§12); nonces and replay and the scope lattice (§4); the machine row cut (§5) | acd50c7, 777c259, 0641e06 |
| F16 | decisions | each with its section link and a1; root and rotation, sudo and depth, Joy's spend, the time source added — §13 | 0641e06 |
| F17 | format | apparatus a link, plain Joy, the spell named once, a relative date, no bold but the status line, a `related:` footer | 0641e06 |

next: round 2 by the reviewing body; the apparatus side, [a1](https://github.com/cyberia-to/apparatus/blob/master/tasks/a1-access-declarations.md), still says the renders write Ward grants and act as `cyberia-apparatus` with org admin, and is to follow this page.

## verdict

—

### round 2 · 2026-10-30

the reviewer re-checked F1–F20: all disposed; four deviations accepted (the C2 gate narrowed to mudra + hemera at the pinned tier, no role kind in v1, the root-signed head and the 24-hour lease, two signatures over two statements). new findings N1–N9: N1 (the hemera tier pinned in §2 and the gate reworded) and N2 (agent-a has no systemd: scheduler, session end and clock named) disposed in this PR; N8's name clash (`sign_digest`, "not in code") disposed; N3–N7, N9 carried as follow-ups on this page: a maximum age for a signed head, the lease on a monotonic timer with an unsynchronized-clock bound, a root-owned `AuthorizedKeysFile`, the root transition inside the `in_force` steps, the `*` in the name grammar and the missing vectors, the App's permission set, Actions secrets under `write`. verdict: approve on condition of N1 and N2.
