"""Pussy stand tests: the pinned file is well formed, round-trips as an override, and names no typo."""
from pathlib import Path
import tempfile
import tomllib
import unittest

import pussy_stand
from train_sources import parse_overrides

HERE = Path(__file__).parent
EXTRA_NAMES = {"soft3"}  # not a phase1.toml sibling; the manager repo itself


class PussyStandTests(unittest.TestCase):
    def broken(self, *entries):
        text = "".join(f'[[sibling]]\nname = "{n}"\nrepo = "{r}"\nrev = "{v}"\n' for n, r, v in entries)
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        path = Path(tmp.name) / "stand.toml"
        path.write_text(text)
        return path

    def test_round_trip(self):
        stand = pussy_stand.load(pussy_stand.DEFAULT)
        expected = {entry["name"]: entry["rev"] for entry in stand["sibling"]}
        parsed = parse_overrides(pussy_stand.override(stand).split(), set(expected))
        self.assertEqual(parsed, expected)

    def test_invalid_stands(self):
        good = "a" * 40
        for entries in (
            [("x", "cyberia-to/x", "a" * 7)],
            [("x", "cyberia-to/x", good), ("x", "cyberia-to/y", good)],
            [("x", "evil/x", good)],
            [],
        ):
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                pussy_stand.load(self.broken(*entries))

    def test_names_are_known(self):
        phase1 = {entry["name"]: entry["repo"] for entry in tomllib.loads((HERE / "phase1.toml").read_text())["sibling"]}
        stand = pussy_stand.load(pussy_stand.DEFAULT)["sibling"]
        self.assertEqual({entry["name"] for entry in stand} - set(phase1), EXTRA_NAMES)
        for entry in stand:
            if entry["name"] in phase1:
                self.assertEqual(entry["repo"], phase1[entry["name"]], entry["name"])


if __name__ == "__main__":
    unittest.main()
