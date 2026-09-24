#!/usr/bin/env python3
"""check every surface against release/components.toml: home chips, layers page, status.md, phase1.toml. exits 1 on drift."""
import re, sys, tomllib, pathlib
root = pathlib.Path(__file__).resolve().parents[1]
reg = tomllib.loads((root/'release/components.toml').read_text())['component']
names = {c['name'] for c in reg}
errs = []
# home: every chip is a component; every crate/product with a facet and phase 1 (not network) is a chip
home = (root/'site/index.html').read_text()
chips = set(re.findall(r'<i>([a-z0-9-]+)</i>', home[home.index('data-go="play"'):home.index('class="band base"')]))
for x in sorted(chips - names): errs.append(f'home chip not in registry: {x}')
expect = {c['name'] for c in reg if c['kind']=='crate' and c['name'] not in ('silicon',)}
for x in sorted(expect - chips - {'honeycrisp'}): errs.append(f'component missing from home chips: {x}')
# layers: every component with a layer appears on the page
lay = (root/'site/layers/index.html').read_text().lower()
for c in reg:
    if 'layer' in c and c['name'] not in lay: errs.append(f'layer {c["layer"]} component missing from layers page: {c["name"]}')
# status.md: every crate/product row exists
st = (root/'status.md').read_text()
for c in reg:
    if c['kind'] in ('crate','product') and c['name'] not in ('silicon','cy','cyb','true-cyber','trident') and not (re.search(r'\[\[(?:[^\]]*\|)?%s\]\] \|' % re.escape(c['name']), st) or re.search(r'\| \[%s\]\(' % re.escape(c['name']), st)):
        errs.append(f'status.md has no row for: {c["name"]}')
# phase1.toml: every phase-1 crate is a sibling; every sibling is a phase-1 component
p1 = tomllib.loads((root/'release/phase1.toml').read_text())
sib = {s['name'] for s in p1.get('sibling', [])}
crates = {c['name'] for c in reg if c['kind']=='crate' and c['phase']=='1' and c['name'] not in ('silicon','trident','joy','nox')}
for x in sorted(sib - names): errs.append(f'phase1.toml sibling not in registry: {x}')
for x in sorted(crates - sib): errs.append(f'phase-1 crate missing from phase1.toml: {x}')
if errs:
    print('\n'.join(errs)); print(f'{len(errs)} drift(s)'); sys.exit(1)
print(f'components: {len(reg)} in registry, {len(chips)} chips, {len(sib)} siblings — no drift')
