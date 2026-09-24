"""Render candidate receipts as a readable GitHub release page."""
import argparse
import html
import json
from pathlib import Path
import re
import tarfile
from urllib.parse import quote

PRODUCTS = {'soft3', 'cyber', 'cyb'}
PLATFORMS = {
    'stack': '🧩 Shared stack',
    'aarch64-apple-darwin': '🍎 macOS · ARM64',
    'x86_64-apple-darwin': '🍎 macOS · x64',
    'aarch64-unknown-linux-gnu': '🐧 Linux · ARM64',
    'x86_64-unknown-linux-gnu': '🐧 Linux · x64',
    'aarch64-linux-android': '🤖 Android · ARM64',
}
LABELS = {
    'green': '🟢 Pass', 'red': '🔴 Fail', 'blocked': '🟠 Blocked',
    'missing': '⚪ No receipt',
}


def cell(value):
    value = re.sub(r'\s+', ' ', str(value)).strip()
    return html.escape(value, quote=False).replace('|', '&#124;').replace('[', '&#91;').replace(']', '&#93;').replace('`', "'")


def link(label, url):
    return f'[{label}]({url})'


def render(directory, metadata=None, audit_url=None, run_url=None):
    directory = Path(directory)
    candidate = json.loads((directory / 'candidate.json').read_text())
    sources = json.loads((directory / 'sources.json').read_text())
    validation = json.loads((directory / 'release-validation.json').read_text())
    inventory = json.loads((directory / 'soft3-dependencies.json').read_text())
    component, name = candidate['component'], candidate['name']
    repo = f'https://github.com/cyberia-to/{component}'
    assets = {a['name']: a['browser_download_url'] for a in (metadata or {}).get('assets', [])}

    def asset_url(filename):
        # Draft download URLs rotate when GitHub updates the intended tag.
        # Committed receipt copies give current draft pages durable links.
        if audit_url:
            suffix = '?raw=true' if filename.endswith('.tar.gz') else ''
            return f'{audit_url}/{quote(filename)}{suffix}'
        return assets.get(filename, f'{repo}/releases')

    def asset(filename, label=None):
        return link(label or filename, asset_url(filename))

    def evidence(platform, gate=None):
        if audit_url and gate and gate.get('log'):
            path = f"logs/{platform['target']}/{Path(gate['log']).name}"
            return f'{audit_url}/{quote(path, safe="/")}'
        if platform.get('archive'):
            return asset_url(platform['archive'])
        return asset_url('release-validation.json')

    platforms = validation['platforms']
    has_binaries = bool(candidate.get('available_binaries'))
    headline = '🟢 GREEN · gates passed' if validation['result'] == 'green' else '🔴 RED · qualification incomplete'
    lines = [f'## 🚦 {cell(component)} · {cell(name)}', '',
             '| Readiness | Distribution |', '|---|---|',
             f'| {headline} | 📝 Draft prerelease |',
             f'| {"📦 Binaries available" if has_binaries else "⛔ No installable binary in this candidate"} | Promotion: owner decision |', '',
             '🟢 Pass · 🔴 Fail · 🟠 Blocked by a prerequisite · ⚪ No receipt · ➖ Gate not selected.', '',
             ]
    versions = candidate.get('versions') or {}
    revision = candidate.get('source_revisions', {}).get(component)
    commit = link(f'`{revision[:8]}`', f'{repo}/commit/{revision}') if revision else '⚪ Unavailable'
    lines += [f'{link(component, repo)} {cell(versions.get(component) or "unresolved")} · {commit}', '']
    component_position = len(lines)
    lines += ['## ✨ Changes at the captured revision', '']
    changes = [c for c in sources.get('changes', []) if c['component'] == component]
    for index, change in enumerate(changes, 1):
        lines.append(f'{index}. {link(cell(change["title"]), change["html_url"])}.')
    if not changes:
        lines.append('⚪ No attributed pull requests in the source receipt.')
    for error in sources.get('change_errors', []):
        if error['component'] == component:
            lines += ['', f'🟠 {cell(error["error"])}.']
    lines += ['', '## 💻 Platforms', '', '| Platform | Qualification | Binary | Evidence |', '|---|---|---|---|']
    for platform in platforms:
        status = LABELS.get(platform['result'], '⚪ Unknown') if platform.get('archive') else LABELS['missing']
        binaries = ', '.join(cell(a['name']) for a in platform.get('artifacts', []))
        binary = '➖ Stack checks' if platform['target'] == 'stack' else (binaries or '⛔ Unavailable')
        receipt = asset(platform['archive'], '📦 Logs + receipt') if platform.get('archive') else '—'
        lines.append(f'| {PLATFORMS.get(platform["target"], cell(platform["target"]))} | {status} | {binary} | {receipt} |')
    lines += ['', 'Missing required receipts keep the candidate RED. Archives without binaries contain qualification evidence only.']
    if run_url:
        lines += ['', f'▶️ {link("CI run and platform jobs", run_url)}']
    # Summaries are derived from original logs; full status coverage remains below.
    logs = {}
    for platform in platforms:
        archive = directory / platform.get('archive', '')
        if not archive.is_file():
            continue
        with tarfile.open(archive) as tar:
            for gate in platform.get('gates', []):
                if gate.get('log'):
                    logs[(platform['target'], gate['name'])] = tar.extractfile(gate['log']).read().decode(errors='replace')

    def finding(platform, gate):
        key = gate['name']
        if key == 'origin-checkouts':
            return 'Origin checkout unavailable: ' + ', '.join(f['name'] for f in gate.get('failures', []))
        if key == 'phase1-pins':
            return 'Missing or drifted source pins: ' + ', '.join(gate.get('drift', []))
        if key == 'release-notes-source':
            return 'Missing PR attribution: ' + ', '.join(e['component'] for e in gate.get('errors', []))
        text = gate.get('reason') or gate.get('error') or logs.get((platform['target'], key), '')
        if 'feature `local-storage`' in text:
            return 'Captured cybergraph lacks the required local-storage feature.'
        if '--locked was passed' in text:
            return 'Committed Cargo.lock is missing or needs an update.'
        if 'source revision mismatch:' in text:
            match = re.search(r'source revision mismatch: ([\w-]+)', text)
            return 'Development source lock differs from origin: ' + (match[1] if match else 'see log') + '.'
        if 'no such command: `conformance`' in text:
            return 'The conformance command is not implemented.'
        if 'nu-cli' in text and ('failed to load manifest' in text or 'failed to read' in text):
            return 'nu-cli cannot resolve because the Nu origin is unavailable.'
        if key == 'soft3-dependency' and 'No such file' in text:
            return 'This captured revision has no release/soft3.toml contract.'
        if gate.get('warnings') and gate.get('exit_code') == 0:
            return 'Command succeeded with compiler warnings; zero-warning gate failed.'
        errors = [line.strip() for line in text.splitlines() if re.match(r'\s*(error|warning|timeout)', line)]
        message = errors[0] if errors else text
        cwd = gate.get('cwd')
        if cwd:
            message = message.replace(str(Path(cwd).parent), '<source>')
        return message[:180] or 'See the recorded command and log.'

    component_file = directory / 'component-inputs.json'
    component_inputs = json.loads(component_file.read_text()) if component_file.is_file() else None
    if component_inputs:
        if (component_inputs['component'] != component or component_inputs['candidate'] != name
                or component_inputs['source_revisions'] != candidate['source_revisions']):
            raise ValueError('component declarations differ from captured candidate sources')
    # Older receipts lack the declaration graph. Never invent a product closure.
    if component_inputs:
        component_rows = component_inputs['components']
    elif component == 'soft3':
        declared = {r['name']: r for data in inventory.get('platforms', {}).values()
                    for r in data.get('repositories', [])}
        component_rows = [{**r, 'packages': declared.get(r['name'], {}).get('declared_packages', []),
                           'relations': [], 'captured': bool(r.get('revision'))}
                          for r in sources.get('repositories', []) if r['name'] not in PRODUCTS]
    else:
        component_rows = []
    declarations_file = 'component-inputs.json' if component_inputs else 'soft3-dependencies.json'
    component_lines = ['## 🧩 Stack components' if component == 'soft3' else '## 🧩 Product components', '']
    if component == 'soft3':
        component_lines += ['Components in the phase-1 stack, with their recorded checks. Cyber and Cyb are downstream products.', '']
    else:
        component_lines += ['Components referenced by this product’s captured manifests, including transitive, optional, target, test and patch declarations.', '']
    component_lines += ['| Component | Package / version | Captured revision | Referenced by | Check |', '|---|---|---|---|---|']
    for row in component_rows:
        owner = row['name']
        repository = 'https://github.com/' + row['repo'] if row.get('repo') else None
        label = link(cell(owner), repository) if repository else cell(owner)
        revision = row.get('revision')
        source = link(f'`{revision[:8]}`', f'{repository}/commit/{revision}') if revision and repository else '🔴 Not captured'
        packages = row.get('packages', [])
        if component == 'soft3':
            primary = [p for p in packages if p['name'] in {owner, 'cyber-' + owner}]
            packages = primary or packages
        declared_versions = sorted({p['version'] for p in packages if isinstance(p.get('version'), str)})
        if len(packages) == 1 and declared_versions:
            version = cell(packages[0]['name']) + ' ' + cell(declared_versions[0])
        elif declared_versions:
            version = ', '.join(cell(v) for v in declared_versions[:3])
            if len(declared_versions) > 3:
                version += ', …'
            version += ' · ' + asset(declarations_file, f'{len(packages)} crates')
        else:
            registry = {(r['package'], r['requirement']) for r in row.get('relations', [])
                        if r['binding'] == 'registry' and r.get('requirement')}
            version = '; '.join(cell(pkg + ' ' + req) + ' (registry)' for pkg, req in sorted(registry)) or '—'
        relations = row.get('relations', [])
        references = {}
        for relation in relations:
            parent = relation['from'].split('/')[0]
            if parent == owner:
                continue
            conditions = ['target-specific' if c.startswith('cfg(') else c for c in relation['conditions']]
            suffix = ' · ' + ', '.join(conditions) if conditions else ''
            if relation['binding'] != 'path':
                suffix += ' · ' + relation['binding']
            # Prefer an unconditional reference when both conditional and normal uses exist.
            previous = references.get(parent)
            if previous is None or len(suffix) < len(previous[0]):
                references[parent] = (suffix, relation['url'])
        use = ', '.join(link(cell(parent + suffix), url) for parent, (suffix, url) in list(references.items())[:3])
        if len(references) > 3:
            use += ' · ' + asset(declarations_file, f'+{len(references) - 3}')
        if not use:
            use = link('phase-1 inventory', f'https://github.com/cyberia-to/soft3/blob/{candidate["manager_revision"]}/release/phase1.toml') if component == 'soft3' else 'See declarations'
        matches = [(p, g) for p in platforms for g in p.get('gates', [])
                   if g['name'] in {f'stack-{owner}', f'{owner}-tests', f'{owner}-build'}]
        matches.sort(key=lambda item: {'red': 0, 'blocked': 1, 'green': 2}.get(item[1]['result'], 3))
        if row.get('missing_manifests'):
            status = asset(declarations_file, '🔴 Missing manifest')
        elif not row.get('captured'):
            status = asset(declarations_file, '🔴 Source unavailable')
        elif matches:
            platform, gate = matches[0]
            status = link(LABELS.get(gate['result'], '⚪ Unknown'), evidence(platform, gate))
        else:
            status = '⚪ No component check'
        component_lines.append(f'| {label} | {version} | {source} | {use} | {status} |')
    if not component_rows:
        component_lines.append('| ⚪ Declaration graph unavailable | — | — | — | No inferred dependencies |')
    component_lines += ['', 'Versions and revisions describe captured source. Registry selection and product integration require successful Cargo resolution. '
                        'A component check is the linked recorded gate; ⚪ means no such check in this candidate. '
                        'Missing manifests are detected from the captured Git trees.', '']
    if component_inputs:
        component_lines += [asset('component-inputs.json', '🔎 Exact package requirements, conditional declarations and manifest links') + '.', '']
    if component_inputs and component_inputs.get('errors'):
        component_lines += ['🟠 Some declarations could not be traced; unresolved workspace references are listed in ' + asset('component-inputs.json') + '.', '']
    if component != 'soft3':
        manager = candidate.get('manager_revision')
        component_lines += ['Stack qualification: ' + link('soft3 candidate and component checks',
                            audit_url.replace(f'cyberia-to/{component}/', 'cyberia-to/soft3/') + '/release-page.md#-stack-components' if audit_url
                            else 'https://github.com/cyberia-to/soft3/releases') +
                            (f' · {link("qualifier " + manager[:8], f"https://github.com/cyberia-to/soft3/tree/{manager}/release")}' if manager else '') +
                            '. Separate stack results do not establish this product’s integration.', '']
    lines[component_position:component_position] = component_lines

    selected = {}
    for platform in platforms:
        for gate in platform.get('gates', []):
            if gate['result'] == 'green':
                continue
            # A concrete failure takes precedence over a blocked prerequisite.
            if gate['name'] not in selected or selected[gate['name']][1]['result'] == 'blocked':
                selected[gate['name']] = (platform, gate)
    priority = ['origin-checkouts', 'soft3-dependency', 'cyber-release', 'cyb-check',
                'soft3-release', 'conformance-snapshot', 'stack-hemera', 'optica-build']
    highlighted = [selected[key] for key in priority if key in selected]
    lines += ['', '## 🧱 Main blockers', '', '| Check | Finding | Evidence |', '|---|---|---|']
    for platform, gate in highlighted:
        lines.append(f'| {LABELS.get(gate["result"], "⚪")} · `{cell(gate["name"])}` | {cell(finding(platform, gate))} | {link("Log / receipt ↗", evidence(platform, gate))} |')
    if not highlighted:
        lines.append('| — | See the complete qualification matrix below. | ' + asset('release-validation.json', 'Receipt ↗') + ' |')
    lines += ['', '<details>', '<summary>🧪 Complete qualification matrix — every recorded gate</summary>', '',
              '| Gate | ' + ' | '.join(PLATFORMS.get(p['target'], cell(p['target'])) for p in platforms) + ' |',
              '|---|' + '---|' * len(platforms)]
    gate_names = dict.fromkeys(g['name'] for p in platforms for g in p.get('gates', []))
    for gate_name in gate_names:
        cells = []
        for platform in platforms:
            gate = next((g for g in platform.get('gates', []) if g['name'] == gate_name), None)
            cells.append(link(LABELS.get(gate['result'], '⚪ Unknown'), evidence(platform, gate)) if gate else ('➖' if platform.get('archive') else '⚪'))
        lines.append(f'| `{cell(gate_name)}` | ' + ' | '.join(cells) + ' |')
    lines += ['', 'Cell links open recorded logs or release downloads. ➖ means the gate was not selected for that job; ⚪ means its platform receipt is missing.', '', '</details>', '',
              '## 📎 Evidence and downloads', '',
              f'{link("GitHub release assets", repo + "/releases")} · linked audit copies retain the original checksums.' if audit_url else
              f'Download the named files from {link("GitHub release assets", repo + "/releases")}.',
              '', '| Asset | Contents |', '|---|---|']
    for filename, description in [('SHA256SUMS', 'Checksums for the original candidate assets'),
                                  ('candidate.json', 'Product versions, exact source revisions and available binaries'),
                                  ('sources.json', 'Origin source capture, pins and change attribution'),
                                  ('release-validation.json', 'Every platform, command, verdict and log hash'),
                                  ('soft3-dependencies.json', 'Machine-readable source and package inventories'),
                                  ('soft3-dependencies.md', 'Original readable dependency inventory'),
                                  ('release-notes.md', 'Original generated notes captured with this candidate')]:
        lines.append(f'| {asset(filename)} | {description} |')
    if component_inputs:
        lines.append(f'| {asset("component-inputs.json")} | Product-scoped manifest trace; supplementary evidence |')
    if audit_url:
        lines += ['', f'🗂️ {link("Audit and reproduction commands", audit_url + "/README.md")}']
    lines += ['', 'The verdict, source revisions and qualification data above come from the linked candidate receipts. This presentation does not change their results.', '']
    return '\n'.join(lines)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--metadata', type=Path)
    parser.add_argument('--audit-url')
    parser.add_argument('--run-url')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    metadata = json.loads(args.metadata.read_text()) if args.metadata else None
    args.output.write_text(render(args.directory, metadata, args.audit_url, args.run_url))
