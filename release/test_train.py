"""Release-integrity tests; synthetic receipts never masquerade as platform builds."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import train
from train_sources import TARGETS, digest, resolve, write_json
from train_gates import Gates, glibc_floor_gate, glibc_requirements, run_gates, chain_genesis_gate, conformance_gate, simulation_gate, source_gates


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

    def pussy_fixture(self, matching=True, rc_time=1793523600, rc_sha256=None):
        manifest = self.root / "pussy/snapshot/manifest.json"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('{"fixture": true}\n')
        expected = digest(manifest) if matching else "0" * 64
        spec = self.root / "soft3/conformance/chain-genesis.toml"
        spec.parent.mkdir(parents=True)
        rc = f"[pussy-rc]\ntime = {rc_time}\n" if rc_time is not None else ""
        if rc_time is not None and rc_sha256:
            rc += f'file_sha256 = "{rc_sha256}"\n'
        spec.write_text(f'[pussy]\nsource_manifest_sha256 = "{expected}"\n{rc}')

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
            (cargo + ["conformance", "check", "chain", "--genesis", genesis], 900),
            (cargo + ["genesis", "build", "--chain", "pussy-rc", "--snapshot", str(self.root / "pussy/snapshot"),
                      "--cache", str(Path.home() / ".cache/cyber/snapshots/pussy"), "--time", "1793523600",
                      "--out", str(self.root / "out/genesis/pussy-rc.tade")], 900),
            (cargo + ["genesis", "verify", str(self.root / "out/genesis/pussy-rc.tade")], 900)])

    def rc_pin_run(self, pinned):
        built = b"rehearsal bytes"
        self.pussy_fixture(rc_sha256=pinned if pinned is None or pinned else hashlib.sha256(built).hexdigest())
        rc = self.root / "out/genesis/pussy-rc.tade"
        rc.parent.mkdir(parents=True)
        rc.write_bytes(built)
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.dict("os.environ", {}, clear=False) as env, \
                patch.object(Gates, "run_sequence", return_value=True):
            env.pop("SOFT3_GENESIS_DIR", None)
            result = chain_genesis_gate(gates, self.root / "soft3", pussy=self.root / "pussy")
        return result, gates, hashlib.sha256(built).hexdigest()

    def test_chain_genesis_pinned_rc_sha256_matching_is_green(self):
        result, gates, _ = self.rc_pin_run("")
        self.assertTrue(result)
        self.assertEqual(gates.rows, [])

    def test_chain_genesis_pinned_rc_sha256_mismatch_is_red(self):
        result, gates, built = self.rc_pin_run("1" * 64)
        self.assertFalse(result)
        row = gates.rows[-1]
        self.assertEqual((row["name"], row["result"]), ("stack-chain-genesis", "red"))
        self.assertIn(built, row["error"])
        self.assertIn("1" * 64, row["error"])

    def test_chain_genesis_unpinned_rc_is_green_without_hashing(self):
        result, gates, _ = self.rc_pin_run(None)
        self.assertTrue(result)

    def test_chain_genesis_pinned_rc_file_missing_is_red(self):
        self.pussy_fixture(rc_sha256="2" * 64)
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.dict("os.environ", {}, clear=False) as env, \
                patch.object(Gates, "run_sequence", return_value=True):
            env.pop("SOFT3_GENESIS_DIR", None)
            self.assertFalse(chain_genesis_gate(gates, self.root / "soft3", pussy=self.root / "pussy"))
        self.assertIn("2" * 64, gates.rows[-1]["error"])

    def test_chain_genesis_skips_the_rehearsal_steps_without_a_pussy_rc_time(self):
        self.pussy_fixture(rc_time=None)
        gates = Gates(self.root / "out", self.sources, "stack")
        with patch.dict("os.environ", {}, clear=False) as env, \
                patch.object(Gates, "run_sequence", return_value=True) as sequence:
            env.pop("SOFT3_GENESIS_DIR", None)
            self.assertTrue(chain_genesis_gate(gates, self.root / "soft3", pussy=self.root / "pussy"))
        steps = sequence.call_args.args[1]
        self.assertEqual(len(steps), 2)
        self.assertNotIn("pussy-rc", [a for argv, _ in steps for a in argv])

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

    def test_stack_gates_have_their_measured_budgets(self):
        (self.root / "soft3").mkdir()
        with patch("train_gates.source_gates"), patch("train_gates.conformance_gate"), \
                patch("train_gates.chain_genesis_gate"), patch("train_gates.simulation_gate"), \
                patch.object(Gates, "run", return_value=True) as run:
            run_gates(self.root, self.root / "out", self.sources, {}, "soft3", "stack", stack=True)
        budgets = {c.args[0]: c.kwargs["timeout"] for c in run.call_args_list if c.args[0].startswith("stack-")}
        self.assertEqual(budgets["stack-zheng"], 1800)
        self.assertEqual(budgets["stack-lens"], 1200)
        self.assertEqual(budgets["stack-radio"], 1200)
        others = {name: t for name, t in budgets.items() if name not in ("stack-zheng", "stack-lens", "stack-radio")}
        self.assertIn("stack-hemera", others)
        self.assertIn("stack-bbg", others)
        self.assertEqual(set(others.values()), {900})
        for call in run.call_args_list:
            if call.args[0].startswith("stack-"):
                self.assertEqual(call.args[1], ["cargo", "test", "--locked"])

    def stack_repo(self, owner="radio"):
        repo = self.root / owner
        repo.mkdir()
        git = lambda *a: subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *a], cwd=repo,
                                        check=True, capture_output=True, text=True).stdout.strip()
        git("init", "-q")
        (repo / "seeds.txt").write_text("one\n")
        (repo / ".gitignore").write_text("target/\n")
        git("add", ".")
        git("commit", "-q", "-m", "init")
        return repo, git, git("rev-parse", "HEAD")

    def run_dirtying_stack(self, result, dirty=True):
        repo, git, revision = self.stack_repo()
        self.sources = {**self.sources, "repositories": [{"name": "radio", "revision": revision}]}
        (repo / "target").mkdir()
        (repo / "target/x").write_text("build output\n")

        def fake(gates, name, argv, cwd, env=None, timeout=1800):
            if name == "stack-radio" and dirty:
                (repo / "seeds.txt").write_text("one\ntwo\n")
                (repo / "new.txt").write_text("fresh\n")
            return gates.record(name, result, log=f"logs/{name}.txt")

        out = self.root / "out"
        with patch("train_gates.source_gates"), patch("train_gates.conformance_gate"), \
                patch("train_gates.chain_genesis_gate"), patch("train_gates.simulation_gate"), \
                patch.object(Gates, "run", autospec=True, side_effect=fake):
            run_gates(self.root, out, self.sources, {}, "soft3", "stack", stack=True)
        rows = {r["name"]: r for r in json.loads((out / "release-validation.json").read_text())["gates"]}
        self.assertEqual(git("rev-parse", "HEAD"), revision)
        self.assertEqual(git("status", "--porcelain"), "")
        self.assertTrue((repo / "target/x").exists())
        return rows["stack-radio"], out

    def test_green_stack_gate_that_dirties_its_tree_is_red_and_restored(self):
        (self.root / "soft3").mkdir()
        row, out = self.run_dirtying_stack("green")
        self.assertEqual(row["result"], "red")
        self.assertEqual(row["reason"], "the test changed its own source tree")
        self.assertEqual(sorted(row["dirtied"]), ["new.txt", "seeds.txt"])
        self.assertEqual(row["dirtied_log"], "logs/stack-radio.dirtied.txt")
        log = (out / row["dirtied_log"]).read_text()
        self.assertIn("+two", log)
        self.assertIn("seeds.txt", log)

    def test_red_stack_gate_that_dirties_its_tree_stays_red_and_restored(self):
        (self.root / "soft3").mkdir()
        row, _ = self.run_dirtying_stack("red")
        self.assertEqual(row["result"], "red")
        self.assertNotIn("reason", row)
        self.assertEqual(sorted(row["dirtied"]), ["new.txt", "seeds.txt"])

    def test_clean_stack_gate_is_untouched(self):
        (self.root / "soft3").mkdir()
        row, out = self.run_dirtying_stack("green", dirty=False)
        self.assertEqual(row["result"], "green")
        self.assertNotIn("dirtied", row)
        self.assertFalse((out / "logs/stack-radio.dirtied.txt").exists())

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


class QualifyCapTests(unittest.TestCase):
    def test_stack_leg_cap_covers_its_gates(self):
        root = Path(__file__).resolve().parent.parent
        text = (root / ".github/workflows/candidate.yml").read_text()
        qualify = text[text.index("\n  qualify:"):]
        found = re.search(r"^    timeout-minutes: (.+)$", qualify, re.M)
        self.assertIsNotNone(found)
        match = re.fullmatch(r"\$\{\{ matrix\.target == 'stack' && (\d+) \|\| (\d+) \}\}", found.group(1).strip())
        self.assertIsNotNone(match, found.group(1))
        stack, other = int(match.group(1)), int(match.group(2))
        self.assertEqual((stack, other), (210, 90))
        measured, setup, sim_timeout = 4752.7, 650, 3600
        self.assertIn(f"timeout={sim_timeout}", (root / "release/train_gates.py").read_text())
        self.assertGreaterEqual(stack * 60, measured + setup + sim_timeout)
        self.assertLessEqual(stack, 360)


def synthetic_elf(versions, wide=True, order="<"):
    """Minimal ELF with section headers: null, .dynstr, .gnu.version_r, .shstrtab."""
    strings = b"\0" + b"libc.so.6\0" + b"".join(v.encode() + b"\0" for v in versions)
    names, at = {}, len(b"\0libc.so.6\0")
    for v in versions:
        names[v] = at
        at += len(v) + 1
    shstr = b"\0.dynstr\0.gnu.version_r\0.shstrtab\0"
    verneed = b""
    for i, v in enumerate(versions):
        last = i == len(versions) - 1
        verneed += struct.pack(order + "IHHII", 0, 0, 0, names[v], 0 if last else 16)
    header = struct.pack(order + "HHIII", 1, len(versions), 1, 16, 0)
    verneed = header + verneed
    ehsize, shentsize = (64, 64) if wide else (52, 40)
    dynstr_at = ehsize
    vneed_at = dynstr_at + len(strings)
    shstr_at = vneed_at + len(verneed)
    shoff = shstr_at + len(shstr)

    def sh(kind, offset, size, link=0):
        if wide:
            return struct.pack(order + "IIQQQQIIQQ", 0, kind, 0, 0, offset, size, link, 0, 1, 0)
        return struct.pack(order + "IIIIIIIIII", 0, kind, 0, 0, offset, size, link, 0, 1, 0)

    sections = (sh(0, 0, 0) + sh(3, dynstr_at, len(strings)) + sh(0x6ffffffe, vneed_at, len(verneed), 1) +
                sh(3, shstr_at, len(shstr)))
    ident = b"\x7fELF" + bytes([2 if wide else 1, 1 if order == "<" else 2, 1]) + bytes(9)
    if wide:
        head = ident + struct.pack(order + "HHIQQQIHHHHHH", 3, 62, 1, 0, 0, shoff, 0, 64, 56, 0, 64, 4, 3)
    else:
        head = ident + struct.pack(order + "HHIIIIIHHHHHH", 3, 3, 1, 0, 0, shoff, 0, 52, 32, 0, 40, 4, 3)
    return head + strings + verneed + shstr + sections


class GlibcFloorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.sources = {"manager_revision": "a" * 40, "repositories": []}

    def gate(self, blob, target="x86_64-unknown-linux-gnu"):
        binary = self.root / "soft3"
        if blob is not None:
            binary.write_bytes(blob)
        gates = Gates(self.root / "out", self.sources, target)
        glibc_floor_gate(gates, binary, target)
        return gates

    def test_parses_versions_in_every_elf_flavour(self):
        for wide, order in ((True, "<"), (False, "<"), (True, ">"), (False, ">")):
            path = self.root / "elf"
            path.write_bytes(synthetic_elf(["GLIBC_2.34", "GLIBC_2.2.5", "GCC_3.0"], wide, order))
            self.assertEqual(glibc_requirements(path), {(2, 34), (2, 2)}, (wide, order))

    def test_non_elf_and_missing_are_empty(self):
        path = self.root / "junk"
        path.write_bytes(b"not an elf at all" * 10)
        self.assertEqual(glibc_requirements(path), set())
        self.assertEqual(glibc_requirements(self.root / "absent"), set())
        path.write_bytes(synthetic_elf(["GLIBC_2.38"])[:100])
        self.assertEqual(glibc_requirements(path), set())

    def test_green_at_or_below_floor(self):
        gates = self.gate(synthetic_elf(["GLIBC_2.34", "GLIBC_2.2.5"]))
        row, = gates.rows
        self.assertEqual((row["name"], row["result"], row["required"], row["floor"]),
                         ("linux-glibc-floor", "green", "2.34", "2.35"))

    def test_red_above_floor_makes_result_red(self):
        gates = self.gate(synthetic_elf(["GLIBC_2.38", "GLIBC_2.2.5"]))
        row, = gates.rows
        self.assertEqual((row["result"], row["required"]), ("red", "2.38"))
        self.assertEqual(json.loads((self.root / "out/release-validation.json").read_text())["result"], "red")

    def test_missing_binary_is_blocked(self):
        row, = self.gate(None).rows
        self.assertEqual((row["result"], row["reason"]), ("blocked", "no executable"))

    def test_unreadable_executable_fails_closed(self):
        row, = self.gate(synthetic_elf(["GLIBC_2.38"])[:100]).rows
        self.assertEqual((row["result"], row["required"]), ("red", "unreadable"))
        row, = self.gate(synthetic_elf(["GCC_3.0"])).rows
        self.assertEqual(row["result"], "red")

    def test_other_targets_record_nothing(self):
        self.assertEqual(self.gate(synthetic_elf(["GLIBC_2.38"]), "aarch64-apple-darwin").rows, [])
        self.assertEqual(self.gate(None, "aarch64-linux-android").rows, [])

    def test_run_gates_calls_floor_for_soft3(self):
        (self.root / "soft3").mkdir()
        with patch("train_gates.source_gates"), patch.object(Gates, "run", return_value=True), \
                patch("train_gates.boot_status"), patch("train_gates.glibc_floor_gate") as floor:
            run_gates(self.root, self.root / "out", self.sources, {}, "soft3", "x86_64-unknown-linux-gnu")
        self.assertEqual(floor.call_args.args[2], "x86_64-unknown-linux-gnu")

    def test_workflow_builds_linux_on_glibc_235(self):
        root = Path(__file__).resolve().parent.parent
        text = (root / ".github/workflows/candidate.yml").read_text()
        rows = dict((t, r) for r, t in re.findall(r"\{runner: ([\w.-]+), target: ([\w-]+)\}", text))
        self.assertEqual(rows["x86_64-unknown-linux-gnu"], "ubuntu-22.04")
        self.assertEqual(rows["aarch64-unknown-linux-gnu"], "ubuntu-22.04-arm")
        qualify = text[text.index("\n  qualify:"):text.index("\n  collect:")]
        self.assertRegex(qualify, r"uses: actions/setup-python@[0-9a-f]{40}")
        self.assertIn("python-version: '3.12'", qualify)
        setup = qualify[qualify.index("uses: actions/setup-python@"):]
        setup = setup[:setup.index("\n      - ", 1)]
        self.assertIn("!contains(matrix.runner, 'build')", setup)
        host = qualify[qualify.index("name: Host Python (build service)"):]
        host = host[:host.index("\n      - ", 1)]
        self.assertIn("if: ${{ contains(matrix.runner, 'build') }}", host)
        self.assertIn("sys.version_info >= (3, 12)", host)


if __name__ == "__main__":
    unittest.main()
