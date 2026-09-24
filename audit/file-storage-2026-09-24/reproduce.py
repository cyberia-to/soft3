#!/usr/bin/env python3
"""Run synthetic file-storage probes without opening a real product store."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True,
                        help="Fresh, nonexistent scratch directory")
    args = parser.parse_args()
    source = args.source_root.resolve()
    output = args.output.resolve()
    report = Path(__file__).resolve().parent
    manifest = json.loads((report / "sources.json").read_text())
    for repo in ("file", "hemera", "radio", "cyb"):
        revision = subprocess.check_output(
            ["git", "-C", str(source / repo), "rev-parse", "HEAD"], text=True
        ).strip()
        if revision != manifest["repositories"][repo]["revision"]:
            raise SystemExit(f"Revision differs from the audited input: {repo}")
        if repo != "cyb":
            clean = subprocess.run(
                ["git", "-C", str(source / repo), "diff", "--quiet", "HEAD", "--"]
            )
            if clean.returncode:
                raise SystemExit(f"Tracked source differs from the audited input: {repo}")
    relative = "cyb/shell/src/worlds/content.rs"
    content = (source / relative).read_bytes()
    expected = manifest["inspected_files"][relative]["sha256"]
    if hashlib.sha256(content).hexdigest() != expected:
        raise SystemExit("Cyb content source differs from the audited input")
    output.mkdir(parents=True, exist_ok=False)
    (output / "src").mkdir()
    dependencies = {
        "file": {"package": "cyber-file", "path": str(source / "file")},
        "hemera": {"package": "cyber-hemera", "path": str(source / "hemera/rs")},
        "cyber-bao": {"path": str(source / "radio/cyber-bao")},
    }
    cargo = '[package]\nname="file-storage-audit-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
    for name, values in dependencies.items():
        cargo += name + " = { " + ", ".join(
            key + " = " + json.dumps(value) for key, value in values.items()
        ) + " }\n"
    cargo += 'serde_json = "1"\n'
    (output / "Cargo.toml").write_text(cargo)
    text = content.decode()
    original = 'std::env::var("HOME")'
    assert text.count(original) == 1
    text = text.replace(original, 'std::env::var("CYBER_AUDIT_FIXTURE_ROOT")')
    (output / "src/content.rs").write_text(text)
    shutil.copyfile(report / "probe.rs", output / "src/main.rs")
    shutil.copyfile(report / "probe.lock", output / "Cargo.lock")
    command = ["cargo", "run", "--locked", "--manifest-path", str(output / "Cargo.toml")]
    environment = dict(os.environ, CYBER_AUDIT_FIXTURE_ROOT=str(output / "fixture"))
    with (output / "probe.log").open("w") as log:
        result = subprocess.run(command, env=environment, stdout=log, stderr=subprocess.STDOUT)
    print(f"Probe exit {result.returncode}; evidence: {output / 'probe.log'}")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
