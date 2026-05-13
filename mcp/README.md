---
tags: cyber, soft3, mcp
crystal-type: spec
crystal-domain: cyber
---
# soft3 MCP server

MCP server exposing the cybergraph to AI assistants (Claude, GPT, etc.).

## tools

| tool | input | output | layer |
|------|-------|--------|-------|
| `particle` | bytes or text | CID (hemera hash) | hemera |
| `cyberlink` | from, to, neuron | signal receipt | cybergraph |
| `search` | query string | ranked particles + scores | tru / rank |
| `query_particle` | CID | energy, rank, axon count + proof | bbg |
| `query_neuron` | CID | focus, karma, stake + proof | bbg |
| `query_axons` | CID, direction | axon list + proof | bbg |
| `verify_proof` | BBG_root, proof | valid/invalid | bbg / lens |
| `get_focus` | — | φ* distribution (top-N) | tru |

## transport

stdio (standard MCP transport). connects to a running soft3 node or directly to a public RPC.

## configuration

```json
{
  "mcpServers": {
    "soft3": {
      "command": "soft3-mcp",
      "args": ["--node", "https://rpc.bostrom.cybernode.ai"]
    }
  }
}
```

## implementation

not yet implemented — scaffold only. blocked on:
- query wire protocol (`schema/`)
- at least one working SDK (cli or py)

MCP server is a thin adapter over the CLI or Python SDK — no direct stack dependencies.
