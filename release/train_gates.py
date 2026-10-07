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

from train_sources import PRODUCTS, digest, write_json


class Gates:
    def __init__(self, output, sources, target):
        self.output, self.sources, self.target = output, sources, target
        self.rows = []
        (output / "logs").mkdir(parents=True, exist_ok=True)

    def record(self, name, result, **detail):
        self.rows.append({"name": name, "result": result, **detail})
        write_json(self.output / "release-validation.json", {
            "target": self.target, "result": "green" if all(r["result"] == "green" for r in self.rows) else "red",
            "manager_revision": self.sources["manager_revision"],
            "source_revisions": {r["name"]: r.get("revision") for r in self.sources["repositories"]},
            "gates": self.rows})
        print(f"{result.upper()} {name}", flush=True)
        return result == "green"

    def run(self, name, argv, cwd, env=None, timeout=1800):
        log = self.output / "logs" / (name + ".txt")
        start = time.monotonic()
        code = -1
        with log.open("w") as stream:
            stream.write("command: " + json.dumps(argv) + "\ncwd: " + str(cwd) + "\n")
            stream.flush()
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
        warnings = bool(re.search(r"(?m)^warning(?:\[|:)", log.read_text(errors="replace")))
        return self.record(name, "green" if code == 0 and not warnings else "red",
                           command=argv, cwd=str(cwd), exit_code=code, warnings=warnings,
                           elapsed_seconds=round(time.monotonic() - start, 3),
                           log=str(log.relative_to(self.output)), log_sha256=digest(log))

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
    if component == "soft3":
        actual = next((r.get("revision") for r in sources["repositories"] if r["name"] == "soft3"), None)
        gates.record("soft3-manager", "green" if actual == sources["manager_revision"] else "red",
                     expected=actual, actual=sources["manager_revision"])
    if component != "soft3":
        contract = directory / component / "release/soft3.toml"
        soft3 = next(r for r in sources["repositories"] if r["name"] == "soft3")
        try:
            dependency = tomllib.loads(contract.read_text())["soft3"]
            actual = tomllib.loads((directory / "soft3/crate/Cargo.toml").read_text())["package"]["version"]
            inherited = sources["soft3_build"]
            valid = (dependency == inherited["contract"] and
                     dependency["revision"] == soft3["revision"] and
                     dependency["version"] == actual and dependency["repository"] == PRODUCTS["soft3"][0])
            gates.record("soft3-dependency", "green" if valid else "red", expected=dependency,
                         actual={"version": actual, "revision": soft3["revision"]})
        except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
            gates.record("soft3-dependency", "red", error=str(error))
        inherited = sources.get("soft3_build", {})
        gates.record("soft3-build", inherited.get("result", "red"),
                     reason=inherited.get("reason", "No authenticated soft3 build receipt."),
                     expected=inherited.get("contract"))


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
            gates.run(f"stack-{owner}", ["cargo", "test", "--locked"], location, timeout=900)
        # Some library tests invoke target/debug/nu through nu-test-support.
        if gates.run("stack-nu-build", ["cargo", "build", "--locked", "--bin", "nu"],
                     directory / "nu", timeout=1800):
            # Upstream CLI integration fixtures include the optional plugin-path.
            gates.run("stack-nu", ["cargo", "test", "--locked", "--features", "nu-cli/plugin",
                      *[arg for package in ["nu-protocol", "nu-engine", "nu-parser", "nu-command", "nu-cmd-lang",
                                            "nu-cmd-extra", "nu-cli", "nu-std", "nu-utils"] for arg in ["-p", package]]],
                      directory / "nu", timeout=1800)
        else:
            gates.blocked("stack-nu", "Nu test executable was not built; see stack-nu-build log")
        # This gate requires the real snapshot command. A scaffold cannot pass it.
        gates.run("conformance-snapshot", ["cargo", "conformance", "--check"], directory / "soft3", timeout=120)
    if component == "soft3":
        crate = root / "crate"
        gates.run("soft3-tests", ["cargo", "test", "--locked"], crate)
        gates.run("soft3-release", ["cargo", "build", "--release", "--locked"], crate)
        binary = crate / "target/release/soft3"
        boot_status(binary, gates)
        if binary.is_file():
            artifacts.append(binary)
    elif component == "cyber":
        gates.run("cyber-tests", ["cargo", "test", "--locked"], root)
        # The shared build receipt owns dependency provenance. The standalone
        # development sources.lock.json must not replace the selected soft3 build.
        gates.run("cyber-format", ["cargo", "fmt", "--check"], root)
        built = gates.run("cyber-release", ["cargo", "build", "--release", "--locked"], root)
        binary = root / "target/release/cyber"
        if built:
            gates.run("cyber-acceptance", ["cargo", "test", "--locked", "--test", "node"], root,
                      env={"CYBER_TEST_BINARY": str(binary)})
        else:
            gates.blocked("cyber-acceptance", "release executable was not built")
        # Graph compilation is a separate product gate, with an origin-pinned builder.
        optica = directory / "optica"
        built = gates.run("optica-build", ["cargo", "build", "--release", "--locked"], optica)
        if built:
            gates.run("protocol-graph", [str(optica / "target/release/optica"), "build", str(root)], root)
        else:
            gates.blocked("protocol-graph", "optica builder failed")
        binary = root / "target/release/cyber"
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
