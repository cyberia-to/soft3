#!/usr/bin/env python3
"""requirements.py — the counter of cyberia/dev § requirements, for soft3.

reads a specification whose normative headings carry ` · ID` and end with a
cases table, and the map in audit/<product>/map.md (requirement · code ·
test · proof · state); prints one row per requirement with the state the
map records, downgraded to `open` when the clause has no case or the map
names no code or test, and to `stale` when any commit touched the code
after the receipt landed. exit 0 always; the numbers are the result.

usage: python3 scripts/requirements.py [--spec specs/chain.md] [--map audit/chain/map.md] [--dots]
"""
import argparse, re, subprocess, sys, os

def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True).stdout.strip()

def read_spec(path):
    ids, cases = [], {}
    cur = None
    for line in open(path, encoding="utf-8"):
        m = re.match(r"^#{1,6} .*· ([A-Z][A-Z0-9-]+)\s*$", line)
        if m:
            cur = m.group(1); ids.append(cur); cases[cur] = 0; continue
        m = re.match(r"^\| ([A-Z][A-Z0-9-]+)\.[a-z] \| (positive|negative) \|", line)
        if m and m.group(1) in cases:
            cases[m.group(1)] += 1
    return ids, cases

def read_map(path):
    rows = {}
    for line in open(path, encoding="utf-8"):
        if not line.startswith("| CHAIN-") and not re.match(r"^\| [A-Z][A-Z0-9-]+ \|", line):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 5: continue
        rows[cells[0]] = dict(code=cells[1], test=cells[2], proof=cells[3], state=cells[4])
    return rows

ORDER = ["open", "stale", "simulated", "measured", "proven"]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", default="specs/chain.md")
    ap.add_argument("--map", default="audit/chain/map.md")
    ap.add_argument("--dots", action="store_true", help="print only the three counts")
    a = ap.parse_args()
    ids, cases = read_spec(a.spec)
    rows = read_map(a.map)
    proof_dir = os.path.dirname(a.map)
    out, counts, notes = [], {k: 0 for k in ORDER}, []
    for rid in ids:
        r = rows.get(rid)
        state, why = "open", ""
        if cases[rid] == 0:
            why = "placeholder: no case"
        elif r is None:
            why = "not in the map"
        elif r["code"] in ("—", "") or r["test"] in ("—", ""):
            why = "no code or no test in the map"
        else:
            state = r["state"] if r["state"] in ORDER else "open"
            if r["proof"] not in ("—", ""):
                receipt = os.path.join(proof_dir, r["proof"])
                landed = git("log", "--format=%H", "--diff-filter=A", "--", receipt).split("\n")[-1]
                if not os.path.exists(receipt) or not landed:
                    state, why = "open", "receipt missing"
                else:
                    paths = [p.strip() for p in r["code"].split(";") if p.strip()]
                    missing = [p for p in paths if not os.path.exists(p)]
                    if missing:
                        state, why = "open", "code path missing: " + ", ".join(missing)
                    else:
                        changed = git("log", "--oneline", f"{landed}..HEAD", "--", *paths)
                        if changed:
                            n = len(changed.split("\n"))
                            state, why = "stale", f"{n} commit(s) touched the code after {r['proof']}"
            elif state != "open":
                state, why = "open", "no receipt"
        counts[state] += 1
        out.append((rid, cases[rid], state, why))
    for rid in rows:
        if rid not in ids: notes.append(f"map names {rid}, which the spec does not carry")
    if a.dots:
        red = counts["open"] + counts["stale"]; yellow = counts["simulated"]; green = counts["measured"] + counts["proven"]
        print(f"🔴 {red} 🟡 {yellow} 🟢 {green} of {len(ids)}")
        return
    w = max(len(r[0]) for r in out)
    for rid, n, state, why in out:
        print(f"{rid:<{w}}  cases {n:>2}  {state:<9} {why}")
    print()
    print("  ".join(f"{k} {v}" for k, v in counts.items()), f" of {len(ids)} requirements, {sum(cases.values())} cases")
    red = counts["open"] + counts["stale"]; yellow = counts["simulated"]; green = counts["measured"] + counts["proven"]
    print(f"dots: 🔴 {red} 🟡 {yellow} 🟢 {green}")
    for n in notes: print("note:", n)

if __name__ == "__main__":
    main()
