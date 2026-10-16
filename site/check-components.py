#!/usr/bin/env python3
"""check every surface against release/components.toml: the home ladder, the chart, status.md, phase1.toml. exits 1 on drift."""
import re, sys, tomllib, pathlib
root = pathlib.Path(__file__).resolve().parents[1]
reg = tomllib.loads((root/'release/components.toml').read_text())['component']
names = {c['name'] for c in reg}
byname = {c['name']: c for c in reg}
errs = []
# the site is one deck; the ladder and the chart are drawn from site/data.js (NODES/EDGES) by ladder-fig.js and chart-fig.js
import json
data = (root/'site/data.js').read_text()
nodes_json = data[data.index('const NODES = ')+len('const NODES = '):]; nodes_json = nodes_json[:nodes_json.index('];')+1]
NODES = json.loads(nodes_json)
nodes = {n['id'] for n in NODES}
rungs = {n['id'] for n in NODES if n.get('hub') or n.get('layer', 0) > 7}
chips = nodes - rungs
for x in sorted(chips - names): errs.append(f'ladder chip not in registry: {x}')
def covered(c, seen=frozenset()):
    if c['name'] in chips or c['name'] in rungs: return True
    p = c.get('part_of')
    return bool(p) and p in byname and p not in seen and covered(byname[p], seen | {p})
APPS = {'cyb','cyber','cyberia'}
def on_site(c):  # products built on soft3 are not decomposed on the site
    return 'layer' in c and not (c['layer'] > 7 and c['name'] not in APPS)
for c in reg:
    if on_site(c) and not covered(c): errs.append(f'layer {c["layer"]} component missing from the ladder: {c["name"]}')
for x in sorted(nodes - names): errs.append(f'chart node not in registry: {x}')
for c in reg:
    if on_site(c) and c['name'] not in nodes and not (c.get('part_of') in nodes): errs.append(f'layer {c["layer"]} component missing from the chart: {c["name"]}')
# status.md: every crate/product row exists
st = (root/'status.md').read_text()
for c in reg:
    if c['kind'] in ('crate','product') and c['name'] not in ('silicon','cy','cyb','true-cyber','trident','cyberia') and not (re.search(r'\[\[(?:[^\]]*\|)?%s\]\] \|' % re.escape(c['name']), st) or re.search(r'\| \[%s\]\(' % re.escape(c['name']), st)):
        errs.append(f'status.md has no row for: {c["name"]}')
# phase1.toml: every phase-1 crate is a sibling; every sibling is a phase-1 component
p1 = tomllib.loads((root/'release/phase1.toml').read_text())
sib = {s['name'] for s in p1.get('sibling', [])}
crates = {c['name'] for c in reg if c['kind']=='crate' and c['phase']=='1' and c['name'] not in ('silicon','trident','joy','nox')}
for x in sorted(sib - names): errs.append(f'phase1.toml sibling not in registry: {x}')
for x in sorted(crates - sib): errs.append(f'phase-1 crate missing from phase1.toml: {x}')
if errs:
    print('\n'.join(errs)); print(f'{len(errs)} drift(s)'); sys.exit(1)
print(f'components: {len(reg)} in registry, {len(chips)} ladder chips, {len(nodes)} chart nodes, {len(sib)} siblings — no drift')
