"""Trace declared component inputs from the captured Git objects, without Cargo resolution.

Optional, target-specific, test and patch declarations are retained as such.
Registry declarations identify ownership only: the captured source is not a
claim that Cargo selected that revision or even the same package version.
"""
import argparse
from collections import defaultdict, deque
import hashlib
import json
from pathlib import Path, PurePosixPath
import posixpath
import subprocess
import tomllib

ROOTS = {'soft3': 'crate/Cargo.toml', 'cyber': 'Cargo.toml', 'cyb': 'shell/Cargo.toml'}


def read_manifests(sources, checkout):
    manifests = {}
    errors = []
    for row in sources['repositories']:
        revision = row.get('revision')
        if not revision:
            continue
        root = Path(checkout) / row['name']
        try:
            paths = subprocess.check_output(
                ['git', 'ls-tree', '-r', '--name-only', revision], cwd=root, text=True).splitlines()
            for path in paths:
                if PurePosixPath(path).name != 'Cargo.toml':
                    continue
                raw = subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=root)
                try:
                    data = tomllib.loads(raw.decode())
                except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
                    errors.append({'manifest': f'{row["name"]}/{path}', 'error': str(error)})
                    continue
                manifests[f'{row["name"]}/{path}'] = {
                    'data': data, 'sha256': hashlib.sha256(raw).hexdigest(),
                    'url': f'https://github.com/{row["repo"]}/blob/{revision}/{path}',
                }
        except (OSError, subprocess.CalledProcessError) as error:
            errors.append({'component': row['name'], 'error': str(error)})
    return manifests, errors


def trace(sources, manifests, errors=()):
    """Return a conservative declaration graph, never a resolved binary closure."""
    by_name = {r['name']: r for r in sources['repositories']}
    trace_errors = list(errors)

    def workspace(path):
        data = manifests[path]['data']
        explicit = data.get('package', {}).get('workspace')
        if explicit:
            return posixpath.normpath(f'{posixpath.dirname(path)}/{explicit}/Cargo.toml')
        parent = PurePosixPath(path).parent
        while len(parent.parts) >= 1:
            candidate = str(parent / 'Cargo.toml')
            if 'workspace' in manifests.get(candidate, {}).get('data', {}):
                return candidate
            parent = parent.parent
        return None

    def package(path):
        item = manifests[path]['data'].get('package', {})
        version = item.get('version')
        if isinstance(version, dict) and version.get('workspace'):
            ws = workspace(path)
            version = manifests.get(ws, {}).get('data', {}).get('workspace', {}).get('package', {}).get('version')
        return {'name': item.get('name'), 'version': version, 'manifest': path,
                'url': manifests[path]['url']}

    packages = {p: package(p) for p in manifests if manifests[p]['data'].get('package', {}).get('name')}
    owners = defaultdict(set)
    for path, item in packages.items():
        owners[item['name']].add(path.split('/')[0])

    component = sources['component']
    root = f'{component}/{ROOTS[component]}'
    rows = {}
    relations = []
    queue = deque([(root, ())])
    visited = set()

    def add(owner, relation=None, target=None):
        row = rows.setdefault(owner, {'name': owner, 'packages': [], 'relations': [], 'missing_manifests': []})
        if relation and relation not in row['relations']:
            row['relations'].append(relation)
            relations.append(relation)
        if target in packages and packages[target] not in row['packages']:
            row['packages'].append(packages[target])
        elif target and target not in manifests and target not in row['missing_manifests']:
            row['missing_manifests'].append(target)

    def dependency(path, alias, value, kind, target, inherited):
        value = {'version': value} if isinstance(value, str) else dict(value)
        declaration = path
        conditions = set(inherited)
        if kind == 'dev-dependencies':
            conditions.add('test')
        if kind == 'build-dependencies':
            conditions.add('build')
        if kind == 'patch':
            conditions.add('patch')
        if target:
            conditions.add(target)
        if value.get('workspace'):
            ws = workspace(path)
            shared = manifests.get(ws, {}).get('data', {}).get('workspace', {}).get('dependencies', {}).get(alias)
            if shared is None:
                issue = {'manifest': path, 'dependency': alias, 'error': 'workspace dependency missing'}
                if issue not in trace_errors:
                    trace_errors.append(issue)
                return
            shared = {'version': shared} if isinstance(shared, str) else shared
            value = {**shared, **value}
            declaration = ws
        if value.get('optional'):
            conditions.add('optional')
        package_name = value.get('package', alias)
        dest = None
        if value.get('path'):
            dest = posixpath.normpath(f'{posixpath.dirname(declaration)}/{value["path"]}/Cargo.toml')
            if dest.startswith('../') or dest.startswith('/'):
                return
            owner = dest.split('/')[0]
            binding = 'path'
        else:
            candidates = owners.get(package_name, set())
            if len(candidates) != 1:
                return
            owner = next(iter(candidates))
            binding = 'git' if value.get('git') else 'registry'
        relation = {'from': path, 'declaration': declaration, 'url': manifests[declaration]['url'],
                    'alias': alias, 'package': package_name, 'requirement': value.get('version'),
                    'kind': kind, 'binding': binding, 'conditions': sorted(conditions), 'to': dest}
        add(owner, relation, dest)
        if dest in manifests:
            queue.append((dest, tuple(sorted(conditions))))

    while queue:
        path, inherited = queue.popleft()
        if (path, inherited) in visited or path not in manifests:
            continue
        visited.add((path, inherited))
        data = manifests[path]['data']
        add(path.split('/')[0], target=path)
        groups = [('', data), *data.get('target', {}).items()]
        for target, group in groups:
            for kind in ['dependencies', 'build-dependencies', 'dev-dependencies']:
                # Dependency crates' tests are not product test inputs.
                if kind == 'dev-dependencies' and path != root:
                    continue
                for alias, value in group.get(kind, {}).items():
                    dependency(path, alias, value, kind, target, inherited)
        if path == root:
            ws = workspace(path)
            for patch_path in dict.fromkeys(p for p in [root, ws] if p):
                for registry in manifests[patch_path]['data'].get('patch', {}).values():
                    for alias, value in registry.items():
                        dependency(patch_path, alias, value, 'patch', '', ())

    # soft3 qualifies the phase-1 stack, which is broader than its CLI's deps.
    if component == 'soft3':
        for owner in by_name:
            if owner in ROOTS:
                continue
            add(owner)
            rows[owner]['packages'] = [p for path, p in packages.items() if path.split('/')[0] == owner]
        rows = {n: r for n, r in rows.items() if n not in ROOTS}
    else:
        rows.pop(component, None)

    for owner, row in rows.items():
        source = by_name.get(owner, {})
        row.update(repo=source.get('repo'), revision=source.get('revision'),
                   pin_matches=source.get('pin_matches'), captured=bool(source.get('revision')))
        row['packages'].sort(key=lambda p: p['manifest'])
    return {'component': component, 'candidate': sources['candidate'],
            'source_revisions': {r['name']: r.get('revision') for r in sources['repositories']},
            'basis': 'Captured Git manifest declarations; optional, target, test and patch inputs included. '
                     'Registry ownership does not establish a resolved version or revision.',
            'components': sorted(rows.values(), key=lambda r: r['name']),
            'manifest_evidence': {p: {'sha256': manifests[p]['sha256'], 'url': manifests[p]['url']}
                                  for p in sorted(manifests)},
            'errors': trace_errors}


def capture(sources, checkout):
    manifests, errors = read_manifests(sources, checkout)
    return trace(sources, manifests, errors)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--checkout', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = capture(json.loads(args.sources.read_text()), args.checkout)
    result['command'] = ['python3', 'release/train_components.py', '--sources', str(args.sources),
                         '--checkout', str(args.checkout), '--output', str(args.output)]
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
