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

**status:** proposed · **date:** 2026-10-30 · **owner's call:** "with a mnemonic, somewhere, in some way, give the agent itself the certainty it has access; the owner proves declarations of access rights for agents and machines with the machinery of soft3. this is the foundation of fast development." · review round 1 disposed 2026-10-30 ([soft3#475](https://github.com/cyberia-to/soft3/pull/475))

## 0. the decision

Access to anything cyberia runs — an account on a host, a repository, a bot, later a secret — is granted by a declaration the owner signs with the neuron derived from the owner's spell (mnemonic), checked by the function that checks every transaction signature on the chain (`mudra::neuron::verify_statement`, [[cybernode/specs/chain|chain]] §2.7), and enforced by the thing accessed. A declaration names an issuer, a subject (an agent or a person, each a neuron), an object with the account on it, the credential the render installs, a set of verbs, a validity interval, a delegation depth and the declaration it revokes. The declarations in force are the owner's source for the renders: a key on a host, a seat in a GitHub team, an entry on a bot's allow list are written from them by the apparatus and never by hand. They do not replace Ward: for a secret they are an input to Ward, which stays the sole permission authority ([[vault/specs/authorization|vault authorization]]). An agent reads its own declarations and knows what it may do before it tries.

## 1. why

Today access is given by hand and remembered nowhere: an ssh key copied to a host, a GitHub invitation, a bot made admin in a group, a token pasted into a config. The owner is the router of every grant, and an agent that needs a host or a repository waits for a person. Three grants are due now — Joy (the owner's agent on the development machine), the person the owner names, and Alex, the mayor of Cyber Valley — to the [apparatus](https://github.com/cyberia-to/apparatus) (private): hosts, the cockpit, the bot, the registers. Giving them by hand repeats the problem; giving them by declaration starts the mechanism the whole estate needs, built from what the chain already has: a statement signed by a neuron, verified against its id.

## 2. the declaration

A declaration is a fixed byte layout. Its signature is the 102-byte NSIG1 envelope ([[mudra/specs/neuron-auth|neuron-auth]]) over a statement that binds the layout to its domain and to the estate.

```
decl_bytes, v1 — every field present, in this order, integers little-endian
version     u8                     = 1
issuer      [32]                   the neuron that signs
parent      [32]                   the grant that lets the issuer issue this one; zeros when the issuer is the root
subject     [32]                   the neuron of an agent or a person
kind        u8                     1 host · 2 repo · 3 bot · 4 secret · 5 root
name        u16 len ‖ bytes        the object with its account:
                                   agent-a/user:caesar · cyberia-to/soft3 · cyber_ia_bot/cyber-valley · <SecretRef hex> · cyberia
credential  u16 len ‖ bytes        what the render installs, covered by the signature:
                                   ssh-ed25519 key (host) · GitHub login (repo) · Telegram user id (bot) · empty (secret, root)
verbs       u8 count ‖ u8 codes    strictly ascending: 1 read · 2 write · 3 ssh · 4 sudo · 5 admin · 6 use · 7 act-as
from        u64                    unix seconds, inclusive
until       u64                    unix seconds, exclusive
depth       u8                     how many further levels the subject may delegate; 0 = may not
revokes     [32]                   the particle this declaration revokes; zeros = none
nonce       u64                    unique per issuer
```

```
statement = hemera("cyberia/access/v1\0" ‖ estate_id ‖ decl_bytes)
particle  = hemera(decl_bytes)
envelope  = mudra::neuron::sign(issuer_key, issuer, statement)        102 bytes
estate_id = hemera("cyberia/estate/v1\0" ‖ "cyberia")                  a test estate uses its own name
```

Canonical means one byte form per value, as on the chain (chain.md §2: fixed order, little-endian, no optional fields). A decoder rejects and never repairs:

- a version other than 1, an unknown kind or verb code, a verb its kind does not admit (host: ssh, sudo · repo: read, write · bot: read, admin · secret: use · root: act-as);
- verbs not strictly ascending, duplicates included;
- a name outside its grammar: ASCII `[a-z0-9_.:@-]` in `/`-separated segments, 1–200 bytes; a host name is exactly `<host>/user:<account>`; only a repo or bot name may end in a whole segment `*`;
- a credential that does not parse for its kind: exactly one `ssh-ed25519` key whose base64 decodes to the 51-byte blob, no options, no comment; a login in GitHub's grammar; a Telegram id that is a u64 in decimal without leading zeros; empty for secret and root;
- `from ≥ until`; trailing or missing bytes.

The test is re-encoding: a declaration that does not re-encode to its own bytes is rejected. Golden vectors — bytes, statement, particle and envelope from a fixed test key for a root grant, a delegated grant, a replacement, a revocation and a root transition — ship with the codec, the negative vectors of §11 beside them.

The particle is the declaration's id — the chain's particle of a file's bytes (chain.md §3.1). The statement is what is signed. They differ on purpose: the statement binds the domain and the estate, so a declaration signed for a test estate verifies nowhere else, and no other NSIG1 statement — a transaction's, whose tag is `soft3/chain/sign/v1` — can be read as a grant.

Three shapes, nothing else:

- a grant: verbs, `revokes` zero;
- a replacement: verbs and `revokes` set; it repeats the kind and name of what it revokes and grants anew — a rotated key, a new interval, a new subject;
- a revocation: no verbs, `revokes` set; it repeats the subject, kind and name of what it revokes, with `from = 0`, `until = 2^64 − 1`, depth 0 and an empty credential, and takes effect as soon as it is in the set.

A declaration with neither verbs nor `revokes` is rejected. The nonce makes a re-grant after a revocation a new particle (§4, nonces and replay).

Rules, each a removal:

1. one root: the owner's spell derives the root neuron at the Cosmos path the old chains used (`Signer::from_spell`, chain.md §2.8) — the key that also claims the owner's legacy balances. No second root of trust; the cost of one key for both is a decision (§13).
2. delegation only attenuates ([[vault/specs/authorization|vault authorization]] § exact request): a delegate issues within its own object, verbs, interval and depth − 1, never wider; §4 gives the order.
3. a declaration binds the action, the object with its account, the interval, the estate and a nonce ([[mudra/specs/identity|identity]] § exact authorization statement): a grant for `ssh` on `host:agent-a/user:caesar` authorizes nothing else, nowhere else, at no other time.
4. the thing accessed enforces; the declaration is never the credential. The credential is the key, login or id the declaration pins, installed by a render (§6, §7).
5. an agent knows its rights by reading: `cy access who-am-i` (§9).

## 3. signing and verifying

The verifier is `mudra::neuron::verify_statement(subject, statement, evidence)` (mudra `src/neuron.rs`), the function `cybernode::wallet::verify` calls for every transaction and chain.md §2.7 names. A declaration verifies iff its bytes are canonical and `verify_statement(issuer, statement, envelope)` holds; nothing else is consulted.

The signer is a new method of `cybernode::wallet::Signer`, over `mudra::neuron::sign(key, subject, statement)`:

```rust
impl Signer {
    /// NSIG1 over the 32-byte statement of another codec (an access declaration, a head).
    pub fn sign_statement(&self, statement: [u8; 32]) -> Result<[u8; 102], Error>;
}
```

`Signer::sign` signs transactions only and the key is private to the struct, hence the method. It adds no signing identity: the envelope is the one neuron-auth defines.

Phase-0 gate: `cy access` links `cybernode::wallet`, and cybernode has no compile proof yet. C2 of [[soft3/proposals/cybernode|cybernode]] is open — its receipt, `cybernode/audit/move-2026-10-27.md`, says "the compile proof is not in this receipt": no sibling set on origin resolves the node's dependencies. Phase 0 starts when `cargo test` is green in cybernode on a pinned sibling set.

The owner's root, without exposing the spell. NSIG1 is an ADR-036 signature: the signer string is the Cosmos address of the key under the `neuron` HRP, the data `cyber:neuron:authority:v1:` ‖ statement (neuron-auth). Two paths produce it without the spell entering a process an agent can read:

- offline: the hardware wallet (or an air-gapped signer) holding the owner's key signs the ADR-036 document; `cy access grant --sign adr036` prints the document, takes back the 64-byte signature and the 33-byte public key, assembles the envelope and writes nothing unless it verifies. Whether the Ledger Cosmos app signs for a `neuron1…` signer is unverified; it is the first check of the order (§12).
- vault: `authorize_action(KeyRef, canonical intent, grant, request_id)` ([[vault/specs/secrets|typed secrets]] § operation sketches) under a Ward grant for this purpose alone. It is a sketch today — "not an implemented or frozen wire API" — and the vault README lists the production Ward adapter as implementation work.

The owner's first grants (§10) wait for one of the two; the default is the offline path, because vault's is not built. The codec, the evaluator and the renders wait only for the C2 gate: they are built and tested against a test root. `Signer::from_spell` never appears in `cy access` — no code path reads a spell from argv, the environment, a file or stdin — and CI greps the tool's source for it. Every signing ceremony ends with the non-exposure check of §11.

Phase 1 adds the chain's signature, not a new kind: the issuer re-signs each declaration as a kind-2 transaction

```
Tx::Cyberlink { from: issuer, nonce: the issuer's chain nonce, token: the native coin, amount: 0,
                p: particle, q: subject, valence: +1, fee: 257 }
```

257 is the floor — `Kind::Cyberlink.min_fee()`, 257 bytes at one unit per byte (chain.md §2.6) — so the issuer needs a funded balance on pussy; its chain nonce (chain.md §2.5) is separate from the declaration's. The transaction's statement is the chain's, `hemera("soft3/chain/sign/v1\0" ‖ chain ‖ tx)`, signed by the same path as above. A reader accepts the link iff `wallet::verify(chain, tx)` passes, `from == issuer`, `q == subject` and `hemera(decl_bytes) == p`. A link with amount 0 is topology only: bbg's `insert` writes no particle record (chain.md §3.2, the particles row), so the chain holds the signed transaction and `axon(p, q)`, never the declaration's bytes. The bytes stay in the apparatus mirror (§8) under their particle and may also be pinned on the particle gateway; a reader fetches them from either and trusts neither, because `hemera(bytes) == p` is the check.

## 4. subjects

| subject | neuron from | today |
|---|---|---|
| the owner | the owner's mnemonic (vault, native profile) | the issuer |
| Joy, the owner's agent on the development machine | a spell generated on the machine, in vault; the public neuron registered by the owner's declaration | ssh to the hosts by a copied key; GitHub as `cyberia-apparatus` |
| the director of soft3, Julia Caesar, kimi, chatgpt, grok (agent-a) | one spell per unix user on agent-a, in that user's vault | `estate.toml` names them; access by copied keys and tokens |
| machines: galapot, agent-a, cyberproxy, deimos, jupiter, io, mimas | a spell per machine, generated at base install | ssh host keys only |
| the person the owner names; Alex, the mayor of Cyber Valley | their own mnemonic (a cyb wallet, or a bostrom key through the bridge) | none |

A person and an agent are the same kind of subject; the difference is only who holds the mnemonic.

## 5. objects and their renders

| object | the render the apparatus writes | from the declaration |
|---|---|---|
| `host:<name>` with `ssh`, `sudo` | `~<user>/.ssh/authorized_keys`, `/etc/sudoers.d/<user>` | the subject's ssh public key is a field of its neuron's registration (a cyberlink neuron → key particle) |
| `repo:<org>/<name>` with `read`, `write`, `merge`, `admin` | GitHub team membership and branch-protection bypass, through the API as `cyberia-apparatus` | the subject's GitHub login is a field of its registration |
| `bot:<handle>` with `read`, `admin` | the bot's allow list and group admin bits | the subject's Telegram id |
| `secret:<SecretRef>` with `use` | a Ward grant in the vault that holds the secret ([[vault/specs/secrets|typed secrets]]) | the declaration is the Ward decision's input |
| `budget:<ledger>` with `spend:<limit>` | the cockpit's money tab, the Akash deposit, the Hetzner card | the counter checks every outflow against the limit |
| `role:<name>` with `act-as` | `estate.toml` `[[agent]]` roles; the plan's lanes | a role is a bundle of object grants, declared once |

## 6. where the declarations live

Phase 0 (now): `apparatus/grants/<particle>.decl` — the canonical bytes and the 102-byte envelope, one file per declaration, committed by the issuer's agent; `apparatus/grants/index.toml` generated. The apparatus repository is private, which is enough for the subjects' public neurons and the objects' names; nothing secret is in a declaration.

Phase 1 (with pussy): the same bytes as cyberlinks on the chain, amount 0: the declaration particle → the subject's neuron, signed by the issuer. The apparatus reads the chain. Revocation is a later link. The private repository keeps a mirror for the hosts that must not depend on a node to boot.

Phase 2 (fathership phase 2): declarations are the only record; GitHub teams, keys and bot lists are renders, re-generated from the graph on every tick, and a hand-made grant is removed by the next tick.

## 7. the tool

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

## 8. the first three grants

| subject | objects | scope | until |
|---|---|---|---|
| Joy | `host:agent-a`, `host:cyberproxy`, `host:deimos`, `host:jupiter`; `repo:cyberia-to/*`; `bot:@cyber_ia_bot`; `budget:akash` | ssh, sudo; write, merge on non-default branches; read; spend ≤ the lease | 90 days, depth 1 (may grant a sub-agent for one task) |
| the person the owner names | `repo:cyberia-to/apparatus`; the cockpit; `host:<as the owner says>` | read; or as the owner says | as the owner says |
| Alex, the mayor | `bot:@cyber_ia_bot` (the Cyber Valley topic); `repo:cyberia-to/cve` | read, write on `cve/`; the mayor's line to the governor | the term of the president director |

The owner issues these three by hand once with `cy access grant`; everything after is the mechanism.

## 9. acceptance

- a declaration signed on the development machine verifies on agent-a with `cy access verify` using only `cybernode::wallet` code; the same bytes verify as a cyberlink statement on the pussy chain's verifier.
- `cy access render` on agent-a produces `authorized_keys` for the five agent users from declarations alone; a key added by hand disappears on the next tick, with a cockpit alert naming it.
- Joy opens an ssh session to agent-a and a pull request to apparatus with no owner action after the owner's one grant.
- a revocation takes effect within one tick on every host and on GitHub.
- the cockpit shows the declarations in force per subject and per object; an agent without a grant for an object raises a decision with the link to the declaration it needs.

## 10. decisions for the owner

1. the subject neurons: Joy's spell lives in the development machine's vault; the agent-a bodies each in their own. default: yes.
2. the person: who, and which objects. no link yet; the owner names them.
3. phase 1 on the chain: declarations as amount-0 cyberlinks on pussy from genesis, or private files until the chain has run a month. default: files now, the chain after the first month.
4. GitHub: the renders act as `cyberia-apparatus`, which must then hold org admin. default: yes, with the owner's own account untouched.
