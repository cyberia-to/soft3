---
title: chain-wire — network, sync and HTTP of the phase-1 root chain
tags: cyber, soft3, spec, chain, radio, tade, network, sync
crystal-type: spec
crystal-domain: cyber
status: accepted
date: 2026-10-15
alias: chain wire, chain sync, chain network, chain api
---

# chain-wire — network, sync and HTTP of the phase-1 root chain

The wire of [chain.md](chain.md): how nodes find each other, what they
gossip, how a node catches up, what it refuses, and the HTTP surface cyb and
true-cyber keep using. Transport is cyber-radio (the iroh fork: QUIC,
hole-punching, relay, gossip, blobs); framing is tade; nothing here changes
a consensus rule.

## 1. transport

- QUIC through `radio` (`iroh::Endpoint`), one endpoint per node process,
  ALPN `cyber/chain/1`; the node's endpoint id (32 bytes) is its wire
  identity; the node key lives in `$home/<chain>/radio.key`;
- every stream carries tade frames; the first frame of every stream is the
  dialect declaration `(*, k)` with `urn:cyberia:chain:1`; a stream whose
  first frame is anything else is closed;
- two kinds of exchange: gossip (iroh-gossip, §2) for announcements and
  bidirectional request streams (§3) for everything with a size;
- relays: the cyber relay on helia-relay is configured as the default relay
  for nodes behind NAT; the three cybernode machines publish direct
  addresses and need none;
- the chain id separates the two chains on the wire: topic ids derive from
  it (§2) and the status exchange (§3) rejects a peer on another chain.

## 2. gossip topics

Topic ids are 32 bytes, `hemera("soft3/chain/topic/v1\0" ‖ chain ‖ name)`:

| name | frame | payload | max size | who publishes |
|---|---|---|---|---|
| `headers` | `H` | one 168-byte header | 168 B | the miner of a new block; every node that validated it (steps 1–5) relays |
| `txs` | `T` | one transaction (§2 of chain.md) | 512 B | the submitter; every node that admitted it to its mempool relays |
| `checkpoints` | `C` | `height: u64 ‖ block_id: [32] ‖ sig: NSIG1[102]` | 142 B | the checkpoint signer; relayed after signature check |
| `shadow/<core>` | `X` | core-defined bytes | 4,096 B | shadow indexers (chain.md §7.3); never consensus |

Rules:

- a node joins `headers`, `txs` and `checkpoints` of each chain it runs; the
  `shadow/*` topics are joined only with the `shadow` feature;
- bodies are never gossiped: a header announces, the body is fetched (§3).
  At 168 bytes a header fits any gossip message limit; a 256 KiB body does
  not, and pushing bodies to peers that have not checked the PoW is the
  header-first rule broken;
- gossip relay happens only after local validation: a header after PoW, a
  transaction after mempool admission, a checkpoint after its signature;
  an invalid message is dropped and scored (§5);
- the iroh-gossip `max_message_size` is set to 4,096 on every topic; larger
  frames are a protocol error.

## 3. request streams

A node opens a bidirectional stream to a peer, declares the dialect, sends
one request frame, reads response frames until the end frame, and closes.
One request per stream; a peer may keep up to 8 streams open per direction.

| frame | direction | payload | meaning |
|---|---|---|---|
| `s` | req | `chain: [32] ‖ rule_set: u16 ‖ height: u64 ‖ tip: [32] ‖ chainwork: [u8; 32] LE ‖ genesis_block: [32]` | status; the first request on every new connection, answered with the same frame |
| `g` | req | `count: u8 ‖ locator: [32] × count ‖ stop: [32]` | get-headers: the locator is the requester's main-chain block ids, newest first, the last 10 then doubling spacing back to genesis; respond from the first locator entry on the responder's main chain |
| `h` | resp | `count: u16 ‖ header × count` | headers, ≤ 2,000 per response, in ascending height; `count = 0` means the locator has no common point except genesis mismatch (peer is on another genesis — disconnect) |
| `r` | req | `from: u64 ‖ count: u16` | get-range: bodies of the responder's main chain from height `from`, `count ≤ 128` |
| `b` | req | `count: u8 ‖ id: [32] × count` | get-blocks by id, `count ≤ 128` |
| `B` | resp | `header[168] ‖ body` | one block per frame, in request order |
| `t` | req | `count: u16 ‖ txid: [32] × count` | get-txs from the mempool, `count ≤ 256` |
| `T` | resp | tx bytes | one per frame |
| `k` | req | `from_height: u64` | get-checkpoints at or above a height |
| `C` | resp | checkpoint (as in §2) | one per frame |
| `e` | resp | empty | end of response |
| `x` | resp | `code: u8 ‖ utf8 reason` | error; codes: 1 unknown, 2 pruned, 3 limit, 4 wrong chain |

All integers little-endian; every list is count-prefixed; a response frame
larger than `MAX_BLOCK_BYTES + 16` is a protocol error.

### 3.1 header-first sync

On every new connection and whenever a gossiped header has an unknown
parent:

1. exchange `s`; disconnect on chain or genesis mismatch; if the peer's
   `chainwork` is not greater than ours and its tip is known, stop;
2. `g` with our locator; validate every returned header through steps 1–5
   of chain.md §5.5 against our header store; repeat `g` from the last
   received header while responses are full (2,000);
3. once a header chain with more work than our tip is known, request bodies
   for it (§3.2) in ascending height, validate steps 6–8, and move the tip
   when the chainwork rule says so (chain.md §5.1–5.2);
4. headers on a chain whose fork point is deeper than `MAX_REORG` are
   stored (they are valid headers) but no bodies are requested; the event
   is logged as a rejected deep fork.

### 3.2 initial block download

A node with no bodies (fresh install, or a pruned node turning archival)
downloads by height ranges along the best header chain:

- the range is 128 blocks; up to 4 ranges are in flight, each to a different
  peer that advertised a tip at or above the range's end;
- a range not fully delivered in 30 s is reassigned; a peer delivering a
  block whose header differs from the one we validated for that height is
  scored and the range reassigned;
- blocks are applied strictly in height order; out-of-order deliveries wait
  in a bounded buffer (512 blocks); the node reports `catching-up: true`
  until its tip is within 2 blocks of the best known header;
- genesis is never downloaded: the genesis file is a blob the node already
  has (embedded particle; `radio` blobs can fetch it by particle from any
  peer if the operator passes only the chain name).

State is rebuilt by replay; there is no state-snapshot sync in phase 1
(chain.md §3.6). On pussy a full replay is minutes; on bostrom it is the
row-43 cost and is the reason bostrom waits.

## 4. mempool

- admission in this order: decode (size ≤ 512, known kind), txid not
  present, signature, nonce in `[current, current + NONCE_WINDOW)`, fee
  floor, balances sufficient assuming all lower-nonce mempool txs of the
  same neuron apply first, coinbase maturity on the spent balance;
- capacity 8,192 transactions or 16 MiB; eviction by lowest fee per byte,
  then oldest; a tx older than 24 h is dropped;
- after a new tip: re-validate every tx (nonces and balances moved), drop
  the invalid, re-admit txs from reorged-out blocks;
- block templates take txs by fee per byte descending with per-neuron nonce
  order preserved, up to the budgets of chain.md §1.6; the coinbase is built
  last from `SUBSIDY + Σ fees`.

## 5. limits, scoring, eclipse resistance

Per-peer limits (token buckets, burst = 4× rate):

| what | rate |
|---|---|
| headers received | 64 /s |
| transactions received | 256 /s |
| bytes received | 4 MiB/s |
| request streams opened by the peer | 16 /s |
| unsolicited block bodies | 0 — bodies arrive only in response to `r`/`b`; any other is a protocol error |

Scoring: a peer starts at 0; +100 and disconnect for an invalid PoW header,
a wrong-chain status, a body whose header we did not request, or a frame
over size; +10 for an invalid signature, a bad nonce beyond the window, a
non-canonical encoding, a header failing the time rule by more than 1 h;
+1 for a duplicate; score decays by 1 per minute; ≥ 100 bans the endpoint id
for 1 h. Nothing is ever charged for a header that fails only the
future-drift rule within the hold window.

Eclipse basics:

- 8 outbound connections, at most 2 to endpoints sharing an IPv4 /16 or
  IPv6 /32 among their direct addresses, at least 1 to a bootstrap endpoint
  until at least 4 other outbound peers are up; up to 64 inbound;
- the bootstrap set (the three cybernode endpoint ids) is compiled in and
  can be extended, not replaced, by the operator;
- anchors: the last 2 outbound peers that served valid blocks are
  remembered and redialed first on restart;
- the miner does not mine without at least 1 outbound peer whose status tip
  is within 2 blocks of ours (parano1d's sync-quorum gate): an isolated node
  never mines a private fork by accident;
- `/status` reports peer count, outbound count and the best header seen
  from each, so an operator can see a lighter chain being served.

What this does not do: no address-gossip protocol, no reputation beyond the
score, no proof-of-stake-weighted peer selection. The residual is stated in
chain.md §8.

## 6. discovery — the wire obeys the graph

Two sources of peers, in order:

1. the compiled bootstrap set;
2. the graph. A node publishes its own address as cyberlinks from its
   neuron, exactly as cyb's `wire.rs` does:
   `(p = hemera("antenna"), q = endpoint_id)` and
   `(p = hemera("socket"), q = packed socket)` with the packed form of
   `wire.rs` (`'4' ‖ ipv4[4] ‖ port BE[2] ‖ zeros`), `amount = 0`,
   `valence = 0`, at the fee floor. Every node indexes kind-2 transactions
   whose `p` is one of those two particles into an address book
   `neuron → (endpoint_id, [socket])`, newest wins; a node that wants peers
   beyond the bootstrap set dials entries of the book, preferring neurons it
   `follow`s (`p = hemera("follow"), q = neuron`) and then a uniform sample.

A node's address costs it two transactions (514 units at the fee floor) and
is public, signed and permanent in the chain — which is what makes it
discovery and not a hidden layer. The locus routing of
[routing.md](routing.md) applies above this once the follow graph is dense;
for the first months the book plus the bootstrap set is enough.

## 7. HTTP

The existing adapter (native-node.md: bounds, request identity, error codes
503/409/400/422) is kept and bound per chain: `127.0.0.1:7781` (pussy),
`127.0.0.1:7782` (bostrom), behind nginx at `cyb.ai/pussy/` and
`cyb.ai/bostrom/`. Routes:

| method, path | response | note |
|---|---|---|
| `GET /health` | `ok` | unchanged |
| `GET /status` | cybermark status particle | fields below |
| `GET /root` | hex bbg root of the tip | unchanged |
| `GET /stats` | graph statistics | unchanged |
| `GET /tip` | `height ‖ id ‖ chainwork` JSON | new |
| `GET /header/{height or id}` | hex header + JSON fields | new |
| `GET /block/{height or id}` | JSON: header fields, txids, size, miner, fees, state root | existed by height; id added |
| `GET /blocks?after=H&limit=N` | ≤ 500 block summaries | existed; `after` cursor added |
| `GET /tx/{txid}` | JSON decoded tx + `height` (or `mempool`) | new |
| `GET /nonce/{neuron}` | next nonce (mempool-aware) | new |
| `GET /balance/{neuron}` | native balance | unchanged; `?token=hex` for others |
| `GET /balances/{owner}` | every token of an owner | new |
| `GET /mempool` | txids and byte total | new |
| `GET /peers` | per peer: endpoint id, direction, best header, score | new |
| `GET /checkpoints` | the checkpoints held | new |
| `GET /epoch/{E}`, `GET /epochs?after=E&limit=N` | epoch records and payout lists | new, chain-economy §11 |
| `GET /focus/{particle}`, `GET /focus?top=N` | φ* of the last settled epoch | new, chain-economy §11 |
| `GET /card/{id}`, `GET /cards/{neuron}` | positions and their stake | new, chain-economy §11 |
| `GET /template?to={neuron}` | header with zero nonce (hex), target, body (hex) | mining template for external workers; expires with the tip |
| `POST /submit` | hex header of a solved template | accepted blocks are relayed; `409` if the template is stale |
| `POST /tx` | body `application/octet-stream` tx bytes, or JSON `{"tx": "hex"}` | returns txid; mempool admission errors are `400` with the reason |
| `GET /v2/history?after=POSITION&limit=N` | `cyber/native-history/v1` entries | kept; each entry gains `height`, `block` and `txid`; `encoding: CGOP-v1-hex` kept for cyberlink entries |
| `POST /v1/link`, `/v1/pay`, `/v1/frame`, `/v2/frame`, `GET /log` | `410 Gone` on chain nodes, with a pointer to `POST /tx` | the unsigned chaosnet ingress has no place on a public chain; `spacepussy-test` keeps them |

Answer shapes the kernel fixes (hex lowercase, bytes in wire order,
integers as JSON numbers, u256 as decimal strings; unknown fields in a
request are rejected). The wallet (`soft3::wallet::client`) reads these;
the node implements them over `chain::api`:

```
GET /tip
  200 {"height":H,"id":"<hex32>","chainwork":"<decimal u256>"}

GET /nonce/{neuron}
  200 {"neuron":"<hex32>","nonce":N,"confirmed":C}
      nonce = the next nonce a transaction may carry (confirmed + the
      mempool's consecutive pending txs of this account); confirmed = the
      state's `neurons[ν].nonce` at the tip
  400 {"error":"<reason>"}                 not a 32-byte hex id

POST /tx   body: application/octet-stream tx bytes, or JSON {"tx":"<hex>"}
  200 {"txid":"<hex32>","kind":"transfer|cyberlink|claim","size":B,"fee":F,
       "account":"<hex32>","nonce":N}       admitted (or already present) and relayed
  400 {"error":"<reason>"}                 decode, signature, nonce, fee, balance,
                                           maturity — the reason text of chain::apply::TxError
  410 {"error":"..."}                      on a chaosnet node (no chain)

GET /tx/{txid}
  200 {"txid":"<hex32>","kind":"<name>","tx":{<decoded fields>},
       "height":H|"mempool","block":"<hex32>"|null,"index":I|null}
      decoded fields: coinbase {height,to,amount,extra}; transfer
      {from,nonce,token,amount,to,fee}; cyberlink {from,nonce,token,amount,
      p,q,valence,fee}; claim {account_id,nonce,fee}; plus "sig":"<hex102>"
      for signed kinds; 32-byte ids and sigs hex, amounts numbers, valence
      −1|0|1
  404 {"error":"unknown"}                  never seen, dropped, or pruned

GET /template?to={neuron}
  200 {"chain":"<name>","height":H,"prev":"<hex32: tip id>",
       "header":"<hex168: candidate with nonce zero>","bits":"<hex8>",
       "target":"<hex32: the 32 LE bytes of decode(bits)>","body":"<hex>",
       "expires":<unix second, 0 = none>}
      `chain::api::Template::json` renders exactly this; `target` repeats
      `bits` decoded so an external worker needs no compact codec

POST /submit   {"header":"<hex168: the solved header>"}
  200 {"id":"<hex32>","height":H}         accepted and relayed
  409 {"error":"stale template"}          prev is no longer the tip
  422 {"error":"<reason>"}                pow, bits or syntax (chain.md §5.5 1–5)
  400 {"error":"<reason>"}                malformed request
```

A template is tied to its `prev`: a miner polls `GET /tip` (every 2 s in
the reference miner, `soft3 mine`, `crate/src/chain/mine/api.rs`) and
refreshes when `id` moves, and refreshes anyway after 30 s to pick up new
transactions; a miner may check that `target` and `bits` agree. The 16-byte nonce (bytes 24..40) is the miner's alone and
nothing else in the header may change between template and submit.

`/status` fields (cybermark, one per line as today): `chain`, `protocol:
soft3/chain/1`, `rule-set`, `activations`, `engine: chain+cybergraph+bbg`,
`version`, `moniker`, `height`, `tip`, `chainwork`, `bits`, `next-bits`,
`genesis` (the chain id — the genesis file's particle, what a wallet
signs for), `genesis-block` (block 0's id), `genesis-time`, `bbg-root`,
`supply`, `conserved: true|false`, `subsidy`, `neurons`, `particles`,
`axons`, `mempool`, `peers`, `outbound`, `checkpoint`, `finalized`,
`catching-up`, `mining: true|false` (a miner took a `/template` within the
last 60 s; the node itself never mines), `storage: bbg-fjall`. true-cyber's sync probe reads `height`, `tip`, `catching-up` and
`chain`, nothing else, so a chain node answers an unchanged client.

Bounds: headers 64 KiB, bodies 8 MiB, one `Content-Length`, no transfer
encoding, unknown JSON fields rejected — all as native-node.md already
states. Shadow indexers expose `/shadow/<core>/…` on the same server when
built with the `shadow` feature; those routes are documented by the cores.

See [chain.md](chain.md) for the rules this wire carries and
[chain-conformance.md](chain-conformance.md) for the wire vectors.
