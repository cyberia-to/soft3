---
tags: cyber, soft3, sdk
crystal-type: spec
crystal-domain: cyber
---
# soft3

developer experience layer for the [[soft3]] stack. makes the 14-repo stack accessible without understanding every repo.

```
neurons / apps
      ↓
 soft3 SDK          ← this repo
      ↓
cybergraph ─── bbg ─── tru ─── glia ─── mir
      ↓           ↓
   radio         lens / strata
```

## what it provides

| component | purpose | status |
|-----------|---------|--------|
| [js/](js/) | JavaScript/TypeScript SDK (current Bostrom chain) | active |
| [schema/](schema/) | canonical wire format definitions | draft |
| [cli/](cli/) | `soft3` command-line tool | scaffold |
| [mcp/](mcp/) | MCP server — cybergraph tools for AI assistants | scaffold |
| [py/](py/) | Python SDK | scaffold |

## core operations

every SDK exposes the same five operations regardless of language:

```
particle(content)              → CID          hemera hash of bytes
cyberlink(from, to, neuron)    → signal       construct + sign a cyberlink
query(cid, dimension)          → value+proof  BBG Lens opening
verify(root, proof)            → bool         proof verification
submit(signal)                 → receipt      send signal to network
```

## quick start (JS)

```ts
import { CyberClient } from '@cybercongress/cyber-js'

const client = await CyberClient.connect('https://rpc.bostrom.cybernode.ai')
const result = await client.rank.search('cyber')
```

## status

dependencies not yet stabilised — full implementation blocked on:
- wire format finalisation (`schema/`)
- BBG proof serialisation (lens `Commitment`/`Opening` serde)
- query RPC protocol definition

scaffold is in place. implementations land per component as deps stabilise.

see [[soft3]] for the full stack description.
