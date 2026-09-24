#!/usr/bin/env python3
"""Read-only source inventory for the neuron/cell convergence roadmap.

Find repositories independently of ignore rules, including nested checkouts and
worktrees. Search working-tree files listed by Git, plus source trees outside Git.
Outputs contain paths, counts and matching terms/line numbers, never source lines.
"""
from __future__ import annotations

import argparse
import collections
import concurrent.futures
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

PRUNE = {'.git', 'target', 'node_modules', '.venv', 'venv', '__pycache__',
         '.cache', '.next', '.nuxt', '.gradle', '.mypy_cache', '.pytest_cache',
         '.ruff_cache', '.tox'}
SENSITIVE_NAMES = {'mnemonic', 'mnemonic.txt', 'seed.txt', 'credentials.json',
                   'secrets.json', 'token.json', 'id_rsa', 'id_ed25519'}
PATTERNS = {
    'cell': re.compile(rb'(?i)cell'),
    'neuron': re.compile(rb'(?i)neuron'),
    'domain': re.compile(rb'(?i)(?:cell[_ -]?id|runtime[-_ ]cell|cell[-_](?:model|engine|node|rune|cli)|'
                         rb'cell::Cell|cyb_core::Cell|SharedCell|run_cell|cell/(?:birth|commit|snapshot|event)|'
                         rb'cell://|(?:ledger|knowledge|building)[-_ ]cell|cells as organs)'),
    'identity': re.compile(rb'(?i)(?:NeuronId|neuron_id|neuron_of|neuron[_ ]identity|'
                           rb'neuron.{0,30}(?:key|sign|author|chain)|(?:key|sign|author).{0,30}neuron)'),
    'dependency': re.compile(rb'(?:cell-(?:model|engine|node|rune|cli)|[./]cell["/]|cyber-cell)'),
}
MAX_BYTES = 32 * 1024 * 1024


def git(path, *args, timeout=60):
    env = dict(os.environ, GIT_OPTIONAL_LOCKS='0', GIT_NO_LAZY_FETCH='1')
    proc = subprocess.run(['git', '-C', str(path), *args], stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, env=env, timeout=timeout)
    if proc.returncode:
        raise RuntimeError(proc.stderr.decode(errors='replace').strip()[:400])
    return proc.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, default=Path.home() / 'cyber')
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    root, out = args.root.resolve(), args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    repos, nonrepo, aliases, skipped_dirs, discovery_errors = {}, collections.defaultdict(list), [], [], []
    owners = {}

    def onerror(error):
        discovery_errors.append({'path': str(error.filename), 'error': str(error)})

    for directory, dirs, files in os.walk(root, followlinks=False, onerror=onerror):
        p = Path(directory)
        rel = p.relative_to(root).as_posix()
        if p == out:
            dirs[:] = []
            continue
        if '.git' in dirs or '.git' in files:
            repos[p] = {'path': rel, 'kind': 'worktree' if (p/'.git').is_file() else 'repository'}
            owners[p] = p
        elif {'HEAD', 'config'}.issubset(files) and {'objects', 'refs'}.issubset(dirs):
            repos[p] = {'path': rel, 'kind': 'bare'}
            owners[p] = p
            dirs[:] = []
            continue
        else:
            owners[p] = owners.get(p.parent)
        kept = []
        for name in dirs:
            child = p/name
            if child.is_symlink():
                target = child.resolve()
                aliases.append({'path': child.relative_to(root).as_posix(),
                                'target': str(target), 'inside_root': target.is_relative_to(root)})
            elif name in PRUNE:
                if name != '.git':
                    skipped_dirs.append(child.relative_to(root).as_posix())
            else:
                kept.append(name)
        dirs[:] = kept
        if owners[p] is None:
            group = p.relative_to(root).parts[0] if p != root else '.'
            for name in files:
                nonrepo[group].append(p/name)

    for path, row in repos.items():
        if row['kind'] == 'bare':
            continue
        try:
            actual = Path(git(path, 'rev-parse', '--show-toplevel').decode().strip()).resolve()
            if actual != path:
                row['kind'] = 'generated-git-marker'
                row['actual_git_root'] = str(actual)
        except Exception as error:
            row['kind'] = 'broken-git-checkout'
            row['git_metadata_error'] = str(error)

    repo_paths = set(repos)
    owner_cache = {}

    def nested_owner(path):
        parent = path.parent
        if parent not in owner_cache:
            found = None
            for ancestor in [parent, *parent.parents]:
                if ancestor in repo_paths:
                    found = ancestor
                    break
                if ancestor == root:
                    break
            owner_cache[parent] = found
        return owner_cache[parent]

    def add_data(path, data, origin, counts, hits):
        if b'\0' in data:
            counts['binary_files_excluded'] += 1
            return
        counts['text_files_scanned'] += 1
        counts['text_bytes_scanned'] += len(data)
        present = [k for k, regex in PATTERNS.items() if regex.search(data)]
        if not present:
            return
        categories = {}
        lines = data.splitlines()
        for key in present:
            locations = [i for i, line in enumerate(lines, 1) if PATTERNS[key].search(line)]
            categories[key] = {'matching_lines': len(locations), 'first_lines': locations[:16]}
            counts[f'{key}_files'] += 1
            counts[f'{key}_lines'] += len(locations)
        hits.append({'path': path.relative_to(root).as_posix(), 'bytes': len(data),
                     'source': origin, 'sha256': hashlib.sha256(data).hexdigest(), 'matches': categories})

    def excluded(path):
        name = path.name.lower()
        if name.startswith('.env') or name in SENSITIVE_NAMES or path.suffix.lower() in {'.pem', '.key', '.p12', '.pfx'}:
            return 'credential_files_excluded'
        if any(part in PRUNE for part in path.relative_to(root).parts):
            return 'build_or_metadata_files_excluded'
        return None

    def search_files(paths, repo_root):
        counts = collections.Counter()
        hits, errors = [], []
        for path in sorted(set(paths)):
            counts['listed_files'] += 1
            relative = path.relative_to(root).as_posix()
            if path.is_relative_to(out):
                counts['audit_output_excluded'] += 1
                continue
            if repo_root is not None and nested_owner(path) != repo_root:
                counts['nested_repo_files_excluded'] += 1
                continue
            if path.is_symlink():
                counts['symlink_files_excluded'] += 1
                continue
            if not path.is_file():
                counts['missing_or_gitlink'] += 1
                continue
            exclusion = excluded(path)
            if exclusion:
                counts[exclusion] += 1
                continue
            try:
                size = path.stat().st_size
                with path.open('rb') as stream:
                    prefix = stream.read(4096)
                if b'\0' in prefix:
                    counts['binary_files_excluded'] += 1
                    continue
                if size > MAX_BYTES:
                    counts['large_files_excluded'] += 1
                    errors.append({'path': relative, 'reason': 'over_32_MiB', 'bytes': size})
                    continue
                data = path.read_bytes()
                add_data(path, data, 'working-tree', counts, hits)
            except OSError as error:
                counts['read_errors'] += 1
                errors.append({'path': relative, 'reason': str(error)})
        return dict(counts), hits, errors

    def source_walk(path):
        found = []
        for directory, dirs, files in os.walk(path, followlinks=False):
            dirs[:] = [d for d in dirs if d not in PRUNE and not (Path(directory)/d).is_symlink()]
            found.extend(Path(directory)/f for f in files)
        return found

    def scan_sparse_index(path, row):
        entries = git(path, 'ls-files', '-v', '-z').split(b'\0')
        missing = {entry[2:] for entry in entries if entry.startswith(b'S ') and not (path/os.fsdecode(entry[2:])).exists()}
        if not missing:
            return
        staged = git(path, 'ls-files', '--stage', '-z').split(b'\0')
        objects = []
        for entry in staged:
            if not entry:
                continue
            meta, name = entry.split(b'\t', 1)
            mode, oid, stage = meta.split()
            if name not in missing or stage != b'0' or mode not in {b'100644', b'100755'}:
                continue
            file = path/os.fsdecode(name)
            exclusion = excluded(file)
            if exclusion:
                row['counts'][exclusion] = row['counts'].get(exclusion, 0) + 1
                continue
            objects.append((file, oid))
        if not objects:
            return
        env = dict(os.environ, GIT_OPTIONAL_LOCKS='0', GIT_NO_LAZY_FETCH='1')
        meta = subprocess.run(['git', '-C', str(path), 'cat-file', '--batch-check'],
                              input=b''.join(oid+b'\n' for _, oid in objects),
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, timeout=90)
        checks = meta.stdout.splitlines()
        if meta.returncode or len(checks) != len(objects):
            raise RuntimeError('cannot inspect sparse index objects locally')
        counts = collections.Counter(row['counts'])
        with subprocess.Popen(['git', '-C', str(path), 'cat-file', '--batch'],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              env=env) as batch:
            for (file, oid), check in zip(objects, checks):
                fields = check.split()
                if len(fields) != 3 or fields[1] != b'blob':
                    row['errors'].append({'path': file.relative_to(root).as_posix(), 'reason': 'sparse_index_blob_unavailable'})
                    continue
                size = int(fields[2])
                if size > MAX_BYTES:
                    counts['large_index_blobs_excluded'] += 1
                    continue
                batch.stdin.write(oid+b'\n')
                batch.stdin.flush()
                header = batch.stdout.readline().split()
                if len(header) != 3 or header[1] != b'blob' or int(header[2]) != size:
                    raise RuntimeError('unexpected Git batch object header')
                data = batch.stdout.read(size)
                if len(data) != size or batch.stdout.read(1) != b'\n':
                    raise RuntimeError('truncated Git batch object')
                counts['sparse_index_blobs_read'] += 1
                add_data(file, data, 'sparse-index-blob', counts, row['hits'])
            batch.stdin.close()
            batch.wait(timeout=30)
        row['counts'] = dict(counts)
        row['coverage'] += '; absent sparse paths scanned from local index blobs'

    def inspect_repo(item):
        path, row = item
        row = dict(row)
        if row['kind'] in {'generated-git-marker', 'broken-git-checkout'}:
            row['counts'], row['hits'], row['errors'] = search_files(source_walk(path), path)
            row['coverage'] = 'filesystem fallback; Git metadata is not a valid repository root'
            return row
        if row['kind'] == 'bare':
            row['coverage'] = 'metadata-only; no working tree'
            row['counts'], row['hits'], row['errors'] = {}, [], []
            try:
                row['head'] = git(path, 'rev-parse', 'HEAD').decode().strip()
            except Exception as error:
                row['errors'].append({'reason': str(error)})
            return row
        try:
            try:
                row['head'] = git(path, 'rev-parse', 'HEAD').decode().strip()
            except RuntimeError:
                row['head'] = 'unborn'
            try:
                row['branch'] = git(path, 'symbolic-ref', '--quiet', '--short', 'HEAD').decode().strip()
            except RuntimeError:
                row['branch'] = '(detached)'
            status = git(path, 'status', '--porcelain=v1', '-z', '--untracked-files=normal')
            row['dirty'] = bool(status)
            row['status_entries'] = len([x for x in status.split(b'\0') if x])
            listed = git(path, 'ls-files', '-z', '--cached', '--others', '--exclude-standard')
            files = [path/os.fsdecode(f) for f in listed.split(b'\0') if f]
            row['counts'], row['hits'], row['errors'] = search_files(files, path)
            row['coverage'] = 'tracked + nonignored untracked working-tree files'
            scan_sparse_index(path, row)
        except Exception as error:
            row.setdefault('counts', {})
            row.setdefault('hits', [])
            row.setdefault('errors', []).append({'reason': str(error)})
            row['coverage'] = 'failed'
        return row

    rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as executor:
        for row in executor.map(inspect_repo, sorted(repos.items())):
            rows.append(row)
            print(f"scanned {row['path']}: {row['counts'].get('text_files_scanned', 0)} text files", flush=True)
    sources = []
    for group, files in sorted(nonrepo.items()):
        counts, hits, errors = search_files(files, None)
        sources.append({'path': group, 'kind': 'non-git-source-tree', 'counts': counts,
                        'hits': hits, 'errors': errors, 'coverage': 'enumerated files outside discovered repositories'})
    finished = datetime.datetime.now(datetime.timezone.utc).isoformat()
    totals = collections.Counter()
    for row in rows + sources:
        totals.update(row['counts'])
    result = {'root': str(root), 'started_utc': started, 'finished_utc': finished,
              'scanner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'repositories': rows, 'non_git_sources': sources,
              'aliases': aliases, 'pruned_directories': skipped_dirs,
              'prune_names': sorted(PRUNE), 'discovery_errors': discovery_errors,
              'totals': dict(totals), 'max_file_bytes': MAX_BYTES}
    (out/'scan.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    flat = [{'repository': row['path'], 'kind': row['kind'], **hit}
            for row in rows + sources for hit in row['hits']]
    with (out/'matches.jsonl').open('w') as stream:
        for hit in flat:
            stream.write(json.dumps(hit, ensure_ascii=False) + '\n')
    print(json.dumps({'repositories': len(rows), 'non_git_sources': len(sources),
                      'totals': dict(totals), 'discovery_errors': len(discovery_errors),
                      'repository_errors': sum(len(x['errors']) for x in rows)}, indent=2))


if __name__ == '__main__':
    main()
