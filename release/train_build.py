"""Consume one immutable soft3 build, authenticated by its checksums manifest."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tomllib

FILES = ('candidate.json', 'sources.json', 'release-validation.json', 'soft3-dependencies.json')
TARGETS = {'stack', 'aarch64-apple-darwin', 'x86_64-apple-darwin',
           'aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu'}


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def contract(text):
    value = tomllib.loads(text)['soft3']
    if value.get('repository') != 'cyberia-to/soft3':
        raise ValueError('soft3 build must belong to cyberia-to/soft3')
    for key, size in [('revision', 40), ('checksums_sha256', 64)]:
        if not re.fullmatch('[0-9a-f]{' + str(size) + '}', value.get(key, '')):
            raise ValueError(f'soft3 contract requires {key}')
    if not isinstance(value.get('release_id'), int) or value['release_id'] <= 0:
        raise ValueError('soft3 contract requires a GitHub release_id')
    if not value.get('build') or not value.get('version'):
        raise ValueError('soft3 contract requires build and version')
    return value


def verify(directory, pin):
    """Verify evidence bytes and source identity; preserve a RED upstream verdict."""
    directory = Path(directory)
    if sha256(directory / 'SHA256SUMS') != pin['checksums_sha256']:
        raise ValueError('soft3 SHA256SUMS differs from the pinned build')
    checksums = {}
    for line in (directory / 'SHA256SUMS').read_text().splitlines():
        checksum, name = line.split('  ', 1)
        path = PurePosixPath(name)
        if (name in checksums or path.is_absolute() or '..' in path.parts
                or not re.fullmatch('[0-9a-f]{64}', checksum)):
            raise ValueError('invalid soft3 checksum entry')
        checksums[name] = checksum
    for name in FILES:
        if checksums.get(name) != sha256(directory / name):
            raise ValueError(f'soft3 build checksum mismatch: {name}')
    candidate, sources, validation, inventory = [json.loads((directory / name).read_text()) for name in FILES]
    revisions = {r['name']: r.get('revision') for r in sources['repositories']}
    if len(revisions) != len(sources['repositories']):
        raise ValueError('duplicate soft3 source input')
    if (candidate['component'] != 'soft3' or sources['component'] != 'soft3'
            or candidate['name'] != pin['build'] or sources['candidate'] != pin['build']
            or candidate['versions']['soft3'] != pin['version']
            or revisions.get('soft3') != pin['revision']
            or candidate['source_revisions'] != revisions
            or validation['source_snapshot_sha256'] != checksums['sources.json']
            or inventory['source_snapshot'] != sources):
        raise ValueError('soft3 build identity or source inventory mismatch')
    platforms = validation['platforms']
    green = (candidate['result'] == validation['result'] == 'green'
             and len(platforms) == len(TARGETS)
             and {p['target'] for p in platforms} == TARGETS)
    for platform in platforms:
        valid = (platform['result'] == 'green' and bool(platform.get('gates'))
                 and all(g['result'] == 'green' for g in platform['gates'])
                 and bool(platform.get('archive'))
                 and checksums.get(platform.get('archive')) == platform.get('archive_sha256')
                 and (platform['target'] == 'stack' or bool(platform.get('artifacts'))))
        green = green and valid
    receipt = {'contract': pin, 'result': 'green' if green else 'red',
               'reason': 'Pinned soft3 qualification passed.' if green else 'Pinned soft3 build is RED or incomplete.',
               'sources_sha256': checksums['sources.json'],
               'validation_sha256': checksums['release-validation.json']}
    return receipt, sources


def fetch(pin, destination):
    """Asset names locate bytes; the committed SHA256SUMS digest authenticates them."""
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=False)
    endpoint = f'repos/{pin["repository"]}/releases/{pin["release_id"]}'
    release = json.loads(subprocess.check_output(['gh', 'api', endpoint]))
    assets = {a['name']: a for a in release['assets']}
    for name in ('SHA256SUMS', *FILES):
        asset = assets.get(name)
        if not asset:
            raise ValueError(f'pinned soft3 build has no {name}')
        raw = subprocess.check_output(['gh', 'api',
            f'repos/{pin["repository"]}/releases/assets/{asset["id"]}',
            '-H', 'Accept: application/octet-stream'])
        (destination / name).write_bytes(raw)
        if name == 'SHA256SUMS' and sha256(destination / name) != pin['checksums_sha256']:
            raise ValueError('soft3 SHA256SUMS differs from the pinned build')
    return verify(destination, pin)


def stack_sources(sources):
    """Old builds captured the entire train; downstream products are not SDK inputs."""
    return [r for r in sources['repositories'] if r['name'] not in {'cyber', 'cyb'}]


def bound_sources(product, pinned):
    if stack_sources(product) != stack_sources(pinned):
        raise ValueError('product component inputs differ from its pinned soft3 build')
