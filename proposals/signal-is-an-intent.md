---
tags: cyber, soft3, proposal, signal, intent, cybergraph, neural, tok
crystal-type: process
crystal-domain: cyber
alias: signal is an intent, the owner's correction of the signal
status: proposed
date: 2026-11-03
task: tasks/one-message.md (branch prop/one-message, soft3#677), unit 1 of the re-cut
---
# signal is an intent — the owner's correction applied across the specifications

## the correction

the owner, 2026-11-03 (translated): the essence of a [[signal]] is that one [[neuron]] creates it as an [[intent]] with provable conditions of completion; then any neurons can fulfil it, even in different blocks; a signal fits in one block only when it is small.

what changes in the meaning:

- declaration: one neuron declares the intent and signs its [[scope]], the provable conditions of completion.
- fulfilment: any neurons fulfil it, possibly in different blocks.
- seal: the signal is complete when its proof attests the scope, $\sigma(s) \vdash \text{scope\_hash}(i)$; the seal commits the whole fulfilment or nothing.
- size: only a small signal fits in one block.

clauses that already say it: cybergraph `specs/intent.md:56–62` (the intend/seal gate), `:36` (an unsealed intent stays on record), `:38–40` and `:94–96` (others discover the intent, fulfil it in sub-signals, the lead seals the parent); `specs/signal.md:16–21` ($h_0$, $h_1$, the span, the intent before the seal); soma `soma-spec.md:1082` ("an intent exists (mine, or one I claim)") and `:1094` (two-phase "when the action is a multi-step computation or invites coordination"); neural `specs/wire.md:36` and `specs/cli.md:97` ("one-shot for small signals"); cyber `intent.md:28` (any neuron self-assigns), `cascade.md:7–27`.

what does not change: one $\sigma$ at the seal covers everything the seal commits (proof aggregation); the seal binding; the lead seals a cascade's parent (`intent.md:40`, `cyber:cascade.md:24`); the one-shot `link(signal)` path for a small local statement; the [[cyberlink]] as the atom. the field set, the chain format and the launch are other units of the task and are not touched here.

## amendments

origin heads read: cybergraph `0a70b6a`, neural `47b086c`, soft3 `3045d54`, bbg `2294281`, tok `c773592`, cyber `68f1260a`, cyb `8957982d`, soma `1d3fe69`; searched and unchanged: tru `e660d1e`, mudra `3969c7a`, nox `49348be`, zheng `ecf2491`, foculus `cdff8dc`, cybernode `52538a6`. `-` is the exact span of the line that changes, byte for byte; `+` replaces that span; the rest of the line stays.

### cybergraph — 13 lines

C1 · `specs/signal.md:3` — the definition was the bundle in one step the owner rejects.

```diff
- a bundle of [[cyberlinks]] a [[neuron]] commits in a single [[step]] — the atomic broadcast unit in [[cyber]].
+ an [[intent]] one [[neuron]] declares with provable conditions of completion, fulfilled in [[cyberlinks]] by any neurons, possibly in different blocks, and sealed when its proof attests them; only a small signal fits in one block.
```

C2 · `specs/README.md:18` — the index restates C1.

```diff
- - [signal.md](signal.md) — the broadcast unit a neuron commits
+ - [signal.md](signal.md) — an intent a neuron declares, fulfilled by any neurons and sealed
```

C3 · `specs/intent.md:16` — the run may be any neurons', not the declarer's.

```diff
- it proves the neuron actually *ran* the scope.
+ it proves the scope was actually *run*.
```

C4 · `specs/intent.md:23` — completion was the declaring neuron's alone; who seals stays as `:40` and `:96` say (the lead).

```diff
-    ├──→ sealed     — neuron finalizes: adds sealing height + zheng proof → [[signal]]
+    ├──→ sealed     — fulfilled by any neurons, possibly across blocks; the seal adds sealing height + zheng proof → [[signal]]
```

C5 · `specs/intent.md:32` — same as C3, in the definition of the sealed state.

```diff
- a sealed intent is a [[signal]]: the neuron ran the scope, produced a [[zheng]] proof $\sigma$ over the execution (its [[cyberlinks]] and [[cyber/impulse]]), and committed it.
+ a sealed intent is a [[signal]]: the scope was run by any neurons, possibly in different blocks; a [[zheng]] proof $\sigma$ was produced over the execution (its [[cyberlinks]] and [[cyber/impulse]]) and committed.
```

C6 · `specs/intent.md:62` — "one transaction" is the single step again; "one thing" is the wording of `docs/README.md:86`; the proof attests the declared scope, whoever ran it.

```diff
- this binding is what makes intent and signal **one transaction** rather than two unrelated records.
+ this binding is what makes intent and signal **one thing** rather than two unrelated records.
- a sealed signal is cryptographic proof the neuron did exactly what it declared
+ a sealed signal is cryptographic proof that what the neuron declared was done exactly
```

C7 · `specs/README.md:72` — as C3 and C6.

```diff
- the neuron runs it ([[nox]]) and proves it ([[zheng]]);
+ any neurons run it ([[nox]]), possibly across blocks, and prove it ([[zheng]]);
- that binding makes intent and signal one transaction,
+ that binding makes intent and signal one thing,
```

C8 · `README.md:45` — as C6.

```diff
- a sealed signal is cryptographic proof the neuron did exactly what it declared
+ a sealed signal is cryptographic proof that what the neuron declared was done exactly
- this binding is what makes intent and signal **one transaction**,
+ this binding is what makes intent and signal **one thing**,
```

C9 · `docs/README.md:78` — the scope is computed by whoever fulfils it.

```diff
- it is an executable specification of what the neuron will compute.
+ it is an executable specification of what will be computed.
```

C10 · `docs/README.md:90` — as C6.

```diff
- a sealed signal is therefore cryptographic proof that the neuron did exactly what it declared
+ a sealed signal is therefore cryptographic proof that what the neuron declared was done exactly
```

C11 · `docs/README.md:92` — the seal gates on the declared run, not on the sealer having run it.

```diff
- the agent cannot seal a computation it did not run as declared.
+ the agent cannot seal a computation that was not run as declared.
```

C12 · `specs/neuron.md:39` — "the atomic act" restates C1.

```diff
- - [[signal]] — the atomic act a neuron performs
+ - [[signal]] — the intent a neuron declares, sealed once fulfilled
```

C13 · `specs/cyberlink.md:23` — "bundled for broadcast" restates C1; the rest of the line (one recursive proof over the batch) is proof aggregation and stays.

```diff
- cyberlinks are bundled into [[signal|signals]] for broadcast.
+ cyberlinks are carried by [[signal|signals]].
```

### neural — 8 lines

N1 · `specs/wire.md:16` — "lands together or not at all" is the one-step submission.

```diff
- a signal is one atomic submission — a batch that lands together or not at all.
+ a signal is one [[intent]] by one [[neuron]] and its seal — fulfilled by any neurons, possibly across blocks; only a small signal fits in one block.
```

N2 · `specs/wire.md:37` — an unsealed intent exists on record (cybergraph `intent.md:36`); the all-or-nothing belongs to the seal.

```diff
- - atomicity — the whole signal commits or aborts; half a [[sentence]] does not exist
+ - atomicity — the seal commits the whole signal or nothing; before the seal a [[sentence]] is an intent on record, not yet an utterance
```

N3 · `specs/cli.md:95` — as N1.

```diff
- an atomic batch of links by one [[neuron]], with a [[zheng]] proof of well-formedness.
+ an intent by one [[neuron]], its links fulfilled by any neurons and sealed with a [[zheng]] proof of well-formedness.
```

N4 · `specs/cli.md:98` — as N2.

```diff
- - **atomic** — the whole signal lands or none of it does; half a [[sentence]] never exists.
+ - **atomic at the seal** — the seal commits the whole signal or none of it; before it, a [[sentence]] is an intent on record.
```

N5 · `specs/README.md:37` — the cross-cutting law repeated N1's error.

```diff
- - a [[signal]] is the unit of submission: one atomic batch of links.
+ - a [[signal]] is the unit of submission: an [[intent]] by one [[neuron]], its links fulfilled by any neurons, possibly across blocks, and sealed.
```

N6 · `specs/sentence.md:12` — the sentence inherits the signal's boundary, which now runs from intent to seal.

```diff
- the signal boundary defines the utterance — a sentence is atomic, all of its links land together or none do.
+ the signal boundary — intent to seal — defines the utterance: a sentence is whole at its seal, its links in one block only when it is small.
```

N7 · `docs/README.md:130` — "a single transaction" as the utterance boundary.

```diff
- a batch packed into a single transaction. The transaction boundary defines the utterance. Order within the batch encodes grammar.
+ cast in one [[signal]]. The signal boundary — intent to seal — defines the utterance. Order within the signal encodes grammar.
```

N8 · `docs/README.md:157` — "transaction-atomic"; the cybergraph does see unsealed intents (cybergraph `intent.md:94`).

```diff
- Transaction-atomic [[semantics]]: every transaction is a linguistic act. A half-submitted [[sentence]] is no [[sentence]] at all — the [[cybergraph]] sees only complete utterances.
+ Signal-bounded [[semantics]]: every signal is a linguistic act. An unsealed [[sentence]] is an intent on record, not yet an utterance — the [[cybergraph]] counts only sealed utterances.
- a clear beginning (transaction start) and end (transaction commit)
+ a clear beginning (intent) and end (seal)
```

### soft3 — 3 lines

S1 · `specs/terms.md:76` — the glossary defined the signal as a batch at one height; the two heights are cybergraph `signal.md:16–17`.

```diff
- - `signal` — a signed, staked batch of `cyberlinks`; carries the `subject` (neuron) and block height.
+ - `signal` — an intent one `subject` (neuron) declares and signs with provable conditions of completion, fulfilled in staked `cyberlinks` by any neurons, possibly across blocks; carries the heights of declaration and sealing.
```

S2 · `specs/languages.md:41` — "transaction-atomic batch", as N7.

```diff
- [[sentence]]s (a transaction-atomic batch of [[cyberlinks]] — the utterance)
+ [[sentence]]s (a chain of [[cyberlinks]] cast in one [[signal]], whole at its seal — the utterance)
```

S3 · `specs/languages.md:235` — "one transaction", as N7.

```diff
- a [[sentence]] packs dialects into one transaction (the utterance)
+ a [[sentence]] packs dialects into one [[signal]] (the utterance)
```

### bbg — 1 line

B1 · `docs/explanation/signal-sync.md:25` — the unit of sync was defined as a batch; what syncs is the sealed signal.

```diff
- it is a signal — a batch of cyberlinks with a zheng proof.
+ it is a sealed signal — the cyberlinks that fulfil a neuron's intent, with a zheng proof.
```

### tok — 5 lines

the word intent meant two things: cybergraph's declared scope and tok's atomic composition. in the owner's sense they are one: the PLUMB operations are the provable conditions, fulfilled by any neurons; the all-or-nothing is the seal's.

T1 · `plumb.md:22` — the heading named atomicity as what an Intent is.

```diff
- ## atomic composition: Intent
+ ## composition: Intent
```

T2 · `plumb.md:24` — atomicity was the definition of the Intent.

```diff
- an [[Intent]] is one or more PLUMB operations composed atomically. all operations commit together, or none commit.
+ an [[Intent]] is one or more PLUMB operations one neuron declares as the provable conditions of its completion; any neurons may fulfil them, in different blocks. at the seal all operations commit together, or none commit.
```

T3 · `plumb.md:68` — the law holds at the seal.

```diff
- 4. Atomicity — all ops in an Intent commit together or none do
+ 4. Atomicity — all ops in an Intent commit together at its seal or none do
```

T4 · `README.md:41` — as T2.

```diff
- - one composition rule: operations compose into [[Intent\|Intents]] that commit atomically or roll back
+ - one composition rule: operations compose into [[Intent\|Intents]] one neuron declares and any neurons fulfil, committed atomically at the seal or rolled back
```

T5 · `README.md:54` — as T3.

```diff
- | Atomicity | all ops in an Intent commit together or none do |
+ | Atomicity | all ops in an Intent commit together at its seal or none do |
```

### cyber — 4 lines

Y1 · `intent.md:24` — as C4.

```diff
- - sealed — neuron finalizes with sealing height + zheng proof → becomes [[signal]]
+ - sealed — fulfilled by any neurons, possibly across blocks; the seal adds sealing height + zheng proof → becomes [[signal]]
```

Y2 · `whitepaper.md:790` — the glossary entry for sentence, as N7.

```diff
- : ordered instruction set of [[cyberlinks]] packed into a single transaction. The transaction boundary defines the utterance. Order within the batch encodes grammar.
+ : ordered instruction set of [[cyberlinks]] cast in one [[cyber/signal]]. The signal boundary — intent to seal — defines the utterance. Order within the signal encodes grammar.
```

Y3 · `specs/money-loop.md:42` — as T2.

```diff
- | Intent | atomic PLUMB op list | [[tok]] PLUMB |
+ | Intent | PLUMB op list one neuron declares and any neurons fulfil, committed at the seal | [[tok]] PLUMB |
```

Y4 · `specs/money-loop.md:43` — as S1.

```diff
- | signal | signed cyberlink batch (+ proof when sealed) | [[cybergraph]], [[foculus]] |
+ | signal | intent signed by one neuron, fulfilled in cyberlinks by any neurons (+ proof when sealed) | [[cybergraph]], [[foculus]] |
```

### cyb — 1 line

K1 · `product/robot.md:105` — copies tok `plumb.md:24`; as T2.

```diff
- an Intent is one or more PLUMB operations composed atomically — they all commit or none do.
+ an Intent is one or more PLUMB operations one neuron declares as the provable conditions of its completion, fulfilled by any neurons — at the seal they all commit or none do.
```

### soma — 1 line

M1 · `soma-spec.md:1092` — as C6 and C11; `:1082` already lets soma fulfil an intent it claims.

```diff
- a sealed signal proves soma did *exactly* what it declared.
+ a sealed signal proves *exactly* what soma declared was done.
- soma cannot seal a computation it did not run as declared
+ soma cannot seal a computation that was not run as declared
```

## kept — lines that look alike and stay

proof aggregation: "atomic" or "the entire batch" names one $\sigma$ covering everything the seal commits; the owner's sentence moves who fulfils and when, not how many proofs the seal carries (cybergraph `intent.md:40`: one recursive proof over the entire cascade). cybergraph `specs/signal.md:35` ("covers the entire signal atomically"), `specs/proof.md:9` ("covering the whole batch atomically"), `:17` ("σ covers, atomically,"), `docs/README.md:107`; tru `specs/impulse.md:23`; cyber `whitepaper.md:470`, `network.md:219`; bbg `specs/architecture.md:167`, `specs/storage.md:445`; foculus `specs/vec.md:109`; tok `programming-model.md:127`.

the seal's all-or-nothing: "atomic" as a property of the seal, which N2, N4 and T3 keep. cybergraph `specs/signal.md:50` ("within one atomic signal": both effects of validation happen at the one seal); cyber `cascade.md:29` (each sub-signal whole at its own seal); tok `plumb.md:33` (pending until the seal); cyb `product/robot.md:68` ("commits or rolls back at workflow transition"); foculus `docs/explanation/latency-targets.md:80`, `:306`.

record names: "batch" as the stored set of a sealed signal's links. cybergraph `specs/signal.md:14`, `specs/query.md:56`, `:97`; bbg `README.md:43`, `specs/state.md:37`, `specs/storage.md:382`, `:443–446`, `specs/indexes.md:174`, `docs/explanation/architecture-overview.md:38`, `:71`; foculus `specs/structural-sync.md:38`, `:218–220`; nox `specs/reduction.md:535`; neural `specs/wire.md:21`; tru `specs/focusing.md:139`; cyber `whitepaper.md:466`, `:894`, `network.md:212`, `:227`.

the small signal, already right: one neuron, one shot, no intent phase. cybergraph `README.md:47`, `:71`, `docs/README.md:80`, `specs/cli.md:39`; soma `README.md:91`, `soma-spec.md:1063`; cyb `decide/wire.md:31` ("one atomic signal at startup", two links); cyber `whitepaper.md:905`; tru `docs/explanation/incentives.md:43`.

the cyberlink, not the signal, is the atom: cybergraph `specs/README.md:19`, `specs/cyberlink.md:3`, `specs/query.md:50`, `docs/README.md:52`; cyber `cyberlink.md:9`; tok `plumb.md:10` (each operation).

left to the other units: cybergraph `specs/order.md:15` (the height rule names one $t$; which of $h_0$, $h_1$ it is belongs to the field set, unit 2); soft3 and cybernode `specs/native-node.md:18` ("One accepted signal advances one block", the node's block policy, unit 2); soft3 `specs/chain-wire.md:42` (chain format, unit 2); cyber `launch.md:48` (launch, unit 3). not specifications or doctrine: `audit/`, `roadmap/`, `research/`, `.claude/` (cyber `roadmap/a-local-node.md:85`, `:96`, `research/state model.md:27`, neural `.claude/plans/neu.md`).

## checks

1. verbatim: every `-` span above re-read from `git show origin/HEAD:<path>` of its repository at the head named in § amendments, by a script that parses this page and asserts each span occurs on its cited line: 41 spans on 36 lines, 0 misses. the line of the soma span is `:1092`; `:1094` is the two-phase test cited in § the correction. every kept line and every clause cited as already right was checked the same way: 67 fragments, one fragment mistyped by me (`intent.md:60`, the display formula), re-cited as the range `:56–62`.
2. completeness: first search, `atomic|bundle|batch of|one transaction|single step|in a single|transaction-atomic|together or none|commits or aborts`, over `*.md` of all fourteen repositories at origin heads; second, `a signal is|signal =|signals are|entire batch|link batch|signal_batch|one-shot|lands together`; third, `neuron (ran|runs|finalizes|did exactly)|one block|same block|half a sentence|utterance|roll back|commit together`, which added C3, C5, C9–C11 and M1; a fourth with new phrasings, `indivisible|whole signal|complete signal|unit of submission|broadcast unit|one authorization|same transaction|emit a signal|in one signal`, added C2 and nothing else. every hit is in § amendments or § kept.
3. minimality: each `+` touches only the span the reason names. changed while checking: "one transaction" becomes "one thing", the wording of cybergraph `docs/README.md:86`, not "one signal", which made C6–C8 circular; C3 drops "by any neurons", the passive is enough; who seals is left as it stands (the lead, cybergraph `intent.md:40`, `:96`); the proof lines that the first draft of soft3#677 amended (its A2, A3) are kept, because there "atomic" is proof aggregation.
