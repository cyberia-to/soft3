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

## 4. the trust anchor and the evaluator

The root is pinned where the declarations cannot reach it: `ROOT` (the owner's neuron), `ESTATE_ID` and the protected set (below) are constants compiled into the renderer and, on a host that runs vault, values of its Ward policy. They are never read from the apparatus repository, which is the input being judged: whoever can write the repository can add files, not move the root.

What is in force is a pure function, recomputed on every tick from the `.decl` files alone:

```
in_force(set, now) -> sorted [(subject, kind, name, credential, verbs, until, particle)]
```

`grants/index.toml` is a view for people; the evaluator never reads it. Inside `in_force`:

1. decode every file; drop, and report, a file whose name is not its particle, whose bytes are not canonical, or whose envelope does not verify for its issuer under `ESTATE_ID`.
2. equivocation: two different particles with one (issuer, nonce) are both dropped.
3. rooted: a grant is rooted iff its issuer is `ROOT` and `parent` is zero, or `parent` is a rooted grant in the set whose subject is the issuer, whose depth is ≥ 1 and which contains it in the order below.
4. revocation: a revocation or replacement of X takes effect iff its issuer is X's issuer or an issuer on X's chain of parents — the root always is. It is permanent: a revocation has no interval that ends, a revocation cannot itself be revoked, and a revoked particle stays revoked; a re-grant is a new declaration with a new nonce. It cascades: every grant rooted through X falls with X.
5. time: a grant is in force iff `from ≤ now < until`.

The order that "contains" uses — the scope lattice:

- object: the same kind, and the same name, or the parent's name ends in the segment `*` and the child's name begins with the parent's segments before it. `*` never matches the protected set, pinned beside the root: the apparatus repository and the charter's repositories, `cyberia-to/cyberia` and `cyberia-to/.github`. A grant on one of them names it exactly and is issued by the root.
- verbs: set inclusion. No verb implies another: `sudo` does not imply `ssh`, `admin` does not imply `read`; a grant names each.
- interval: the child's `[from, until)` inside the parent's.
- depth: the child's ≤ the parent's − 1.
- spend is not in v1 (§6). When it returns it carries a unit and a limit, and a child is contained iff the unit is the same and the limit ≤ the parent's; the children's limits are counted against the parent's by the one Ward counter, not added up by the evaluator.

Roles grant only by signed declarations. `estate.toml` roles and the plan's lanes describe; they grant nothing. v1 carries no role kind: each first grant is direct, and a role bundle comes with a later version when one is needed, with the credential question (a bundle spans kinds, a credential is per kind) answered there.

Nonces and replay. identity.md: "replay and double-spend prevention use authenticated nonce/nullifier state". A declaration is a standing grant, not a one-time action, so replaying its bytes is harmless — the same particle, the same grant. The replay that matters is resurrection: a revoked declaration re-entering a set it was dropped from, or a declaration signed long ago and held back. The authenticated state against both is the signed head of §7: the set only grows, a revoked particle stays revoked, and a new particle from an issuer must carry a nonce above that issuer's highest in the last signed head. On the chain the issuer's account nonce orders its links (chain.md §2.5), and the declaration's nonce remains only what makes particles distinct.

Time. `now` is the renderer host's clock, and the renderer changes nothing unless that clock is synchronized (systemd's `NTPSynchronized=yes`; on galapot, the system's time sync). identity.md is plain that "wall-clock assertions from an untrusted host do not satisfy a chain timelock": from phase 1 the time is the chain's median time past (chain.md §1.4). A decision (§13).

Subjects, keys and rotation. identity.md separates a stable subject from a rotatable authority key; NSIG1 does not — the neuron is `hemera(pubkey)` and "no key registry exists" (chain.md §2.7). In this profile a new key is a new neuron, so:

- a subject's key rotates by replacing each of its grants with one to the new neuron, in one ceremony;
- a credential (ssh key, login, Telegram id) rotates by a replacement that changes only the credential;
- the root rotates by a root transition: kind `root`, name `cyberia`, subject the new root, verb `act-as`, depth 0, parent zero, signed by the current root. From its `from` the renderer treats the new root as root; the old root's grants stay in force until they expire or the new root revokes them (it is an ancestor of all of them); the pinned constant moves in the renderer's next release, and the transition stays in the set for good.

A transition is signed by the current root, so a lost or stolen root cannot be recovered by one. That needs a recovery quorum — [[mudra/specs/identity|identity]] § programmable policies: "authorized recovery quorum and policy transition"; [[mudra/specs/quorum|quorum]] — which this proposal does not specify. Until it is specified, a compromised root is answered out of band only: the owner's break-glass key (§7) on every host, and a renderer release with a new `ROOT`. A decision (§13).

## 5. subjects

| subject | neuron | today |
|---|---|---|
| the owner | the owner's spell at the Cosmos path (§2 rule 1), signing offline (§3) | the root; the only issuer in phase 0 |
| Joy, the owner's agent on the development machine | derived in custody on galapot | ssh to the hosts by a copied key; no GitHub login of its own |
| the agent-a bodies: the director of soft3 (`claude`), Julia Caesar (`caesar`), kimi, chatgpt, grok | one per unix user, derived in custody on agent-a | `estate.toml` names them; copied keys and tokens; every one acts on GitHub as `cyberia-apparatus` |
| the person the owner names; Alex, the mayor of Cyber Valley | their own spell (a cyb wallet, or a bostrom key through the bridge) | none |

A person and an agent are the same kind of subject; the difference is only who holds the spell.

Joy is a body (cyberia `dev.md`:93, "bodies today: … master · joy") and a genesis neuron ([[cyberia/architecture|architecture]]:51). This proposal grants the body that works for the owner on galapot and settles nothing about the genesis neuron's authority. Joy has no `[[agent]]` row in apparatus `estate.toml`; that row, with the neuron field every agent row gains, is a follow-up of [apparatus a1](https://github.com/cyberia-to/apparatus/blob/master/tasks/a1-access-declarations.md).

Custody. A subject's key is derived inside custody — vault's `derive_neuron`, implemented in the library (vault `src/custody/uses.rs`). In phase 0 a subject signs nothing: every first grant has depth 0 and the root is the only issuer. A subject's key is first used to delegate (depth ≥ 1) or to sign its phase-1 links, through vault's `authorize_action` (§3).

Machines are objects, not subjects. No first grant names a machine as a subject, and a neuron per machine would be new custody with no use, so the row is cut. A machine gets a neuron when a declaration first needs one — a host-to-host deploy, say — and not before.

Registrations. What a render installs is pinned inside the declaration that grants it — the `credential` field (§2), covered by the issuer's signature: the subject's ssh-ed25519 key for a host, its GitHub login for a repository, its Telegram user id for the bot. Nothing is read from a registration the subject could change on its own. The object carries the account it reaches — `host:agent-a/user:caesar`, never `host:agent-a` — so a grant opens exactly one account.

## 6. objects and their renders

| object | verbs | the render | notes |
|---|---|---|---|
| `host:<host>/user:<account>` | ssh, sudo | the managed `~<account>/.ssh/authorized_keys`; `/etc/sudoers.d/access-<account>` | sudo on a production host is the owner's verdict, per grant |
| `repo:<org>/<name>` | read, write | membership of the GitHub teams `access-<name>-read` and `access-<name>-write` | per subject only with a login per subject; no merge, no bypass |
| `bot:<handle>/<topic>` | read, admin | the bot's allow-list file, read by the bot | the bot waits for its token (apparatus c2, c3) |
| `secret:<SecretRef>` | use | none: an input to Ward | not in phase 0 |
| `root:cyberia` | act-as | none: moves the root (§4) | |

Ward stays the sole permission authority. Vault authorization: "Ward is the sole permission authority configured by Soul … It MUST NOT mint a parallel set of grants". For a `secret:` object the declarations in force are a fact Ward's policy may read — the root grants subject S use of R until T, particle P — and Ward decides; the apparatus writes no Ward grants. Phase 0 renders no secret object: no production Ward adapter exists yet (vault README, stage). A secret reaches an agent today by the hand that put it there; that is named here, not solved.

The irreversible lane. The home `AGENTS.md` names it — "deleting data, keys, payments, the production chain" — and the owner holds it. Two grants are that lane's authority, so each is the owner's verdict, given per grant, never through a delegate:

- standing `sudo` on a production host (agent-a, cyberproxy, deimos, jupiter, io, mimas): root on a host deletes its data and reads its keys;
- spending from a budget (the Akash deposit, the Hetzner card, the Filecoin wallets): a payment. Budget objects and the `spend` verb are dropped from phase 0. A spend grant means something only as a numeric limit with a unit that Ward checks at every outflow, and no Ward adapter runs; the cockpit's money tab shows, it enforces nothing.

GitHub. A team seat belongs to a login, and every agent-a body acts as `cyberia-apparatus` (`estate.toml`: `github = "cyberia-apparatus"` on each agent-a row), so a repository grant to one of them would be a grant to all of them. Per-subject repository scope is therefore dropped from phase 0 for subjects that share a login: a subject gets a repository grant only with its own GitHub identity, a machine user or an app per subject — a decision (§13). The renderer does not act as `cyberia-apparatus` and holds no org admin: it is a GitHub App installed on cyberia-to with one organization permission, Members (read and write) — the teams API, no contents, no administration, no branch protection.

No merge, no bypass. Merging follows the release train (the home `AGENTS.md` § release train): "Other agents do not merge"; the product director merges into release branches, and into a default branch only through a pull request the owner approves, enforced by branch protection with the owner exempt. A declaration cannot widen that, so v1 has no `merge` verb and renders no branch-protection bypass. `write` renders as GitHub's push permission; on a repository whose default branch is unprotected that permission can merge, so the renderer lists every such repository in a write grant as a cockpit alert.

## 7. the renderer

`cy access render --host <h>` runs on each host from the apparatus tick (a systemd timer); `cy access render --github` and `--bot` run where the cockpit runs (launchd, beside it). One tick is 15 minutes, the cockpit's period (`to.cyberia.cockpit`, `StartInterval` 900).

A render is the difference between `in_force(set, now)` and what the managed files and team lists hold: it writes only that difference and reports every line it removes. It fails closed:

- empty or unverifiable input — no `.decl` files, a failed fetch, a set in which nothing verifies, a set that does not contain the last signed head — changes nothing and raises a cockpit alert. Deleting the grants directory revokes nothing; revocation is by adding.
- a lease bounds that "nothing": if no input has verified for 24 hours (96 ticks), the managed files are emptied — a profile must "refuse operation when it cannot establish lease validity" (vault authorization, lines 79–88). Expiry needs no input: a grant whose `until` passes leaves on the next tick, computed from the last verified set.
- an unsynchronized clock (§4) changes nothing.

Monotone. The renderer keeps a head: a sequence number and the sorted particle set of the last input it applied. An input that lacks a particle of the head is rejected; no declaration ever leaves the set. At the end of every ceremony the root also signs a head, `statement = hemera("cyberia/access/head/v1\0" ‖ estate_id ‖ seq ‖ the set's particles, sorted)`, kept as `grants/head` with its list and envelope. A renderer rejects a signed head whose sequence is below the highest it has seen and a set that lacks any particle of that head; this is what keeps a fresh host, with no local head, from being handed a set without a revocation. In phase 1 the chain is the append-only log and its order replaces the signed head.

Strict. A render writes from templates with a fixed option set; a declaration contributes tokens, never syntax:

- `authorized_keys`: one line per grant, `restrict,pty ssh-ed25519 <base64> access:<particle>`, the key re-serialized from its parsed bytes;
- sudoers: exactly `<account> ALL=(ALL:ALL) NOPASSWD: ALL` for `sudo`, no command lists, checked with `visudo -cf` before it is moved in;
- an account must already exist on the host and match `^[a-z_][a-z0-9_-]{0,31}$`; the renderer never creates or deletes unix users;
- team names, logins and Telegram ids are checked against their grammar before any call.

Break-glass. The owner's key sits outside every render: sshd's `AuthorizedKeysFile` names a root-owned owner file besides the managed one, and the renderer never opens it. On GitHub the owner's account is an org owner, which a Members-only app cannot touch.

What a removed line does not reach. Removing a key ends no open session, and revocation "cannot retract a signature, password or code already delivered" (vault authorization, lines 79–88). So when a grant leaves the set the render also:

- ends the account's sessions (`loginctl terminate-user <account>`) when no grant to that account is left;
- removes the login from the teams; open pull requests stay, as the record;
- lists in the cockpit every secret the account could read on that host — its env files, tokens in its home, the bot's token — for rotation by the secret's owner. A revocation is complete when those are rotated, and the cockpit shows it open until then.

Delay. The maximum revocation delay, which vault authorization asks every profile to report: one tick after the revocation reaches origin — 15 minutes plus the fetch — for a host that reaches origin; the 24-hour lease for one that does not.

## 8. where the declarations live

Phase 0 (now): `apparatus/grants/<particle hex>.decl` — `decl_bytes ‖ envelope`, one file per declaration — and `apparatus/grants/head`, committed by whoever ran the ceremony; `grants/index.toml` is generated for people and ignored by the evaluator (§4). The apparatus repository is private; nothing secret is in a declaration — public neurons, account names, public keys, logins, ids. The repository is a channel, not a trust: whoever can write it can only add files the evaluator drops.

Phase 1 (with pussy): each declaration also a kind-2 link, amount 0, `p` its particle, `q` its subject (§3). The renderer reads the links from the chain and the bytes from the mirror, by particle. A revocation is a later link. The private repository keeps the bytes and serves the hosts that must not depend on a node to boot.

Phase 2 (fathership phase 2): the declarations are the only record the renders are written from; GitHub teams, keys and bot lists are re-generated from the graph on every tick, and a hand-made grant is removed by the next one.

## 9. the tool

`cy access`, a tool of the `cy` toolset (cyb `cli/tools.toml`), over `cybernode::wallet` (§3):

```
cy access grant    --to <neuron> --object host:agent-a/user:joy --credential "ssh-ed25519 AAAA…" \
                   --verbs ssh --until +90d --depth 0 --sign adr036
cy access revoke   <particle> --sign adr036
cy access head     --sign adr036                 # the signed head over the set, at the end of a ceremony
cy access list     [--subject <neuron>] [--object <kind:name>]
cy access who-am-i [--neuron <neuron>]
cy access render   --host <name> | --github | --bot
cy access verify   <file.decl>                   # canonical bytes and verify_statement, nothing else
```

`--until` takes a period and writes the absolute time into the bytes. `--sign` is `adr036`, the offline path, or `vault` once `authorize_action` exists; there is no `--spell` and no code path that reads one. `who-am-i` lists the grants in force whose subject is the caller's neuron or whose object names the caller's account on this host. An agent that lacks a grant asks for it as a decision, with the declaration it needs written out by `cy access grant --dry-run`.

## 10. the first grants

All at depth 0, issued by the root alone, valid 90 days and renewed by a new declaration. The owner signs them once by the offline path and signs the head; everything after is the mechanism.

| subject | object | verbs | waits for |
|---|---|---|---|
| Joy | `host:agent-a/user:joy`, `host:cyberproxy/user:joy`, `host:deimos/user:joy`, `host:jupiter/user:joy` | ssh | the `joy` accounts (apparatus a1); sudo on any of them is a separate grant and the owner's verdict (§6) |
| Joy | `repo:cyberia-to/*` — never matching apparatus, cyberia, .github (§4) | read, write | Joy's own GitHub identity (§6) |
| Joy | `bot:cyber_ia_bot/*` | read | the bot's token |
| the person the owner names | as the owner says, within the kinds of §6 | as the owner says | who, and which objects |
| Alex, the mayor | `bot:cyber_ia_bot/cyber-valley`; `repo:cyberia-to/cve`, the whole repository — a team cannot be scoped to a directory | read, write | Alex's Telegram id and GitHub login, pinned as credentials |

Joy's spend is none: budgets are not in phase 0 (§6).

## 11. acceptance

- golden vectors: the codec reproduces the bytes, statement, particle and envelope of every vector of §2 from the fixed test key, in cybernode's tests and through `cy access verify`.
- negative vectors, each rejected by `cy access verify` or dropped by `in_force`, with its reason: a flipped byte in the bytes and in the envelope; a wrong issuer (an envelope by another key); expired (`now ≥ until`); not yet valid (`now < from`); revoked, and a grant rooted through a revoked one; a child wider than its parent in object, verbs, interval and depth, each; an unknown root; a wildcard reaching the protected set; a grant for another host or account than the one rendered (`host:deimos/user:joy` is not rendered on agent-a); non-canonical bytes (unsorted verbs, a duplicate verb, a verb the kind does not admit, a trailing byte, `from ≥ until`, an ssh credential with options); an edited `index.toml` (no effect); an empty input (no change); a set missing a particle of the signed head (no change); a new declaration whose nonce is at or below its issuer's in the signed head.
- a declaration signed on galapot by the offline path verifies on agent-a with `cy access verify`; in phase 1 its link verifies with `wallet::verify` under the pussy chain id and the reader recovers the same grant.
- `cy access render --host agent-a` writes the managed `authorized_keys` of every account a grant in force names, and of no other, from declarations alone; a key added by hand to a managed file is gone on the next tick (15 minutes), with a cockpit alert naming it; the break-glass file is untouched.
- the revocation drill: the root revokes Joy's `host:agent-a/user:joy`; within one tick after the revocation reaches origin Joy's ssh is refused, its open session on agent-a is ended, and the cockpit lists the secrets of that account for rotation; a re-grant with a new nonce restores access, and the revoked particle stays revoked.
- with its own GitHub identity, Joy pushes a branch and opens a pull request to a stack repository, and cannot merge it.
- non-exposure: after every ceremony a grep for the spell's words and the secret key's hex finds nothing in the shell histories, in the argv and environments of the processes captured during the ceremony, in `cy access`'s logs, in the apparatus repository and in the agents' transcripts on the machine; `from_spell` does not occur in `cy access`'s source.
- the cockpit shows the declarations in force per subject and per object, the alerts of §7, and for every host the time since its last verified input.

## 12. order

| phase | repo | move | gate |
|---|---|---|---|
| 0 | soft3 | this page | the review's findings disposed; the owner's verdict on §13 |
| 0 | owner | the offline path: the Ledger signs an ADR-036 document for a `neuron1…` signer | the assembled envelope verifies with `verify_statement` |
| 0 | cybernode | the C2 compile proof | `cargo test` green on a pinned sibling set ([[soft3/proposals/cybernode|cybernode]] C2) |
| 0 | cybernode | `Signer::sign_statement`; the codec and `in_force` as a module beside the wallet, public so every subject can check its own grants; the vectors | round trip with `verify_statement`; §11's golden and negative vectors |
| 0 | cyb | `access` in `cli/tools.toml` | `cy install access`; `cy access verify` on the vectors |
| 0 | apparatus | [a1](https://github.com/cyberia-to/apparatus/blob/master/tasks/a1-access-declarations.md): the `joy` accounts, Joy's `[[agent]]` row and the neuron fields, the owner files for break-glass, the host timers, the GitHub App with Members only, the bot's allow-list file | the render on agent-a; the drill |
| 0 | apparatus | the owner's ceremony: the first grants and the head | §11, the non-exposure check included |
| 1 | cybernode, apparatus | the chain reader: links from pussy, bytes from the mirror | `in_force` from the chain equals `in_force` from the files |
| 2 | apparatus | hand-made grants removed wherever a render reaches | a full tick removes nothing |

## 13. decisions for the owner

Each names its section here and the apparatus side, [a1](https://github.com/cyberia-to/apparatus/blob/master/tasks/a1-access-declarations.md).

1. the root and its rotation ([§2 rule 1](#2-the-declaration), [§4](#4-the-trust-anchor-and-the-evaluator)): the root is the owner's legacy neuron — the key that also holds the owner's balances — pinned in the renderer; it rotates by a transition the old root signs; recovery after a lost or stolen root waits for a recovery quorum, with break-glass and a renderer release meanwhile. default: yes, and the quorum as its own proposal.
2. the root's signing path in phase 0 ([§3](#3-signing-and-verifying)): the offline ADR-036 path on the owner's hardware wallet, or wait for vault's `authorize_action`. default: offline.
3. sudo and depth ([§6](#6-objects-and-their-renders), [§10](#10-the-first-grants)): Joy's host grants carry ssh only, each sudo a separate verdict; every first grant has depth 0. default: yes.
4. Joy's spend ([§6](#6-objects-and-their-renders)): none in phase 0; Akash top-ups stay the owner's. default: none.
5. GitHub identity ([§6](#6-objects-and-their-renders)): a GitHub identity per subject that needs repository grants — Joy's first — or no per-subject repository grants in phase 0; the renderer as a GitHub App with Members only. default: a machine user for Joy; the agent-a bodies keep `cyberia-apparatus` outside the declarations until each has its own.
6. the time source ([§4](#4-the-trust-anchor-and-the-evaluator)): the host's synchronized clock in phase 0, the chain's median time past from phase 1. default: yes.
7. phase 1 on the chain ([§8](#8-where-the-declarations-live)): amount-0 links on pussy from genesis, or files until the chain has run a month. default: files now, the chain after the first month.
8. the person ([§10](#10-the-first-grants)): who, and which objects. no link yet; the owner names them.
9. subject custody ([§5](#5-subjects)): Joy's key derived in custody on galapot, each agent-a body's in its own. default: yes.

related: [[mudra/specs/neuron-auth|neuron-auth]] · [[mudra/specs/identity|identity]] · [[vault/specs/authorization|vault authorization]] · [[vault/specs/secrets|typed secrets]] · [[cybernode/specs/chain|chain]] · [[soft3/proposals/cybernode|cybernode]] · [apparatus a1](https://github.com/cyberia-to/apparatus/blob/master/tasks/a1-access-declarations.md) (the apparatus's side)
