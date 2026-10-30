---
title: access declarations
tags: cyber, soft3, mudra, vault, apparatus, proposal, security
crystal-type: proposal
crystal-domain: cyber
alias: access declarations, grants, declarations of access rights, who may do what
status: proposed
date: 2026-10-30
---
# access declarations

**status:** proposed · **date:** 2026-10-30 · **owner's call:** "with a mnemonic, somewhere, in some way, give the agent itself the certainty it has access; the owner proves declarations of access rights for agents and machines with the machinery of soft3. this is the foundation of fast development."

## 0. the decision

Access to anything cyberia runs — a host, a repository, a bot, a secret, a budget — is granted by a declaration the owner signs with the neuron derived from the owner's mnemonic, verified by the same code that verifies a cyberlink on the chain, and enforced by the thing accessed. A declaration names a subject (an agent, a machine, a person — each a neuron), an object, a scope, a validity interval and an issuer. The set of declarations in force is the only source of truth about who may do what; a key on a host, a seat in a GitHub team, an admin bit on a bot are renders of it, written by the apparatus from the declarations and never by hand. An agent reads its own declarations and knows what it may do before it tries.

## 1. why

Today access is given by hand and remembered nowhere: an ssh key copied to a host, a GitHub invitation, a bot made admin in a group, a token pasted into a config. The owner is the router of every grant, and an agent that needs a host or a repository waits for a person. Three grants are due now — [[joy]] (the owner's agent on the development machine), the person the owner names, and the mayor of Cyber Valley — to the [[apparatus]]: hosts, the cockpit, the bot, the registers. Giving them by hand repeats the problem; giving them by declaration starts the mechanism the whole estate needs, and it is the same mechanism the chain already has: a statement signed by a neuron, verified against its id.

## 2. the declaration

A declaration is a cyberlink's statement signed under NSIG1 ([[mudra/specs/neuron-auth|neuron-auth]]): the issuer's neuron is `neuron_of(pubkey)`, the statement is the 32-byte digest of the canonical declaration bytes, the envelope is the 102 bytes the chain's wallet already produces (`cybernode::wallet::Signer`, `wallet::tx::statement`). Nothing new is signed; a grant is a cyberlink whose particle is the declaration and whose target is the subject's neuron.

```
declaration/v1
issuer:    neuron          — the owner's, or a delegate's within its own grant
subject:   neuron          — an agent, a machine, a person
object:    kind ‖ name     — host:agent-a · repo:cyberia-to/apparatus · bot:@cyber_ia_bot ·
                             secret:<SecretRef> · budget:<ledger> · role:<name>
scope:     verbs           — ssh · sudo · read · write · merge · admin · spend:<limit> · act-as:<role>
valid:     [from, until]   — unix seconds; until ≤ issuer's own until for a delegate
depth:     n               — how many times the subject may delegate (0 = may not)
revokes:   particle | —    — the declaration this one replaces
nonce:     u64
```

Canonical encoding is the chain's (`chain-wire.md` integer rules, little-endian, fixed order), so the same bytes hash to the same particle on every machine. The particle of a declaration is its id; a revocation is a declaration naming it in `revokes` with an empty scope.

Rules, each a removal:

1. one key: the owner's mnemonic derives the owner's neuron through mudra's spell; the same derivation that claims the owner's legacy balances on the chain. no second root of trust.
2. delegation only attenuates ([[vault/specs/authorization|vault authorization]]): a delegate may issue within its own scope, interval and depth − 1, never wider.
3. a declaration binds the action, the object, the interval and a nonce ([[mudra/specs/identity|identity]] § exact authorization statement): a grant for `ssh` on `host:agent-a` authorizes nothing else, nowhere else, at no other time.
4. the thing accessed enforces; the declaration is never the credential. the host's `authorized_keys`, the GitHub team, the bot's admin list and the vault's Ward grants are written from the declarations in force by the apparatus, on every tick, and a key not backed by a declaration is removed.
5. an agent knows its rights by reading: `cy access who-am-i` lists the declarations in force for the caller's neuron; an agent that lacks a grant asks for it as a decision, with the link to the declaration it needs.

## 3. subjects

| subject | neuron from | today |
|---|---|---|
| the owner | the owner's mnemonic (vault, native profile) | the issuer |
| Joy, the owner's agent on the development machine | a spell generated on the machine, in vault; the public neuron registered by the owner's declaration | ssh to the hosts by a copied key; GitHub as `cyberia-apparatus` |
| the director of soft3, Julia Caesar, kimi, chatgpt, grok (agent-a) | one spell per unix user on agent-a, in that user's vault | `estate.toml` names them; access by copied keys and tokens |
| machines: galapot, agent-a, cyberproxy, deimos, jupiter, io, mimas | a spell per machine, generated at base install | ssh host keys only |
| the person the owner names; Alex, the mayor of Cyber Valley | their own mnemonic (a cyb wallet, or a bostrom key through the bridge) | none |

A person and an agent are the same kind of subject; the difference is only who holds the mnemonic.

## 4. objects and their renders

| object | the render the apparatus writes | from the declaration |
|---|---|---|
| `host:<name>` with `ssh`, `sudo` | `~<user>/.ssh/authorized_keys`, `/etc/sudoers.d/<user>` | the subject's ssh public key is a field of its neuron's registration (a cyberlink neuron → key particle) |
| `repo:<org>/<name>` with `read`, `write`, `merge`, `admin` | GitHub team membership and branch-protection bypass, through the API as `cyberia-apparatus` | the subject's GitHub login is a field of its registration |
| `bot:<handle>` with `read`, `admin` | the bot's allow list and group admin bits | the subject's Telegram id |
| `secret:<SecretRef>` with `use` | a Ward grant in the vault that holds the secret ([[vault/specs/secrets|typed secrets]]) | the declaration is the Ward decision's input |
| `budget:<ledger>` with `spend:<limit>` | the cockpit's money tab, the Akash deposit, the Hetzner card | the counter checks every outflow against the limit |
| `role:<name>` with `act-as` | `estate.toml` `[[agent]]` roles; the plan's lanes | a role is a bundle of object grants, declared once |

## 5. where the declarations live

Phase 0 (now): `apparatus/grants/<particle>.decl` — the canonical bytes and the 102-byte envelope, one file per declaration, committed by the issuer's agent; `apparatus/grants/index.toml` generated. The apparatus repository is private, which is enough for the subjects' public neurons and the objects' names; nothing secret is in a declaration.

Phase 1 (with pussy): the same bytes as cyberlinks on the chain, amount 0: the declaration particle → the subject's neuron, signed by the issuer. The apparatus reads the chain. Revocation is a later link. The private repository keeps a mirror for the hosts that must not depend on a node to boot.

Phase 2 (fathership phase 2): declarations are the only record; GitHub teams, keys and bot lists are renders, re-generated from the graph on every tick, and a hand-made grant is removed by the next tick.

## 6. the tool

`cy access` in the `cy` toolset ([[cyb]] `cli/tools.toml`), over `cybernode::wallet` and the vault:

```
cy access grant   --to <neuron> --object host:agent-a --scope ssh,sudo --until 2027-01-01 --depth 0
cy access revoke  <particle>
cy access list    [--subject <neuron>] [--object <kind:name>]
cy access who-am-i
cy access render  --host agent-a      # writes authorized_keys / sudoers from the declarations in force
cy access verify  <file.decl>         # the chain's verifier, nothing else
```

`grant` signs with the caller's neuron from its vault; the owner's mnemonic never leaves the vault, and signing is a Ward-authorized operation with a receipt. `render` runs from the apparatus tick on every host and from the cockpit for GitHub and the bot.

## 7. the first three grants

| subject | objects | scope | until |
|---|---|---|---|
| Joy | `host:agent-a`, `host:cyberproxy`, `host:deimos`, `host:jupiter`; `repo:cyberia-to/*`; `bot:@cyber_ia_bot`; `budget:akash` | ssh, sudo; write, merge on non-default branches; read; spend ≤ the lease | 90 days, depth 1 (may grant a sub-agent for one task) |
| the person the owner names | `repo:cyberia-to/apparatus`; the cockpit; `host:<as the owner says>` | read; or as the owner says | as the owner says |
| Alex, the mayor | `bot:@cyber_ia_bot` (the Cyber Valley topic); `repo:cyberia-to/cve` | read, write on `cve/`; the mayor's line to the governor | the term of the president director |

The owner issues these three by hand once with `cy access grant`; everything after is the mechanism.

## 8. acceptance

- a declaration signed on the development machine verifies on agent-a with `cy access verify` using only `cybernode::wallet` code; the same bytes verify as a cyberlink statement on the pussy chain's verifier.
- `cy access render` on agent-a produces `authorized_keys` for the five agent users from declarations alone; a key added by hand disappears on the next tick, with a cockpit alert naming it.
- Joy opens an ssh session to agent-a and a pull request to apparatus with no owner action after the owner's one grant.
- a revocation takes effect within one tick on every host and on GitHub.
- the cockpit shows the declarations in force per subject and per object; an agent without a grant for an object raises a decision with the link to the declaration it needs.

## 9. decisions for the owner

1. the subject neurons: Joy's spell lives in the development machine's vault; the agent-a bodies each in their own. default: yes.
2. the person: who, and which objects. no link yet; the owner names them.
3. phase 1 on the chain: declarations as amount-0 cyberlinks on pussy from genesis, or private files until the chain has run a month. default: files now, the chain after the first month.
4. GitHub: the renders act as `cyberia-apparatus`, which must then hold org admin. default: yes, with the owner's own account untouched.
