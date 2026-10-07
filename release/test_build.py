"""Pinned build isolation and integrity. All receipts here are synthetic."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import train

from train_build import FILES, TARGETS, bound_sources, contract, fetch, stack_sources, verify
from train_sources import component_bindings, resolve, snapshot


class BuildTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.pin = {'repository': 'cyberia-to/soft3', 'version': '0.10.0', 'revision': 'a' * 40,
                    'release_id': 42, 'build': 'candidate-20260924.1'}
        self.sources = {'component': 'soft3', 'candidate': self.pin['build'], 'phase1_sha256': 'fixture',
                        'repositories': [{'name': 'soft3', 'revision': 'a' * 40},
                                         {'name': 'nu', 'revision': 'b' * 40}]}
        self.candidate = {'component': 'soft3', 'name': self.pin['build'], 'result': 'green',
                          'versions': {'soft3': '0.10.0'}, 'source_revisions': {'soft3': 'a' * 40, 'nu': 'b' * 40}}
        self.validation = {'result': 'green', 'platforms': [
            {'target': target, 'result': 'green', 'archive': target + '.tar.gz', 'archive_sha256': 'd' * 64,
             'artifacts': [{'name': 'synthetic-binary'}], 'gates': [{'name': 'real-gate', 'result': 'green'}]}
            for target in sorted(TARGETS)]}
        self.seal()

    def seal(self):
        (self.root / 'sources.json').write_text(json.dumps(self.sources))
        self.validation['source_snapshot_sha256'] = hashlib.sha256((self.root / 'sources.json').read_bytes()).hexdigest()
        (self.root / 'candidate.json').write_text(json.dumps(self.candidate))
        (self.root / 'release-validation.json').write_text(json.dumps(self.validation))
        (self.root / 'soft3-dependencies.json').write_text(json.dumps({'source_snapshot': self.sources}))
        lines = [f'{hashlib.sha256((self.root / name).read_bytes()).hexdigest()}  {name}' for name in FILES]
        lines += ['d' * 64 + '  ' + target + '.tar.gz' for target in sorted(TARGETS)]
        (self.root / 'SHA256SUMS').write_text('\n'.join(lines) + '\n')
        self.pin['checksums_sha256'] = hashlib.sha256((self.root / 'SHA256SUMS').read_bytes()).hexdigest()

    def collect_product(self):
        proof = self.root / 'soft3-build'
        proof.mkdir()
        for name in ('SHA256SUMS', *FILES):
            shutil.copy2(self.root / name, proof / name)
        inherited, upstream = verify(proof, self.pin)
        sources = {**upstream, 'component': 'cyber', 'candidate': 'candidate-20260925.1',
                   'manager_revision': 'e' * 40, 'soft3_build': inherited,
                   'repositories': [*upstream['repositories'], {'name': 'cyber', 'revision': 'c' * 40}]}
        snapshot_file = self.root / 'product.json'
        snapshot_file.write_text(json.dumps(sources))
        artifacts = self.root / 'artifacts'
        artifacts.mkdir()
        revisions = {r['name']: r.get('revision') for r in sources['repositories']}
        for target in TARGETS - {'stack'}:
            work = self.root / target
            work.mkdir()
            (work / 'cyber').write_bytes(b'synthetic product binary')
            files = {
                'candidate.json': {'name': sources['candidate'], 'component': 'cyber', 'target': target,
                    'versions': {'cyber': '0.8.0', 'soft3': '0.10.0'}, 'source_revisions': revisions,
                    'source_snapshot_sha256': train.digest(snapshot_file),
                    'artifacts': [{'name': 'cyber', 'sha256': train.digest(work / 'cyber')}]},
                'sources.json': sources, 'soft3-dependencies.json': sources,
                'release-validation.json': {'result': 'green', 'complete': True,
                    'gates': [{'name': 'product-check', 'result': 'green'}]}}
            for name, value in files.items():
                (work / name).write_text(json.dumps(value))
            (work / 'soft3-dependencies.md').write_text('synthetic inventory\n')
            train.checksums(work)
            with tarfile.open(artifacts / f"cyber-{sources['candidate']}-{target}.tar.gz", 'w:gz') as tar:
                for path in work.iterdir(): tar.add(path, arcname=path.name)
        output = self.root / 'collected'
        train.collect(argparse.Namespace(snapshot=snapshot_file, artifacts=artifacts, output=output))
        return json.loads((output / 'candidate.json').read_text()), json.loads((output / 'release-validation.json').read_text())

    def test_product_collect_consumes_stack_without_rebuilding_it(self):
        candidate, validation = self.collect_product()
        self.assertEqual(candidate['result'], 'green')
        self.assertEqual(candidate['soft3_build']['contract'], self.pin)
        self.assertTrue(validation['platforms'][0]['inherited'])
        self.assertEqual(len(validation['platforms']), 5)

    def test_green_product_cannot_overrule_failed_stack(self):
        self.validation['platforms'][0]['gates'][0]['result'] = 'red'
        self.seal()
        candidate, validation = self.collect_product()
        self.assertEqual(candidate['result'], 'red')
        self.assertEqual(validation['platforms'][0]['result'], 'red')
        self.assertTrue(all(p['result'] == 'green' for p in validation['platforms'][1:]))

    def test_stack_crates_cannot_escape_into_registry_or_git_resolution(self):
        repositories = [{'name': 'hemera', 'declared_packages': [{'name': 'cyber-hemera'}]}]
        for source in ['registry+https://github.com/rust-lang/crates.io-index', 'git+https://github.com/cyberia-to/hemera?rev=other']:
            metadata = {'packages': [{'name': 'cyber-hemera', 'version': '0.3.1', 'source': source}]}
            self.assertEqual(component_bindings(metadata, repositories)['result'], 'red')
        metadata = {'packages': [{'name': 'cyber-hemera', 'version': '0.3.1', 'source': None},
                                {'name': 'serde', 'version': '1', 'source': 'registry+example'}]}
        self.assertEqual(component_bindings(metadata, repositories)['result'], 'green')

    def test_complete_pinned_build_verifies(self):
        self.assertEqual(verify(self.root, self.pin)[0]['result'], 'green')

    def test_replaced_checksums_manifest_rejected(self):
        (self.root / 'SHA256SUMS').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'pinned build'):
            verify(self.root, self.pin)

    def test_replaced_sources_rejected(self):
        (self.root / 'sources.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
            verify(self.root, self.pin)

    def test_wrong_build_identity_rejected_even_with_valid_checksums(self):
        self.candidate['name'] = 'candidate-20260925.1'
        self.seal()
        with self.assertRaisesRegex(ValueError, 'identity'):
            verify(self.root, self.pin)

    def test_red_gate_cannot_be_hidden_by_green_upstream_summary(self):
        self.validation['platforms'][0]['gates'][0]['result'] = 'red'
        self.seal()
        self.assertEqual(verify(self.root, self.pin)[0]['result'], 'red')

    def test_missing_platform_is_red(self):
        self.validation['platforms'].pop()
        self.seal()
        self.assertEqual(verify(self.root, self.pin)[0]['result'], 'red')

    def test_changed_component_revision_rejected(self):
        downstream = copy.deepcopy(self.sources)
        downstream['repositories'][1]['revision'] = 'c' * 40
        with self.assertRaisesRegex(ValueError, 'component inputs differ'):
            bound_sources(downstream, self.sources)

    def test_old_train_members_are_excluded_from_stack_inputs(self):
        self.sources['repositories'] += [{'name': 'cyber'}, {'name': 'cyb'}]
        self.assertEqual([r['name'] for r in stack_sources(self.sources)], ['soft3', 'nu'])

    def test_soft3_snapshot_excludes_downstream_product_heads(self):
        (self.root / 'release').mkdir()
        (self.root / 'release/phase1.toml').write_text('[[sibling]]\nname="nu"\nrepo="nushell/nushell"\n')
        def command(args, **kwargs):
            if args[:3] == ['git', 'status', '--porcelain']: return ''
            if args[:2] == ['git', 'rev-parse']: return 'a' * 40
            return '[{"title":"fixture","html_url":"https://example.com/pr"}]'
        def remote(row): return {**row, 'available': True, 'revision': 'a' * 40}
        with patch('train_sources.command', side_effect=command), patch('train_sources.resolve', side_effect=remote) as resolve_source:
            output = snapshot(self.root, self.root / 'snapshot', 'soft3', self.pin['build'])
        self.assertEqual({call.args[0]['name'] for call in resolve_source.call_args_list}, {'soft3', 'nu'})
        self.assertEqual({r['name'] for r in output['repositories']}, {'soft3', 'nu'})

    def test_product_snapshot_resolves_only_product_origin(self):
        import base64
        pin_text = '[soft3]\n' + ''.join(f'{key} = {json.dumps(value)}\n' for key, value in self.pin.items())
        product = {'name': 'cyber', 'repo': 'cyberia-to/cyber', 'revision': 'c' * 40, 'available': True}
        def command(args, **kwargs):
            if args[:3] == ['git', 'status', '--porcelain']: return ''
            if args[:2] == ['git', 'rev-parse']: return 'e' * 40
            if 'contents/release/soft3.toml' in args[2]: return base64.b64encode(pin_text.encode()).decode()
            return '[{"title":"fixture","html_url":"https://example.com/pr"}]'
        with patch('train_sources.command', side_effect=command), patch('train_sources.resolve', return_value=product) as remote, patch('train_sources.fetch_build', return_value=verify(self.root, self.pin)):
            output = snapshot(self.root, self.root / 'snapshot', 'cyber', 'candidate-20260925.1')
        self.assertEqual(remote.call_count, 1)
        self.assertEqual(remote.call_args.args[0]['name'], 'cyber')
        self.assertEqual(output['soft3_build']['contract'], self.pin)
        self.assertEqual({r['name'] for r in output['repositories']}, {'soft3', 'nu', 'cyber'})

    def test_upstream_pin_is_stable_when_default_head_advances(self):
        with patch('train_sources.command', side_effect=['ref: refs/heads/main\tHEAD\n' + 'f' * 40 + '\tHEAD', 'ahead']):
            source = resolve({'name': 'nu', 'repo': 'nushell/nushell', 'branch': 'main', 'rev': 'b' * 40, 'source': 'upstream'})
        self.assertEqual(source['revision'], 'b' * 40)
        self.assertTrue(source['pin_matches'])

    def test_upstream_pin_outside_default_history_is_red(self):
        with patch('train_sources.command', side_effect=['ref: refs/heads/main\tHEAD\n' + 'f' * 40 + '\tHEAD', 'diverged']):
            source = resolve({'name': 'nu', 'repo': 'nushell/nushell', 'branch': 'main', 'rev': 'b' * 40, 'source': 'upstream'})
        self.assertFalse(source['available'])

    def test_commit_only_contract_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'checksums_sha256'):
            contract('[soft3]\nrepository="cyberia-to/soft3"\nrevision="' + 'a' * 40 + '"')


if __name__ == '__main__':
    unittest.main()
