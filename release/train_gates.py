"""Executable stack and product gates; every failure produces a receipt."""
import json
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import tempfile
import time
import tomllib
import urllib.request

from train_sources import PRODUCTS, command, digest, write_json


class Gates:
    def __init__(self, output, sources, target):
        self.output, self.sources, self.target = output, sources, target
        self.rows = []
        (output / "logs").mkdir(parents=True, exist_ok=True)

    def record(self, name, result, **detail):
        self.rows.append({"name": name, "result": result, **detail})
        self.write()
        print(f"{result.upper()} {name}", flush=True)
        return result == "green"

    def write(self):
        write_json(self.output / "release-validation.json", {
            "target": self.target, "result": "green" if all(r["result"] == "green" for r in self.rows) else "red",
            "manager_revision": self.sources["manager_revision"],
            "source_revisions": {r["name"]: r.get("revision") for r in self.sources["repositories"]},
            "gates": self.rows})

    def run(self, name, argv, cwd, env=None, timeout=1800):
        return self.run_sequence(name, [(argv, timeout)], cwd, env)

    def run_sequence(self, name, steps, cwd, env=None):
        """One gate, several commands: `steps` is [(argv, timeout)]. The first failing
        step (non-zero exit, timeout, or a warning in its output) makes the gate red and
        later steps do not run; all output lands in the gate's one log."""
        log = self.output / "logs" / (name + ".txt")
        start = time.monotonic()
        code, argv, warnings = -1, steps[0][0], False
        with log.open("w") as stream:
            for index, (argv, timeout) in enumerate(steps):
                stream.write(("\n" if index else "") + "command: " + json.dumps(argv) + "\ncwd: " + str(cwd) + "\n")
                stream.flush()
                code = -1
                begin = stream.tell()
                try:
                    process = subprocess.Popen(argv, cwd=cwd, stdout=stream, stderr=subprocess.STDOUT,
                                               env={**os.environ, **(env or {})}, start_new_session=True)
                    try:
                        code = process.wait(timeout=timeout)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
                        stream.write(f"\ntimeout after {timeout} seconds\n")
                except OSError as error:
                    stream.write(str(error) + "\n")
                stream.flush()
                with log.open("rb") as reader:
                    reader.seek(begin)
                    warnings = bool(re.search(r"(?m)^warning(?:\[|:)", reader.read().decode(errors="replace")))
                if code != 0 or warnings:
                    break
        return self.record(name, "green" if code == 0 and not warnings else "red",
                           command=argv, cwd=str(cwd), exit_code=code, warnings=warnings,
                           elapsed_seconds=round(time.monotonic() - start, 3),
                           log=str(log.relative_to(self.output)), log_sha256=digest(log),
                           **({"steps": [step[0] for step in steps]} if len(steps) > 1 else {}))

    def blocked(self, name, reason):
        return self.record(name, "blocked", reason=reason)


def source_gates(gates, directory, sources, checkouts, component):
    failures = [r for r in checkouts if r["result"] != "green"]
    gates.record("origin-checkouts", "red" if failures else "green", failures=failures)
    drift = [r["name"] for r in sources["repositories"] if r["name"] not in PRODUCTS and r.get("pin_matches") is not True]
    gates.record("phase1-pins", "red" if drift else "green", drift=drift,
                 manifest_sha256=sources["phase1_sha256"])
    gates.record("release-notes-source", "red" if sources.get("change_errors") else "green",
                 errors=sources.get("change_errors", []))
    if component != "soft3":
        contract = directory / component / "release/soft3.toml"
        soft3 = next(r for r in sources["repositories"] if r["name"] == "soft3")
        try:
            dependency = tomllib.loads(contract.read_text())["soft3"]
            actual = tomllib.loads((directory / "soft3/crate/Cargo.toml").read_text())["package"]["version"]
            valid = (dependency["revision"] == soft3["revision"] == sources["manager_revision"] and
                     dependency["version"] == actual and dependency["repository"] == PRODUCTS["soft3"][0])
            gates.record("soft3-dependency", "green" if valid else "red", expected=dependency,
                         actual={"version": actual, "revision": soft3["revision"]})
        except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
            gates.record("soft3-dependency", "red", error=str(error))


CONFORMANCE_RUNNER = "cargo-conformance"


def conformance_gate(gates, soft3):
    """Run `cargo conformance --check` from the runner soft3 carries in-tree.

    A clean origin checkout has no installed cargo subcommand, so the gate builds the
    runner from soft3/conformance/rs, which closes over the sibling checkouts beside
    soft3 by path. The runner regenerates the stack snapshot from the current build and
    compares it byte for byte with soft3/conformance/snapshots (conformance/specs/README.md).
    A soft3 revision whose crate declares no runner binary is blocked, never green.
    """
    manifest = soft3 / "conformance/rs/Cargo.toml"
    try:
        bins = tomllib.loads(manifest.read_text()).get("bin", [])
    except (OSError, tomllib.TOMLDecodeError):
        bins = []
    if not any(b.get("name") == CONFORMANCE_RUNNER for b in bins):
        return gates.blocked("conformance-snapshot",
                             f"cargo conformance is not implemented: {manifest.relative_to(soft3.parent)} declares no "
                             f"`{CONFORMANCE_RUNNER}` binary, see soft3/conformance/specs/README.md")
    return gates.run("conformance-snapshot",
                     ["cargo", "run", "--locked", "--quiet", "--manifest-path", str(manifest), "--bin",
                      CONFORMANCE_RUNNER, "--", "conformance", "--check"], soft3, timeout=900)


GENESIS_ENV = "SOFT3_GENESIS_DIR"


def chain_genesis_gate(gates, soft3, genesis_dir=None, pussy=None):
    """chain-conformance.md §10 in the release train: the pussy genesis file
    (pinned snapshot inputs, `soft3 genesis build`) must reproduce the blessed
    particle, state root and block-0 id in soft3/conformance/chain.snap.
    Where the file comes from, in order:
      a. `pussy.tade` in an explicit genesis_dir or SOFT3_GENESIS_DIR (local runs):
         checked as it is;
      b. else the pinned cyberia-to/space-pussy checkout (`pussy`, the `pussy`
         sibling of release/phase1.toml) holding snapshot/manifest.json: its
         sha256 must equal [pussy].source_manifest_sha256 in
         conformance/chain-genesis.toml (a mismatch is red, nothing is built),
         then the train builds the file into <output>/genesis/pussy.tade with
         `soft3 genesis build` (IPFS fetches cached in ~/.cache/cyber/snapshots/pussy,
         never in the checkout) and checks it, all under this one gate;
      c. else blocked, never green."""
    gate = "stack-chain-genesis"
    cargo = ["cargo", "run", "--locked", "--quiet", "--manifest-path", str(soft3 / "crate/Cargo.toml"), "--"]
    explicit = genesis_dir or os.environ.get(GENESIS_ENV)
    if explicit and (Path(explicit) / "pussy.tade").is_file():
        genesis = Path(explicit) / "pussy.tade"
        return gates.run(gate, cargo + ["conformance", "check", "chain", "--genesis", str(genesis)], soft3, timeout=900)
    manifest = pussy / "snapshot/manifest.json" if pussy else None
    if manifest is None or not manifest.is_file():
        return gates.blocked(gate, "no pussy genesis file: pin cyberia-to/space-pussy in release/phase1.toml "
                                   f"(the train builds the file from its snapshot) or set {GENESIS_ENV} to a directory holding pussy.tade")
    try:
        expected = tomllib.loads((soft3 / "conformance/chain-genesis.toml").read_text())["pussy"]["source_manifest_sha256"]
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        return gates.record(gate, "red", error=f"cannot read [pussy].source_manifest_sha256 from conformance/chain-genesis.toml: {error}")
    actual = digest(manifest)
    if actual != expected:
        return gates.record(gate, "red", error=f"pinned space-pussy snapshot/manifest.json sha256 {actual} != "
                                               f"conformance/chain-genesis.toml [pussy].source_manifest_sha256 {expected}")
    genesis = gates.output / "genesis/pussy.tade"
    genesis.parent.mkdir(parents=True, exist_ok=True)
    cache = Path.home() / ".cache/cyber/snapshots/pussy"
    build = cargo + ["genesis", "build", "--chain", "pussy", "--snapshot", str(pussy / "snapshot"),
                     "--cache", str(cache), "--out", str(genesis)]
    check = cargo + ["conformance", "check", "chain", "--genesis", str(genesis)]
    return gates.run_sequence(gate, [(build, 900), (check, 900)], soft3)


SIMULATION_SUITE_ENV = "SOFT3_SIMULATION_SUITE"


def simulation_gate(gates, directory):
    """The chain simulator (cyberia-to/simulation, pinned in release/phase1.toml):
    N in-process nodes on this soft3 revision's kernel behind the wire's port,
    a simulated network, miners at simulated hashrate shares grinding real
    headers, adversaries and the economy's strategies, with liveness, safety,
    conservation, determinism, resource and economy-bound invariants. The
    `gate` suite (SOFT3_SIMULATION_SUITE overrides) runs in about ten minutes;
    its verdict table lands beside the receipts as simulation-gate.md, its
    JSONL metrics under simulation/. A train without the pinned checkout is
    blocked, never green; any failed invariant is red (expected reds — known
    open findings reproduced on purpose — are not)."""
    sim = directory / "simulation"
    if not (sim / "chain/Cargo.toml").is_file():
        return gates.blocked("simulation-gate", "no simulation checkout: pin cyberia-to/simulation in release/phase1.toml")
    suite = os.environ.get(SIMULATION_SUITE_ENV, "gate")
    return gates.run("simulation-gate",
                     ["cargo", "run", "--locked", "--quiet", "--profile", "sim", "-p", "simulation-chain", "--", suite,
                      "--out", str(gates.output / "simulation"), "--md", str(gates.output / "simulation-gate.md"),
                      "--budget", "1500"], sim, timeout=3600)


def boot_status(binary, gates):
    if not binary.is_file():
        return gates.blocked("node-status", "release executable was not built")
    with tempfile.TemporaryDirectory(prefix="soft3-release-") as home, socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        address = f"127.0.0.1:{listener.getsockname()[1]}"
        listener.close()
        log = gates.output / "logs/node-status.txt"
        argv = [str(binary), "node", "--home", home, "--bind", address]
        with log.open("w") as stream:
            stream.write("command: " + json.dumps(argv) + "\n")
            stream.flush()
            process = subprocess.Popen(argv, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
            status = "red"
            try:
                deadline = time.monotonic() + 20
                while time.monotonic() < deadline and process.poll() is None:
                    try:
                        with urllib.request.urlopen(f"http://{address}/status", timeout=1) as response:
                            body = response.read().decode()
                            stream.write(body)
                            if response.status == 200 and "height" in body and "chain" in body:
                                status = "green"
                                break
                    except (OSError, ValueError):
                        time.sleep(0.1)
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
        return gates.record("node-status", status, command=argv, log="logs/node-status.txt", log_sha256=digest(log))


# Per-gate wall-clock budget of the stack's `cargo test --locked`, in seconds. Default 900.
# zheng: its soundness review recursion::review::a_state_unbound_from_the_last_public_input_is_refused
# (a two-segment IVC at 2^15 under hemera 0.4) runs ~985 s alone at opt-level 3 (zheng CI run
# 40293728603: suite 1077.95 s) and the build takes 262 s on the train stand (rehearsal
# 40299054024), so the gate needs ~1340 s.
# No test leaves the gate; the budget follows the measurement (plan G10b).
# lens: cyber-lens-rspcs's exhaustive bit-flip soundness scans (already spread over every core)
# are most of the gate; green at 615.7-856.0 s on release/0.2 across six rehearsals
# (40267704972 .. 40307972588), so 900 s leaves 44 s at the worst sample (plan G10a).
# radio: green at 605.3-779.8 s after radio#47's dev profile (40286177469 .. 40307972588); compile
# alone varies ~120 s between runners and an earlier head passed at 881.2 s (40267704972) (plan G10a).
STACK_TEST_BUDGET = {"zheng": 1800, "lens": 1200, "radio": 1200}
STACK_TEST_DEFAULT = 900


# Plan G4a, rehearsal 40316574923: a failing proptest in radio appended its seed to the tracked
# iroh-blobs/proptest-regressions/*.txt, so the radio checkout went dirty and source-inputs-unchanged
# was red (`changed: [radio]`) while that mutation leaked into the inputs the soft3 product build
# consumes. Disabling proptest persistence is no answer: it would also stop replaying the committed
# seeds (proptest 1.10 config.rs:146). Instead the stack gate carries the evidence (it goes red with
# `dirtied` and a diff log) and the checkout is restored to its recorded revision, so
# source-inputs-unchanged then measures what the product build consumed.
def restore_stack_checkout(gates, name, repository, revision):
    if not revision or not (repository / ".git").exists():
        return
    porcelain = command(["git", "status", "--porcelain"], cwd=repository)
    if not porcelain:
        return
    # Listed by git, not parsed from porcelain: tracked changes against HEAD, then untracked files.
    tracked = command(["git", "diff", "--name-only", "HEAD"], cwd=repository)
    untracked = command(["git", "ls-files", "--others", "--exclude-standard"], cwd=repository)
    paths = [p for p in (tracked + "\n" + untracked).splitlines() if p]
    try:
        diff = command(["git", "diff", "--binary", "HEAD"], cwd=repository)
    except (RuntimeError, UnicodeDecodeError) as error:
        diff = f"(diff unavailable: {error})"
    log = f"logs/{name}.dirtied.txt"
    (gates.output / log).write_text(f"{porcelain}\n\n{diff}\n")
    row = gates.rows[-1]
    assert row["name"] == name, (row["name"], name)
    row["dirtied"], row["dirtied_log"] = paths, log
    if row["result"] == "green":
        row["result"], row["reason"] = "red", "the test changed its own source tree"
    gates.write()
    command(["git", "checkout", "--force", "--detach", revision], cwd=repository)
    command(["git", "clean", "-fd"], cwd=repository)


def run_gates(directory, output, sources, checkouts, component, target, stack=False):
    gates = Gates(output, sources, target)
    source_gates(gates, directory, sources, checkouts, component)
    root = directory / component
    if not root.is_dir():
        gates.blocked("product", "origin checkout unavailable")
        return []
    artifacts = []
    if stack:
        # The stack owns the hard gates; products consume this verdict and source set.
        owners = {"hemera": "rs", "bbg": "rs", "lens": ".", "nox": "rs", "zheng": "rs",
                  "cybergraph": ".", "foculus": ".", "tru": "rs", "tok": "rs", "mudra": ".", "vault": ".",
                  "neuron": ".", "file": ".", "radio": "."}
        for owner, path in owners.items():
            location = directory / owner / path
            gates.run(f"stack-{owner}", ["cargo", "test", "--locked"], location, timeout=STACK_TEST_BUDGET.get(owner, STACK_TEST_DEFAULT))
            revision = next((r.get("revision") for r in sources["repositories"] if r["name"] == owner), None)
            restore_stack_checkout(gates, f"stack-{owner}", directory / owner, revision)
        conformance_gate(gates, directory / "soft3")
        chain_genesis_gate(gates, directory / "soft3", pussy=directory / "pussy")
        simulation_gate(gates, directory)
    if component == "soft3":
        crate = root / "crate"
        gates.run("soft3-tests", ["cargo", "test", "--locked"], crate)
        # Plan C4: soft3#406 moved the node's tests to cybernode/rs (rehearsal 40525967811: soft3-tests
        # ran 2 tests against 314 in 40446879208). SOFT3_CONFORMANCE_DIR is explicit because chain.snap
        # stays in soft3 (proposals/cybernode.md) and cybernode#1 reads it from there. 1800 s is the
        # budget soft3-tests had for these tests.
        cybernode = directory / "cybernode" / "rs"
        if cybernode.is_dir():
            gates.run("cybernode-tests", ["cargo", "test", "--locked"], cybernode,
                      env={"SOFT3_CONFORMANCE_DIR": str(root / "conformance")}, timeout=1800)
        else:
            gates.blocked("cybernode-tests", "cybernode checkout unavailable")
        gates.run("soft3-release", ["cargo", "build", "--release", "--locked"], crate)
        binary = crate / "target/release/soft3"
        boot_status(binary, gates)
        if binary.is_file():
            artifacts.append(binary)
    elif component == "cyber":
        gates.run("cyber-tests", ["cargo", "test", "--locked"], root)
        gates.run("cyber-release", ["nu", "scripts/release.nu", "--locked-sources"], root)
        # Graph compilation is a separate product gate, with an origin-pinned builder.
        optica = directory / "optica"
        built = gates.run("optica-build", ["cargo", "build", "--release", "--locked"], optica)
        if built:
            gates.run("protocol-graph", [str(optica / "target/release/optica"), "build", str(root)], root)
        else:
            gates.blocked("protocol-graph", "optica builder failed")
        binary = root / "dist/cyber"
        if binary.is_file():
            artifacts.append(binary)
    elif target == "aarch64-linux-android":
        gates.run("android-build", ["make", "android"], root, timeout=2700)
        artifacts.extend(root.glob("shell/gen/android/app/build/outputs/apk/release/*.apk"))
        verifier = Path(os.environ.get("ANDROID_HOME", "")) / "build-tools/35.0.0/apksigner"
        for apk in artifacts:
            gates.run("android-signature", [str(verifier), "verify", str(apk)], root)
    else:
        checked = gates.run("cyb-check", ["cargo", "check", "--tests", "--locked"], root)
        if checked:
            gates.run("cyb-tests", ["cargo", "test", "--locked"], root)
            argv = ["make", "fleet"] if "apple" in target else ["xvfb-run", "-a", "make", "fleet"]
            gates.run("cyb-fleet", argv, root)
            if "apple" in target:
                gates.run("wasm-toolchain", ["rustup", "target", "add", "wasm32-unknown-unknown"], root)
                gates.run("trunk-tool", ["cargo", "install", "trunk", "--version", "0.21.14", "--locked"], root)
                gates.run("cyb-dmg", ["make", "dmg"], root, timeout=2700)
                artifacts.extend(root.glob("target/release/*.dmg"))
            else:
                gates.run("cyb-release", ["cargo", "build", "--release", "--locked", "-p", "cyb"], root, timeout=2700)
                if (root / "target/release/cyb").is_file():
                    artifacts.append(root / "target/release/cyb")
        else:
            for name in ["cyb-tests", "cyb-fleet", "cyb-dmg" if "apple" in target else "cyb-release"]:
                gates.blocked(name, "cyb check failed; see cyb-check log")
    return artifacts
