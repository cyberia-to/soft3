"""Release-integrity tests; synthetic receipts never masquerade as platform builds."""
import argparse
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import train
from train_sources import TARGETS, digest, resolve, write_json
from train_gates import Gates, chain_genesis_gate, conformance_gate, simulation_gate, source_gates


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

    def test_qualifier_revision_must_match_consumed_soft3(self):
        (self.root / "cyber/release").mkdir(parents=True)
        (self.root / "soft3/crate").mkdir(parents=True)
        (self.root / "soft3/crate/Cargo.toml").write_text('[package]\nversion = "0.10.0"\n')
        revision = "c" * 40
        (self.root / "cyber/release/soft3.toml").write_text(
            f'[soft3]\nrepository = "cyberia-to/soft3"\nversion = "0.10.0"\nrevision = "{revision}"\n')
        self.sources["repositories"].append({"name": "soft3", "revision": revision})
        self.sources["phase1_sha256"] = "fixture"
        gates = Gates(self.root, self.sources, "stack")
        source_gates(gates, self.root, self.sources, [], "cyber")
        self.assertEqual(gates.rows[-1]["name"], "soft3-dependency")
        self.assertEqual(gates.rows[-1]["result"], "red")
        self.sources["manager_revision"] = revision
        source_gates(gates, self.root, self.sources, [], "cyber")
        self.assertEqual(gates.rows[-1]["result"], "green")

    def test_warning_is_red_even_with_zero_exit(self):
        gates = Gates(self.root, self.sources, "stack")
        self.assertFalse(gates.run("warning", ["python3", "-c", "print('warning: fixture')"], self.root))

    def test_chain_genesis_without_the_file_is_blocked_not_green(self):
        gates = Gates(self.root / "out", self.sources, "aarch64-apple-darwin")
        with patch.dict("os.environ", {}, clear=False) as env:
            env.pop("SOFT3_GENESIS_DIR", None)
            self.assertFalse(chain_genesis_gate(gates, self.root / "soft3", genesis_dir=self.root / "no-genesis",
                                                pussy=self.root / "pussy"))
        self.assertEqual(gates.rows[-1]["result"], "blocked")

    def pussy_fixture(self, matching=True):
        manifest = self.root / "pussy/snapshot/manifest.json"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('{"fixture": true}\n')
        expected = digest(manifest) if matching else "0" * 64
        spec = self.root / "soft3/conformance/chain-genesis.toml"
        spec.parent.mkdir(parents=True)
        spec.write_text(f'[pussy]\nsource_manifest_sha256 = "{expected}"\n')

    def test_chain_genesis_env_file_is_checked_without_a_build(self):
        genesis = self.root / "genesis/pussy.tade"
        genesis.parent.mkdir()
        genesis.write_bytes(b"x")
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.object(Gates, "run", return_value=True) as run, \
                patch.object(Gates, "run_sequence") as sequence:
            self.assertTrue(chain_genesis_gate(gates, self.root / "soft3", genesis_dir=genesis.parent,
                                               pussy=self.root / "pussy"))
        sequence.assert_not_called()
        name, argv, cwd = run.call_args.args[:3]
        self.assertEqual(name, "stack-chain-genesis")
        self.assertEqual(argv[-5:], ["conformance", "check", "chain", "--genesis", str(genesis)])

    def test_chain_genesis_builds_from_the_pinned_checkout_then_checks(self):
        self.pussy_fixture()
        gates = Gates(self.root / "out", self.sources, "stack")
        soft3 = self.root / "soft3"
        with patch.dict("os.environ", {}, clear=False) as env, \
                patch.object(Gates, "run_sequence", return_value=True) as sequence:
            env.pop("SOFT3_GENESIS_DIR", None)
            self.assertTrue(chain_genesis_gate(gates, soft3, pussy=self.root / "pussy"))
        name, steps, cwd = sequence.call_args.args[:3]
        self.assertEqual(name, "stack-chain-genesis")
        self.assertEqual(cwd, soft3)
        cargo = ["cargo", "run", "--locked", "--quiet", "--manifest-path", str(soft3 / "crate/Cargo.toml"), "--"]
        genesis = str(self.root / "out/genesis/pussy.tade")
        self.assertEqual(steps, [
            (cargo + ["genesis", "build", "--chain", "pussy", "--snapshot", str(self.root / "pussy/snapshot"),
                      "--cache", str(Path.home() / ".cache/cyber/snapshots/pussy"), "--out", genesis], 900),
            (cargo + ["conformance", "check", "chain", "--genesis", genesis], 900)])

    def test_chain_genesis_manifest_mismatch_is_red_and_builds_nothing(self):
        self.pussy_fixture(matching=False)
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.dict("os.environ", {}, clear=False) as env, \
                patch.object(Gates, "run_sequence") as sequence, patch.object(Gates, "run") as run:
            env.pop("SOFT3_GENESIS_DIR", None)
            self.assertFalse(chain_genesis_gate(gates, self.root / "soft3", pussy=self.root / "pussy"))
        sequence.assert_not_called()
        run.assert_not_called()
        row = gates.rows[-1]
        self.assertEqual(row["result"], "red")
        self.assertIn("0" * 64, row["error"])
        self.assertIn(digest(self.root / "pussy/snapshot/manifest.json"), row["error"])

    def test_chain_genesis_sequence_stops_at_the_first_failure(self):
        gates = Gates(self.root / "out", self.sources, "stack")
        ok = ["python3", "-c", "print('built')"]
        bad = ["python3", "-c", "import sys; sys.exit(3)"]
        never = ["python3", "-c", "print('must not run')"]
        self.assertFalse(gates.run_sequence("seq", [(ok, 60), (bad, 60), (never, 60)], self.root))
        self.assertEqual(gates.rows[-1]["exit_code"], 3)
        log = (self.root / "out/logs/seq.txt").read_text()
        self.assertIn("built", log)
        self.assertNotIn("must not run", log)

    def test_simulation_without_the_pinned_checkout_is_blocked_not_green(self):
        gates = Gates(self.root / "out", self.sources, "stack")
        self.assertFalse(simulation_gate(gates, self.root))
        self.assertEqual(gates.rows[-1]["result"], "blocked")

    def test_simulation_gate_runs_the_gate_suite_from_the_checkout(self):
        manifest = self.root / "simulation/chain/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname = "simulation-chain"\nversion = "0.1.0"\n')
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.object(Gates, "run", return_value=True) as run:
            self.assertTrue(simulation_gate(gates, self.root))
        name, argv, cwd = run.call_args.args[:3]
        self.assertEqual(name, "simulation-gate")
        self.assertEqual(argv[:3], ["cargo", "run", "--locked"])
        self.assertIn("gate", argv)
        self.assertEqual(cwd, self.root / "simulation")
        self.assertIn(str(self.root / "out" / "simulation-gate.md"), argv)

    def test_conformance_scaffold_is_blocked_not_green(self):
        manifest = self.root / "soft3/conformance/rs/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname = "cyber-conformance"\nversion = "0.1.0"\n')
        gates = Gates(self.root, self.sources, "stack")
        self.assertFalse(conformance_gate(gates, self.root / "soft3"))
        self.assertEqual(gates.rows[-1]["result"], "blocked")

    def test_conformance_runner_is_built_from_soft3(self):
        manifest = self.root / "soft3/conformance/rs/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname = "cyber-conformance"\nversion = "0.1.0"\n\n'
                            '[[bin]]\nname = "cargo-conformance"\npath = "src/main.rs"\n')
        gates = Gates(self.root, self.sources, "stack")
        with patch.object(Gates, "run", return_value=True) as run:
            self.assertTrue(conformance_gate(gates, self.root / "soft3"))
        argv = run.call_args.args[1]
        self.assertEqual(argv[:3], ["cargo", "run", "--locked"])
        self.assertEqual(argv[-2:], ["conformance", "--check"])

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


class RehearsalTests(unittest.TestCase):
    DEFAULT, PR, BRANCH = "d" * 40, "e" * 40, "f" * 40

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.manager = self.root / "manager"
        (self.manager / "release").mkdir(parents=True)
        (self.manager / "release/phase1.toml").write_text(
            '[[sibling]]\nname = "lens"\nrepo = "cyberia-to/lens"\nrev = "' + self.DEFAULT + '"\n')
        git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a],
                                        cwd=self.manager, check=True, capture_output=True)
        git("init", "-q")
        git("add", ".")
        git("commit", "-qm", "fixture")
        self.calls = []

    def fake(self, args, cwd=None, timeout=120):
        self.calls.append(args)
        if args[:2] == ["git", "ls-remote"]:
            if "--symref" in args:
                return f"ref: refs/heads/main\tHEAD\n{self.DEFAULT}\tHEAD"
            if args[-1] == "refs/pull/7/head":
                return f"{self.PR}\trefs/pull/7/head"
            if args[-1] == "refs/heads/feat/x":
                return f"{self.BRANCH}\trefs/heads/feat/x"
            return ""
        if args[:2] == ["git", "status"]:
            return ""
        if args[:2] == ["git", "rev-parse"]:
            return "a" * 40
        return "[]"

    def snap(self, overrides=(), cut=False):
        with patch("train_sources.command", side_effect=self.fake):
            return train.snapshot(self.manager, self.root / "out", "soft3", "candidate-20260924.1", overrides, cut)

    def lens(self, result):
        return next(r for r in result["repositories"] if r["name"] == "lens")

    def lookups(self):
        return [c for c in self.calls if c[:2] == ["git", "ls-remote"]]

    def test_cut_with_override_is_refused_before_network(self):
        with self.assertRaisesRegex(ValueError, "rehearsal"):
            self.snap(["lens=feat/x"], cut=True)
        self.assertEqual(self.calls, [])

    def test_malformed_overrides_are_refused(self):
        for bad in [["nope=main"], ["lens=a", "lens=b"], ["lens=../x"], ["lens=-x"], ["lens"], ["lens=/x"],
                    ["lens=a=b"], ["lens=a b"], ["lens="]]:
            with self.assertRaises(ValueError, msg=bad):
                self.snap(bad)
            self.assertEqual(self.lookups(), [])

    def test_branch_override(self):
        result = self.snap(["lens=feat/x"])
        row = self.lens(result)
        self.assertEqual(row["revision"], self.BRANCH)
        self.assertEqual(row["branch"], "main")
        self.assertEqual(row["override"], {"ref": "feat/x", "revision": self.BRANCH,
                                           "default_branch": "main", "default_revision": self.DEFAULT})
        self.assertFalse(row["pin_matches"])
        saved = json.loads((self.root / "out/sources.json").read_text())
        self.assertEqual(saved["overrides"], {"lens": "feat/x"})
        self.assertTrue(saved["rehearsal"])

    def test_commit_override_skips_ls_remote(self):
        sha = "1" * 40
        row = self.lens(self.snap([f"lens={sha}"]))
        self.assertEqual(row["revision"], sha)
        self.assertEqual(row["override"]["default_revision"], self.DEFAULT)
        self.assertEqual(len(self.lookups()), 1 + 3)  # lens default plus three products, none for the ref
        self.assertTrue(all("--symref" in c for c in self.lookups()))

    def test_pull_request_override(self):
        row = self.lens(self.snap(["lens=pull/7/head"]))
        self.assertEqual(row["revision"], self.PR)
        self.assertIn("refs/pull/7/head", [c[-1] for c in self.lookups()])

    def test_missing_ref_is_unavailable(self):
        row = self.lens(self.snap(["lens=gone"]))
        self.assertFalse(row["available"])
        self.assertIn("gone", row["error"])
        self.assertFalse(row["pin_matches"])

    def test_no_override_is_unchanged(self):
        result = self.snap()
        saved = json.loads((self.root / "out/sources.json").read_text())
        self.assertEqual(saved["overrides"], {})
        self.assertFalse(saved["rehearsal"])
        row = self.lens(result)
        self.assertNotIn("override", row)
        self.assertEqual(row["revision"], self.DEFAULT)
        self.assertTrue(row["pin_matches"])

    def test_draft_refuses_rehearsal(self):
        write_json(self.root / "candidate.json", {"name": "candidate-20260924.1", "component": "cyber",
                   "result": "green", "source_revisions": {"cyber": "b" * 40}, "overrides": {"lens": "feat/x"}})
        with patch("train.command") as call:
            with self.assertRaisesRegex(ValueError, "cannot be drafted"):
                train.draft(argparse.Namespace(output=self.root))
        call.assert_not_called()

    def test_collect_carries_overrides_into_notes(self):
        sources = {"component": "cyber", "candidate": "candidate-20260924.1", "manager_revision": "a" * 40,
                   "repositories": [{"name": "lens", "revision": self.BRANCH}], "overrides": {"lens": "feat/x"}}
        snapshot = self.root / "snapshot.json"
        write_json(snapshot, sources)
        artifacts = self.root / "artifacts"
        artifacts.mkdir()
        output = self.root / "assembled"
        train.collect(argparse.Namespace(snapshot=snapshot, artifacts=artifacts, output=output))
        self.assertEqual(json.loads((output / "candidate.json").read_text())["overrides"], {"lens": "feat/x"})
        notes = (output / "release-notes.md").read_text()
        self.assertIn("REHEARSAL RED (not a candidate)", notes.splitlines()[0])
        self.assertIn(f"lens=feat/x → {self.BRANCH}", notes)


if __name__ == "__main__":
    unittest.main()
