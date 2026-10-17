#!/usr/bin/env python3
"""rehearsal-genesis — derive a rehearsal genesis from a `soft3 genesis build` file.

Rewrites the params frame (name, time, bits, subsidy) of a pipeline-built
genesis tade stream and inserts funded balances in canonical (owner, token)
order. Everything else (dialect frame, every snapshot balance and link) is
kept byte for byte. The output is checked by `soft3 genesis verify` and by
the node's own `Genesis::decode` (subsidy = floor(S0 / 105,120,000)).

  rehearsal-genesis.py IN OUT --name pussy-rc --time UNIX --bits 0x1e040000 \
           --fund NEURON_HEX=AMOUNT [--fund ...] [--drop-links]

Rehearsal nets only (`pussy-rc`); a launch genesis is `soft3 genesis build`
output, untouched. A run with the input's own name/time/bits and no --fund
reproduces the input byte for byte (checked 2026-10-16 on pussy.tade,
sha256 461fbfb4…). --drop-links keeps every balance and omits the 29,113
link frames: with them a block template costs 17 s on an M4 Max and 26.6 s
on deimos (the bbg root is recommitted per template), see
audit/chain/rehearsal-2026-10-16.md.
"""
import argparse, hashlib, struct, sys

MARK = 0x1F

def varint(b, i):
    n = s = 0
    while True:
        c = b[i]; i += 1
        n |= (c & 0x7F) << s; s += 7
        if c < 0x80:
            return n, i

def enc_varint(n):
    out = bytearray()
    while True:
        c = n & 0x7F; n >>= 7
        if n: out.append(c | 0x80)
        else:
            out.append(c); return bytes(out)

def frames(b):
    i = 0
    while i < len(b):
        assert b[i] == MARK, f"byte {i}: no marker"
        sig, ren = b[i + 1], b[i + 2]
        ln, j = varint(b, i + 3)
        yield sig, ren, b[j:j + ln]
        i = j + ln

def frame(sig, ren, data):
    return bytes([MARK, sig, ren]) + enc_varint(len(data)) + data

def rd_str(d, i):
    n = d[i]; return d[i + 1:i + 1 + n].decode(), i + 1 + n

def params_decode(d):
    v = struct.unpack_from("<I", d, 0)[0]; i = 4
    name, i = rd_str(d, i); hrp, i = rd_str(d, i); denom, i = rd_str(d, i)
    time, bits, subsidy = struct.unpack_from("<QIQ", d, i); i += 20
    ck = d[i:i + 33]; i += 33
    sunset = struct.unpack_from("<Q", d, i)[0]; i += 8
    assert i == len(d), "params trailing bytes"
    return dict(version=v, name=name, hrp=hrp, denom=denom, time=time, bits=bits,
                subsidy=subsidy, ck=ck, sunset=sunset)

def params_encode(p):
    s = lambda x: bytes([len(x.encode())]) + x.encode()
    return (struct.pack("<I", p["version"]) + s(p["name"]) + s(p["hrp"]) + s(p["denom"])
            + struct.pack("<QIQ", p["time"], p["bits"], p["subsidy"]) + p["ck"]
            + struct.pack("<Q", p["sunset"]))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("inp"); ap.add_argument("out")
    ap.add_argument("--name", required=True)
    ap.add_argument("--time", type=int, required=True)
    ap.add_argument("--bits", required=True)
    ap.add_argument("--token", help="native token id hex (default: the token of the largest balance set)")
    ap.add_argument("--fund", action="append", default=[])
    ap.add_argument("--drop-links", action="store_true", help="omit every link frame (light rehearsal state)")
    a = ap.parse_args()
    raw = open(a.inp, "rb").read()
    fr = list(frames(raw))
    assert fr[0][:2] == (ord("*"), ord("k")), "first frame not (*, k)"
    assert fr[1][:2] == (ord("~"), ord("p")), "second frame not (~, p)"
    p = params_decode(fr[1][2])
    bal = [f[2] for f in fr[2:] if f[:2] == (ord("$"), ord("b"))]
    links = [f[2] for f in fr[2:] if f[:2] == (ord("#"), ord("l"))]
    assert len(bal) + len(links) == len(fr) - 2
    if a.drop_links:
        links = []
    if a.token:
        native = bytes.fromhex(a.token)
    else:
        from collections import Counter
        native = Counter(b[32:64] for b in bal).most_common(1)[0][0]
    for f in a.fund:
        n, amt = f.split("=")
        owner = bytes.fromhex(n); assert len(owner) == 32
        rec = owner + native + struct.pack("<Q", int(amt))
        assert all(b[:64] != rec[:64] for b in bal), f"{n} already holds the native token"
        bal.append(rec)
    bal.sort(key=lambda b: b[:64])
    s0 = sum(struct.unpack_from("<Q", b, 64)[0] for b in bal if b[32:64] == native)
    old = dict(p)
    p.update(name=a.name, time=a.time, bits=int(a.bits, 16), subsidy=s0 // 105_120_000)
    out = frame(ord("*"), ord("k"), fr[0][2]) + frame(ord("~"), ord("p"), params_encode(p))
    out += b"".join(frame(ord("$"), ord("b"), b) for b in bal)
    out += b"".join(frame(ord("#"), ord("l"), l) for l in links)
    open(a.out, "wb").write(out)
    print(f"in   {a.inp} sha256 {hashlib.sha256(raw).hexdigest()} name {old['name']} time {old['time']} bits {old['bits']:#010x} subsidy {old['subsidy']}")
    print(f"out  {a.out} ({len(out)} bytes) sha256 {hashlib.sha256(out).hexdigest()}")
    print(f"     name {p['name']} hrp {p['hrp']} denom {p['denom']} time {p['time']} bits {p['bits']:#010x}")
    print(f"     native token {native.hex()}  S0 {s0}  subsidy {p['subsidy']}  balances {len(bal)}  links {len(links)}")

if __name__ == "__main__":
    main()
