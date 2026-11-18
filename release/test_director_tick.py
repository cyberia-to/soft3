"""The director's tick reads every pinned sibling and hides no red gate."""
import json
from pathlib import Path
import tempfile
import unittest

import director_tick

MANIFEST = Path(__file__).resolve().parent / "phase1.toml"


class DirectorTickTests(unittest.TestCase):
    def test_committed_manifest_senses_every_pinned_sibling(self):
        sensed, unpinned = director_tick.siblings(MANIFEST)
        entries = json.loads(json.dumps(sensed))
        declared = sum(1 for line in MANIFEST.read_text().splitlines() if line.strip() == "[[sibling]]")
        self.assertEqual(len(entries) + len(unpinned), declared)
        self.assertTrue(all(row["repo"] and row["branch"] and len(row["rev"]) == 40 for row in entries))
        self.assertIn("cyberia-to/optica", [row["repo"] for row in entries])

    def test_sibling_without_a_pin_is_reported_not_dropped(self):
        sensed, unpinned = director_tick.siblings(MANIFEST)
        self.assertEqual(unpinned, ["nu"])
        self.assertNotIn("cyberia-to/nu", [row["repo"] for row in sensed])

    def test_sibling_without_a_pin_does_not_stop_the_read(self):
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "phase1.toml"
            manifest.write_text(
                '[[sibling]]\nname = "nu"\nrepo = "cyberia-to/nu"\nnote = "no remote yet"\n\n'
                '[[sibling]]\nname = "evy"\nrepo = "cyberia-to/evy"\nbranch = "main"\nrev = "%s"\n' % ("2" * 40))
            sensed, unpinned = director_tick.siblings(manifest)
        self.assertEqual(unpinned, ["nu"])
        self.assertEqual(sensed, [{"repo": "cyberia-to/evy", "branch": "main", "rev": "2" * 40}])

    def test_reduction_keeps_every_gate_that_is_not_green(self):
        rows = director_tick.reduce_receipts([
            {"result": "red", "gates": [{"name": "phase1-pins", "result": "green"},
                                        {"name": "soft3-tests", "result": "red", "detail": "two tests failed"},
                                        {"name": "simulation-gate", "result": "blocked", "reason": "no checkout"}]},
            {"result": "green", "gates": [{"name": "node-status", "result": "green"}]}])
        self.assertEqual(rows[0]["result"], "red")
        self.assertEqual([gate["name"] for gate in rows[0]["red"]], ["soft3-tests", "simulation-gate"])
        self.assertEqual(rows[0]["red"][0]["detail"], "two tests failed")
        self.assertEqual(rows[0]["red"][1]["detail"], "no checkout")
        self.assertEqual(rows[1], {"result": "green", "red": []})

    def test_a_receipt_without_gates_reduces_to_no_reds(self):
        self.assertEqual(director_tick.reduce_receipts([{"result": "green"}]), [{"result": "green", "red": []}])

    def test_reducing_nothing_is_an_error_not_an_empty_file(self):
        with self.assertRaises(ValueError):
            director_tick.main(["receipts"])


if __name__ == "__main__":
    unittest.main()
