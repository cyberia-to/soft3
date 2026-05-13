---
tags: cyber, soft3, python
crystal-type: spec
crystal-domain: cyber
---
# soft3 Python SDK

Python client for the soft3 stack. target audience: data science, ML pipelines, AI integration.

## API

```python
from soft3 import CyberClient, particle, cyberlink

# particle CID computation
cid = particle(b"hello world")  # hemera hash → 32-byte CID

# connect to node
client = CyberClient("https://rpc.bostrom.cybernode.ai")

# query
p = client.query_particle(cid)
print(p.energy, p.rank)
print(p.proof.verify(client.bbg_root))

# submit cyberlink
receipt = client.cyberlink(
    from_cid=particle(b"python"),
    to_cid=particle(b"cybergraph"),
    neuron=my_key.neuron_id(),
    signer=my_key,
)

# search
results = client.search("soft3 stack")
for r in results:
    print(r.cid.hex(), r.rank)
```

## install

```
pip install soft3
```

not yet published — scaffold only.

## implementation

depends on:
- hemera WASM or FFI bindings (CID computation)
- BBG proof verification (Lens opening)
- query wire protocol (`schema/`)

blocked on schema stabilisation and lens serde.
