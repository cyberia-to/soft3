#!/usr/bin/env python3
"""Verify retained local release evidence; never build, migrate, or deploy."""
import hashlib
import json
import re
import shlex
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
LOGS = HERE / "implementation-baseline"


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def evidence(path):
    path = path if isinstance(path, Path) else ROOT / path
    assert path.is_file() and not path.is_symlink(), path
    return {"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size,
            "sha256": sha(path)}


def main():
    source = json.loads((HERE / "source-manifest.json").read_text())
    dispositions = json.loads((HERE / "final-dispositions.json").read_text())
    scan = json.loads((HERE / "final-scan/scan.json").read_text())
    discovered = {r["path"] for r in scan["repositories"]}
    discovered.update(r["path"] for r in scan["non_git_sources"])
    assert discovered == set(dispositions), "discovery/disposition mismatch"
    verified = 0
    for repo, data in source["repositories"].items():
        for f in data["files"]:
            path = ROOT / repo / f["path"]
            assert path.is_file() and not path.is_symlink(), path
            assert sha(path) == f["sha256"], f"source changed: {path}"
            verified += 1
    for f in source["build_sibling_manifests"]:
        assert f["sha256"] and sha(ROOT / f["path"]) == f["sha256"], f
    context = json.loads((ROOT / "ctx/ctx.sources.json").read_text())
    assert sha(ROOT / "ctx/ctx.md") == context["output_sha256"]
    for f in context["sources"]:
        assert sha(Path(f["path"])) == f["sha256"], f"stale context: {f['path']}"
    artifacts = json.loads((HERE / "artifact-manifest.json").read_text())
    for f in artifacts["artifacts"]:
        assert evidence(f["path"])["sha256"] == f["sha256"], f
    compatibility = json.loads((HERE / "compatibility.json").read_text())
    for section in ["runtime_shims", "retained_interfaces"]:
        for item in compatibility[section]:
            for p in item["paths"]:
                assert list(ROOT.glob(p)), f"missing compatibility path: {p}"

    # These are actual final invocations, separately from Cargo declarations.
    # Owner regression runs below have their own dated audit/profile contracts.
    profiles = [
        ("neuron", "neuron", "cargo test --workspace --all-features --offline", "rustc 1.95.0; NEURON_LEGACY_BINARY=retained original binary", "p13-neuron-all.log"),
        ("neuron-clippy", "neuron", "cargo clippy --workspace --all-targets --all-features --offline --no-deps -- -D warnings", "rustc 1.95.0", "p13-neuron-clippy.log"),
        ("bbg", "bbg/rs", "cargo test --all-features --offline --no-fail-fast", "rustc 1.95.0", "p13-bbg-all.log"),
        ("cyb-core", "cyb", "cargo test -p cyb-core --offline", "rustc 1.95.0; package default features", "p13-cyb-core-all.log"),
        ("cyb-bodies", "cyb", "cargo build --locked --offline -p cyb -p cy", "rustc 1.95.0; package default features; native debug", "p13-cyb-build-source-final.log"),
        ("soft3-node", "soft3/crate", "cargo build --locked --offline --bin soft3", "rustc 1.95.0; package default features; native debug", "p13-soft3-build-source-final.log"),
        ("cyber-launcher", "cyber", "cargo build --release --locked --offline", "rustc 1.95.0; default features; native release", "p13-cyber-launcher-release.log"),
        ("cyber-release-processes", "cyber", "cargo test --locked --offline --all-targets", "rustc 1.95.0; CYBER_TEST_BINARY=cyber/target/release/cyber", "p13-cyber-launcher-release-tests.log"),
        ("fleet", "cyb", "FLEET_SKIP_BUILD=1 FLEET_KEEP=1 python3 harness/fleet.py", "fresh cy/cyb/soft3 native debug executables", "p13-fleet-source-final.log"),
        ("actual-model-bodies", "cyb", "SOMA_TEST_MODEL=/Users/master/llm/qwen3-0.6b-abl.model CYB_TASK_GUI=/Users/master/cyber/cyb/target/release/cyb python3 harness/tasks.py", "fresh native debug CLI + release Bevy; actual pinned local model", "p13-real-tasks-source-final.log"),
        ("identity-wasm", ".", "python3 soft3/audit/neuron-cell/check_identity_consumers.py --out soft3/audit/neuron-cell/implementation-baseline/identity-consumers", "rustc 1.98.0; seven isolated no-default-feature roots; exact features in identity-consumers/results.json", "p13-identity-consumers.log"),
        ("macos", "cyb", "make dmg; make -o release -o apps dmg", "rustc 1.98.0; aarch64-apple-darwin release; default features; second invocation only final version metadata packaging", "p13-dmg-verified.log"),
        ("web", "cyb", "make apps", "Trunk release; rustc 1.98.0 wasm32-unknown-unknown; NO_COLOR=true", "p13-apps.log"),
        ("android", "cyb", "make android", "rustc 1.98.0; aarch64-linux-android release; default features; Gradle release APK", "p13-android-final.log"),
        ("artifact-validation", ".", "python3 soft3/audit/neuron-cell/verify_artifacts.py", "readonly DMG verification/mount/detach; APK zip/library/signature/package checks", "p13-artifact-verification.log"),
        ("protocol-site", "cyber", "nu scripts/build.nu --output /Users/master/cyber/cyber/build", "local optica source build", "p13-protocol-site.log"),
        ("project-site", "cyberia-blog", "nu scripts/build.nu --public-only", "local public optica source build", "p13-project-site.log"),
    ]
    runs = []
    for name, cwd, command, profile, logfile in profiles:
        p = LOGS / logfile
        text = p.read_text()
        assert text.strip(), p
        assert not re.search(r"test result: FAILED|^error: could not compile|^FAIL |^FAILED ", text, re.M), p
        rows = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert all(r[0] == "ok" and r[2] == "0" for r in rows), p
        runs.append({"name": name, "cwd": cwd, "command": command, "profile": profile,
                     "result": "passed", "log": evidence(p),
                     "rust_test_totals": {k: sum(int(r[i]) for r in rows)
                                          for i, k in [(1, "passed"), (2, "failed"), (3, "ignored")]} if rows else None})
    for logfile, required in {
        "p13-fleet-source-final.log": ["32 passed", "0 failed"],
        "p13-real-tasks-source-final.log": ["PASS cli:", "PASS bevy:", "PASS both actual bodies"],
        "p13-android-final.log": ["BUILD SUCCESSFUL"],
        "p13-project-site.log": ["Done! Built"],
        "p13-protocol-site.log": ["Done! Built"],
    }.items():
        text = (LOGS / logfile).read_text()
        assert all(s in text for s in required), (logfile, required)
    supporting_names = [
        "p07-archive-process-tests.log", "p07-archive-bbg-tests.log",
        "p07-archive-migration-tests.log", "p07-archive-storage-faults.log",
        "p04-text-archive-boundaries.log", "p04-text-archive-boundaries-existing.log",
        "p04-text-archive-boundaries-cyb.log", "p13-cybergraph-stack-default-final.log",
        "p13-cybergraph-stack-all-features.log", "p12-rune-events.log",
        "p12-cyb-cli-final.log", "p12-cyb-build-closure-tests.log",
        "p12-cyb-build-dry-run.log", "p13-foculus-partial-mode.log",
        "p09-composition-tests.log", "soma-agent-final.log", "soma-real-provider-tests.log",
        "soma-child-receipt-tests.log", "soma-bevy-correlation-tests.log",
        "soma-agent-runtime-seam.log", "soma-kernel-final.log",
        "p10-mudra-final.log", "p10-lytics-event-final.log", "p10-lytics-ingest-tests.log",
        "p10-cyberia-my-domain.log", "p10-cyberia-research-domain.log",
        "p10-cyberia-wasm.log", "p10-cyberia-research-wasm.log",
        "p10-js-build.log", "p10-js-compat-tests.log", "p10-neptune-wallet-tests.log",
        "p10-true-cyber-tests.log", "p10-inf-workspace-tests.log", "p10-neuron-query-tests.log",
        "p10-soft3-stack-check.log", "p13-dmg-package-final.log",
        "p13-diffcheck.log", "p13-residual-api-check.log", "p13-final-doc-links.log",
    ]
    supporting = []
    for name in supporting_names:
        p = LOGS / name
        assert not re.search(r"test result: FAILED", p.read_text()), p
        supporting.append(evidence(p))
    binary_deps = []
    for name in ["cyb/target/release/cyb", "cyb/target/debug/cyb", "cyb/target/debug/cy",
                 "soft3/crate/target/debug/soft3", "cyber/target/release/cyber"]:
        binary = ROOT / name
        depfile = binary.with_suffix(".d")
        files = shlex.split(depfile.read_text().splitlines()[0].split(": ", 1)[1])
        for f in files:
            path = Path(f)
            assert path.is_absolute() and path.is_file(), path
            assert path.stat().st_mtime_ns <= binary.stat().st_mtime_ns, f"binary older than input: {path}"
        binary_deps.append({**evidence(binary), "depfile": evidence(depfile),
                            "input_count": len(files), "newer_or_missing_inputs": 0})
    identity = json.loads((LOGS / "identity-consumers/results.json").read_text())
    assert len(identity["profiles"]) == 7 and all(p["result"] == "passed" for p in identity["profiles"])
    model = Path("/Users/master/llm/qwen3-0.6b-abl.model")
    output = {
        "schema": "soft3/neuron-convergence-release-validation/1",
        "verified_utc": datetime.now(timezone.utc).isoformat(),
        "result": "passed-for-declared-local-profiles",
        "evaluation": "G01–G14 are the manual contract/scenario review in release.md; this script rechecks retained evidence, source/artifact/context hashes and binary input freshness. It does not rerun tests or prove semantic correctness from log text.",
        "inputs": [evidence(HERE / n) for n in ["source-manifest.json", "final-dispositions.json", "artifact-manifest.json", "compatibility.json", "release.md"]]
                  + [evidence(ROOT / "soft3/roadmap/neuron-cell-convergence.md"), evidence(Path(__file__))],
        "verified_sources": verified, "dispositions": len(dispositions),
        "required_build_manifests": len(source["build_sibling_manifests"]),
        "context": {"manifest": evidence(ROOT / "ctx/ctx.sources.json"), "output": evidence(ROOT / "ctx/ctx.md"), "verified_inputs": len(context["sources"]), "stale_inputs": 0},
        "gates": [{"gate": f"G{i:02}", "package": f"P{i-1:02}", "result": "passed-for-declared-local-profiles", "review": "release.md#plan-closure-matrix"} for i in range(1, 15)],
        "actual_profiles": runs, "supporting_owner_evidence": supporting,
        "identity_wasm_profiles": identity,
        "local_model": {"path": str(model), "bytes": model.stat().st_size, "sha256": sha(model)},
        "original_legacy_executable": evidence(LOGS / "legacy-cell-binary"),
        "selected_body_evidence": [evidence(p) for p in sorted((LOGS / "p13-bodies").iterdir()) if p.is_file()],
        "binaries": binary_deps,
        "artifact_verification": artifacts,
        "limits": ["Non-atomic local working-copy observation, with explicit dirty source hashes; no commit/publication/deployment claim.", "Prior owner regression logs have their own dates/profile bounds; this is not a claim that every historical suite ran on every file hash in the final inventory.", "Binary dependency mtimes are a freshness check, not a reproducible-build proof.", "Source gaps and retained worktrees/vendor/generated/foreign meanings follow final-dispositions and compatibility; no unseen Omi content is certified.", "Scope follows accepted roadmap section 11; complete Hermes integration/provider/channel parity is subsequent work."]
    }
    target = HERE / "validation-manifest.json"
    temporary = target.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(output, indent=2, ensure_ascii=False) + "\n")
    temporary.replace(target)
    print(json.dumps({"result": output["result"], "gates": len(output["gates"]),
                      "sources": verified, "dispositions": len(dispositions),
                      "profiles": len(runs), "source_mismatches": 0, "context_mismatches": 0,
                      "output": str(target)}))


if __name__ == "__main__":
    main()
