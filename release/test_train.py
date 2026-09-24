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
from train_gates import Gates, run_gates, source_gates


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.sources = {"component": "soft3", "candidate": "candidate-20260924.1", "manager_revision": "a" * 40,
                        "repositories": [{"name": "soft3", "revision": "b" * 40}]}
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
            "name": self.sources["candidate"], "component": "soft3", "target": target,
            "versions": {"cyber": "0.8.0", "soft3": "0.10.0", "cyb": "0.15.1"},
            "source_revisions": {"soft3": "b" * 40},
            "source_snapshot_sha256": snapshot_hash or digest(self.snapshot),
            "artifacts": [{"name": "cyber", "sha256": digest(directory / "cyber")}]})
        write_json(directory / "release-validation.json", {"result": "green", "complete": True,
                   "gates": [{"name": "meaningful-gate", "result": gate}]})
        train.checksums(directory)
        if corrupt:
            (directory / "cyber").write_bytes(b"changed after qualification")
        archive = self.artifacts / f"soft3-{self.sources['candidate']}-{target}.tar.gz"
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

    def test_product_dependency_matches_build_not_current_manager_head(self):
        (self.root / "cyber/release").mkdir(parents=True)
        (self.root / "soft3/crate").mkdir(parents=True)
        (self.root / "soft3/crate/Cargo.toml").write_text('[package]\nversion = "0.10.0"\n')
        pin = {"repository": "cyberia-to/soft3", "version": "0.10.0", "revision": "c" * 40,
               "build": "candidate-20260924.1", "release_id": 42, "checksums_sha256": "d" * 64}
        body = '[soft3]\n' + ''.join(f'{key} = {json.dumps(value)}\n' for key, value in pin.items())
        (self.root / "cyber/release/soft3.toml").write_text(body)
        self.sources["repositories"] = [{"name": "soft3", "revision": pin["revision"]}]
        self.sources["soft3_build"] = {"contract": pin, "result": "red", "reason": "Stack failed"}
        self.sources["phase1_sha256"] = "fixture"
        gates = Gates(self.root, self.sources, "stack")
        source_gates(gates, self.root, self.sources, [], "cyber")
        self.assertEqual(gates.rows[-2]["name"], "soft3-dependency")
        self.assertEqual(gates.rows[-2]["result"], "green")
        self.assertEqual(gates.rows[-1]["result"], "red")
        self.sources["soft3_build"]["contract"] = {**pin, "checksums_sha256": "e" * 64}
        source_gates(gates, self.root, self.sources, [], "cyber")
        self.assertEqual(gates.rows[-2]["result"], "red")

    def test_warning_is_red_even_with_zero_exit(self):
        gates = Gates(self.root, self.sources, "stack")
        self.assertFalse(gates.run("warning", ["python3", "-c", "print('warning: fixture')"], self.root))

    def test_nu_tests_are_blocked_when_their_shell_cannot_build(self):
        (self.root / "soft3").mkdir()
        with patch("train_gates.source_gates"), patch("train_gates.boot_status"), \
                patch.object(Gates, "run", return_value=False) as run:
            run_gates(self.root, self.root, self.sources, [], "soft3", "stack", stack=True)
        names = [call.args[0] for call in run.call_args_list]
        self.assertIn("stack-nu-build", names)
        self.assertNotIn("stack-nu", names)
        receipt = json.loads((self.root / "release-validation.json").read_text())
        self.assertEqual(receipt["result"], "red")
        self.assertEqual(receipt["gates"][-1]["name"], "stack-nu")
        self.assertEqual(receipt["gates"][-1]["result"], "blocked")

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
