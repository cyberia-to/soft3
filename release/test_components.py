"""Declaration provenance tests: no inferred binary closure or borrowed verdicts."""
import unittest

from train_components import trace


class ComponentTests(unittest.TestCase):
    def setUp(self):
        self.sources = {'component': 'cyber', 'candidate': 'fixture', 'repositories': [
            {'name': n, 'repo': f'example/{n}', 'revision': n, 'pin_matches': True}
            for n in ['cyber', 'cyb', 'soft3', 'engine', 'storage', 'helper']]}
        self.manifests = {}

    def manifest(self, path, data):
        self.manifests[path] = {'data': data, 'url': 'https://example.com/' + path, 'sha256': path}

    def components(self):
        return {r['name']: r for r in trace(self.sources, self.manifests)['components']}

    def test_workspace_path_and_version_are_relative_to_workspace(self):
        self.manifest('cyber/Cargo.toml', {'package': {'name': 'cyber'},
                      'dependencies': {'engine': {'path': '../engine/child'}}})
        self.manifest('engine/Cargo.toml', {'workspace': {
            'package': {'version': '1.2.3'}, 'dependencies': {'storage': {'path': '../storage'}}}})
        self.manifest('engine/child/Cargo.toml', {'package': {'name': 'engine', 'version': {'workspace': True}},
                      'dependencies': {'storage': {'workspace': True}}})
        self.manifest('storage/Cargo.toml', {'package': {'name': 'storage', 'version': '2.0.0'}})
        rows = self.components()
        self.assertEqual(rows['engine']['packages'][0]['version'], '1.2.3')
        self.assertEqual(rows['storage']['relations'][0]['to'], 'storage/Cargo.toml')
        self.assertEqual(rows['storage']['relations'][0]['declaration'], 'engine/Cargo.toml')

    def test_registry_ownership_does_not_traverse_captured_version(self):
        self.manifest('cyber/Cargo.toml', {'package': {'name': 'cyber'},
                      'dependencies': {'engine': '1'}})
        self.manifest('engine/Cargo.toml', {'package': {'name': 'engine', 'version': '2'},
                      'dependencies': {'storage': {'path': '../storage'}}})
        rows = self.components()
        self.assertEqual(set(rows), {'engine'})
        self.assertEqual(rows['engine']['relations'][0]['binding'], 'registry')
        self.assertEqual(rows['engine']['relations'][0]['requirement'], '1')
        self.assertFalse(rows['engine']['packages'])

    def test_missing_inputs_and_conditional_test_paths_remain_visible(self):
        self.manifest('cyber/Cargo.toml', {'package': {'name': 'cyber'},
            'dependencies': {'missing': {'path': '../missing'}},
            'dev-dependencies': {'helper': {'path': '../helper', 'optional': True}}})
        self.manifest('helper/Cargo.toml', {'package': {'name': 'helper'},
            'dependencies': {'storage': {'path': '../storage'}},
            'dev-dependencies': {'engine': {'path': '../engine'}}})
        rows = self.components()
        self.assertFalse(rows['missing']['captured'])
        self.assertEqual(rows['missing']['missing_manifests'], ['missing/Cargo.toml'])
        self.assertEqual(rows['storage']['relations'][0]['conditions'], ['optional', 'test'])
        self.assertNotIn('engine', rows)  # Dependency tests do not belong to this product.

    def test_unresolved_workspace_reference_is_explicit(self):
        self.manifest('cyber/Cargo.toml', {'package': {'name': 'cyber'},
                      'dependencies': {'engine': {'workspace': True}}})
        result = trace(self.sources, self.manifests)
        self.assertEqual(result['errors'][0]['dependency'], 'engine')
        self.assertFalse(result['components'])

    def test_soft3_inventory_excludes_downstream_products(self):
        self.sources['component'] = 'soft3'
        self.manifest('soft3/crate/Cargo.toml', {'package': {'name': 'soft3'}})
        self.assertEqual(set(self.components()), {'engine', 'storage', 'helper'})


if __name__ == '__main__':
    unittest.main()
