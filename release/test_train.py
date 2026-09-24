"""Release-integrity tests; synthetic receipts never masquerade as platform builds."""
import argparse
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import train
from train_sources import TARGETS, digest, resolve, write_json
from train_gates import Gates


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.sources = {"component": "cyber", "candidate": "candidate-20260924.1", "manager_revision": "a" * 40,
                        "repositories": [{"name": "cyber", "revision": "b" * 40}]}
        self.snapshot = self.root / "snapshot.json"
        write_json(self.snapshot, self.sources)
        self.artifacts = self.root / "artifacts"
        self.artifacts.mkdir()

    def receipt(self, target, *, gate="green", inventory=True, corrupt=False, snapshot_hash=None):
        directory = self.root / target
        directory.mkdir()
        write_json(directory / "sources.json", self.sources)
        if inventory:
            write_json(directory / "soft3-dependencies.json", self.sources)
            (directory / "soft3-dependencies.md").write_text("synthetic fixture\n")
        (directory / "cyber").write_bytes(b"synthetic executable fixture")
        write_json(directory / "candidate.json", {
            "name": self.sources["candidate"], "component": "cyber", "target": target,
            "versions": {"cyber": "0.8.0", "soft3": "0.10.0", "cyb": "0.15.1"},
            "source_revisions": {"cyber": "b" * 40},
            "source_snapshot_sha256": snapshot_hash or digest(self.snapshot),
            "artifacts": [{"name": "cyber", "sha256": digest(directory / "cyber")}]})
        write_json(directory / "release-validation.json", {"result": "green", "complete": True,
                   "gates": [{"name": "meaningful-gate", "result": gate}]})
        train.checksums(directory)
        if corrupt:
            (directory / "cyber").write_bytes(b"changed after qualification")
        archive = self.artifacts / f"cyber-{self.sources['candidate']}-{target}.tar.gz"
        with tarfile.open(archive, "w:gz") as tar:
            for path in directory.iterdir():
                tar.add(path, arcname=path.name)
        return archive

    def collect(self):
        output = self.root / "assembled"
        train.collect(argparse.Namespace(snapshot=self.snapshot, artifacts=self.artifacts, output=output))
        return json.loads((output / "release-validation.json").read_text())

    def test_complete_set_collects(self):
        for target in ["stack", *TARGETS]:
            self.receipt(target)
        self.assertEqual(self.collect()["result"], "green")

    def test_missing_platform_is_red(self):
        self.receipt("stack")
        result = self.collect()
        self.assertEqual(result["result"], "red")
        self.assertEqual(len(result["platforms"]), 5)

    def test_red_gate_cannot_be_hidden_by_green_summary(self):
        for target in ["stack", *TARGETS]:
            self.receipt(target, gate="red" if target == "stack" else "green")
        self.assertEqual(self.collect()["result"], "red")

    def test_missing_inventory_is_red(self):
        self.receipt("stack", inventory=False)
        self.assertIn("inventory missing", self.collect()["platforms"][0]["error"])

    def test_modified_binary_is_red(self):
        self.receipt("stack", corrupt=True)
        self.assertIn("binary checksum", self.collect()["platforms"][0]["error"])

    def test_mixed_source_sets_are_red(self):
        self.receipt("stack", snapshot_hash="wrong")
        self.assertIn("source inventory differs", self.collect()["platforms"][0]["error"])

    def test_archive_traversal_rejected(self):
        archive = self.receipt("stack")
        with tarfile.open(archive, "w:gz") as tar:
            member = tarfile.TarInfo("../escape")
            member.size = 1
            tar.addfile(member, io.BytesIO(b"x"))
        self.assertIn("unsafe archive", self.collect()["platforms"][0]["error"])

    def test_origin_lookup_failure_is_explicit(self):
        with patch("train_sources.command", side_effect=RuntimeError("missing remote")):
            source = resolve({"name": "nu", "repo": "cyberia-to/nu"})
        self.assertFalse(source["available"])
        self.assertIn("missing remote", source["error"])

    def test_origin_pin_drift_is_red(self):
        with patch("train_sources.command", return_value="ref: refs/heads/main\tHEAD\n" + "a" * 40 + "\tHEAD"):
            source = resolve({"name": "test", "repo": "cyberia-to/test", "rev": "b" * 40})
        self.assertFalse(source["pin_matches"])

    def test_warning_is_red_even_with_zero_exit(self):
        gates = Gates(self.root, self.sources, "stack")
        self.assertFalse(gates.run("warning", ["python3", "-c", "print('warning: fixture')"], self.root))

    def test_candidate_identifier_cannot_be_a_version_tag(self):
        for name in ["v0.8.0", "candidate-20269999.1", "../escape", "candidate-20260924.0"]:
            with self.assertRaises(ValueError):
                train.candidate_name(name)

    def test_draft_operation_never_promotes_or_pushes(self):
        write_json(self.root / "candidate.json", {"name": self.sources["candidate"], "component": "cyber",
                   "result": "red", "source_revisions": {"cyber": "b" * 40}})
        with patch("train.command", return_value="draft URL") as call:
            train.draft(argparse.Namespace(output=self.root))
        argv = call.call_args.args[0]
        self.assertEqual(argv[:3], ["gh", "release", "create"])
        self.assertIn("--draft", argv)
        self.assertNotIn("--draft=false", argv)
        self.assertNotIn("push", argv)


if __name__ == "__main__":
    unittest.main()
