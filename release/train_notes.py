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

    def asset(filename, label=None):
        url = assets.get(filename, f'{repo}/releases/download/{quote(name)}/{quote(filename)}')
        return link(label or filename, url)

    def evidence(platform, gate=None):
        if audit_url and gate and gate.get('log'):
            path = f"logs/{platform['target']}/{Path(gate['log']).name}"
            return f'{audit_url}/{quote(path, safe="/")}'
        if platform.get('archive'):
            return assets.get(platform['archive'], f'{repo}/releases/download/{quote(name)}/{quote(platform["archive"])}')
        return assets.get('release-validation.json', f'{repo}/releases/download/{quote(name)}/release-validation.json')

    platforms = validation['platforms']
    has_binaries = bool(candidate.get('available_binaries'))
    headline = '🟢 GREEN · gates passed' if validation['result'] == 'green' else '🔴 RED · qualification incomplete'
    lines = [f'## 🚦 {cell(component)} · {cell(name)}', '',
             '| Readiness | Distribution |', '|---|---|',
             f'| {headline} | 📝 Draft prerelease |',
             f'| {"📦 Binaries available" if has_binaries else "⛔ No installable binary in this candidate"} | Promotion: owner decision |', '',
             '🟢 Pass · 🔴 Fail · 🟠 Blocked by a prerequisite · ⚪ No receipt · ➖ Gate not selected.', '',
             '## 🔗 Versions and sources', '', '| Product | Version | Captured source |', '|---|---|---|']
    versions = candidate.get('versions') or {}
    for product in ['soft3', 'cyber', 'cyb']:
        revision = candidate.get('source_revisions', {}).get(product)
        source = link(f'`{revision[:8]}`', f'https://github.com/cyberia-to/{product}/commit/{revision}') if revision else '⚪ Unavailable'
        lines.append(f'| {link(product, f"https://github.com/cyberia-to/{product}")} | {cell(versions.get(product) or "unresolved")} | {source} |')
    manager = candidate.get('manager_revision')
    if manager:
        lines += ['', f'Qualifier: {link("soft3 " + manager[:8], f"https://github.com/cyberia-to/soft3/tree/{manager}/release")}. '
                  f'Exact versions and revisions: {asset("candidate.json")}.']
    lines += ['', '## ✨ Changes at the captured revisions', '']
    for index, change in enumerate(sources.get('changes', []), 1):
        lines.append(f'{index}. {cell(change["component"])}: {link(cell(change["title"]), change["html_url"])}.')
    if not sources.get('changes'):
        lines.append('⚪ No attributed pull requests in the source receipt.')
    for error in sources.get('change_errors', []):
        lines += ['', f'🟠 {cell(error["component"])}: {cell(error["error"])}.']
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
    lines += ['', 'Cell links open the corresponding log or receipt. ➖ means the gate was not selected for that job; ⚪ means its platform receipt is missing.', '', '</details>', '',
              '<details>', '<summary>📚 Full soft3 source inventory</summary>', '',
              'Pin status compares source revisions only. Component test results are in the qualification matrix above.', '',
              '| Component | Captured revision | Source pin | Primary declared package |', '|---|---|---|---|']
    platform_inventories = inventory.get('platforms', {})
    declared = {r['name']: r for data in platform_inventories.values() for r in data.get('repositories', [])}
    for row in sources.get('repositories', []):
        revision = row.get('revision')
        repository = row.get('repo', f'cyberia-to/{row["name"]}')
        commit = link(f'`{revision[:8]}`', f'https://github.com/{repository}/commit/{revision}') if revision else '⚪ Unavailable'
        pin = '📌 Product source' if row['name'] in PRODUCTS else ('🟢 Matches' if row.get('pin_matches') else '🔴 Missing / drifted')
        primary = [p for p in declared.get(row['name'], {}).get('declared_packages', []) if p['manifest'] in ['Cargo.toml', 'rs/Cargo.toml', 'crate/Cargo.toml']]
        packages = '; '.join(cell(p['name']) + (' @ ' + cell(p['version']) if isinstance(p.get('version'), str) else ' · workspace version') for p in primary)
        lines.append(f'| {link(cell(row["name"]), "https://github.com/" + repository)} | {commit} | {pin} | {packages or asset("soft3-dependencies.json", "Declarations ↗")} |')
    lines += ['', 'Declared versions come from source manifests. Resolved package closure, resolution failures and all nested declarations are retained in ' + asset('soft3-dependencies.json') + '.', '', '</details>', '',
              '## 📎 Evidence and downloads', '', '| Asset | Contents |', '|---|---|']
    for filename, description in [('SHA256SUMS', 'Checksums for the original candidate assets'),
                                  ('candidate.json', 'Product versions, exact source revisions and available binaries'),
                                  ('sources.json', 'Origin source capture, pins and change attribution'),
                                  ('release-validation.json', 'Every platform, command, verdict and log hash'),
                                  ('soft3-dependencies.json', 'Machine-readable source and package inventories'),
                                  ('soft3-dependencies.md', 'Original readable dependency inventory'),
                                  ('release-notes.md', 'Original generated notes captured with this candidate')]:
        lines.append(f'| {asset(filename)} | {description} |')
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
