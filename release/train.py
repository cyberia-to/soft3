#!/usr/bin/env python3
"""One candidate engine for soft3, cyber and cyb. Drafts only."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import re
import shutil
import tarfile

from train_sources import PRODUCTS, TARGETS, command, digest, inventory, materialize, snapshot, write_json
from train_gates import run_gates
from train_notes import render as render_notes


def candidate_name(value):
    if not re.fullmatch(r"candidate-\d{8}\.[1-9]\d*", value):
        raise ValueError("expected candidate-YYYYMMDD.N")
    datetime.datetime.strptime(value[10:18], "%Y%m%d")
    return value


def checksums(directory):
    paths = sorted(p for p in directory.rglob("*") if p.is_file() and p.name != "SHA256SUMS")
    (directory / "SHA256SUMS").write_text("".join(f"{digest(p)}  {p.relative_to(directory).as_posix()}\n" for p in paths))


def build(args):
    sources = json.loads(args.snapshot.read_text())
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    checkouts = materialize(sources, args.checkout.resolve())
    actual = inventory(sources, args.checkout.resolve(), output)
    component = "soft3" if args.stack else sources["component"]
    artifacts = run_gates(args.checkout.resolve(), output, actual, checkouts,
                          component, args.target, stack=args.stack)
    versions = {r["name"]: r.get("version") for r in actual["repositories"] if r["name"] in PRODUCTS}
    candidate = {"name": sources["candidate"], "component": sources["component"], "target": args.target,
                 "versions": versions, "source_revisions": {r["name"]: r.get("revision") for r in actual["repositories"]},
                 "source_snapshot_sha256": digest(args.snapshot), "manager_revision": sources["manager_revision"],
                 "artifacts": []}
    for artifact in artifacts:
        shutil.copy2(artifact, output / artifact.name)
        candidate["artifacts"].append({"name": artifact.name, "sha256": digest(artifact)})
    write_json(output / "candidate.json", candidate)
    validation = json.loads((output / "release-validation.json").read_text())
    changed = []
    for row in sources["repositories"]:
        location = args.checkout / row["name"]
        if not row["available"] or not (location / ".git").exists():
            continue
        try:
            if command(["git", "rev-parse", "HEAD"], cwd=location) != row["revision"] or command(["git", "status", "--porcelain"], cwd=location):
                changed.append(row["name"])
        except RuntimeError:
            changed.append(row["name"])
    validation["gates"].append({"name": "source-inputs-unchanged", "result": "red" if changed else "green", "changed": changed})
    if changed:
        validation["result"] = "red"
    validation["complete"] = True
    if actual["package_resolution"]["result"] != "green":
        validation["result"] = "red"
        validation["gates"].append({"name": "package-resolution", **actual["package_resolution"]})
    write_json(output / "release-validation.json", validation)
    checksums(output)
    archive = output.parent / f"{sources['component']}-{sources['candidate']}-{args.target}.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        for path in sorted(output.iterdir()):
            tar.add(path, arcname=path.name)
    print(f"{validation['result'].upper()}: {archive}")


def collect(args):
    sources = json.loads(args.snapshot.read_text())
    name, component = sources["candidate"], sources["component"]
    candidate_name(name)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    expected = ["stack", *TARGETS] + (["aarch64-linux-android"] if component == "cyb" else [])
    results, versions, binaries, details, inventories = [], None, [], [], {}
    source_revisions = {r["name"]: r.get("revision") for r in sources["repositories"]}
    for target in expected:
        archives = list(args.artifacts.rglob(f"{component}-{name}-{target}.tar.gz"))
        row = {"target": target, "result": "red"}
        if len(archives) != 1:
            row["error"] = "expected exactly one platform receipt archive"
            results.append(row)
            continue
        archive = archives[0]
        try:
            with tarfile.open(archive) as tar:
                files = {}
                for member in tar.getmembers():
                    path = Path(member.name)
                    if path.is_absolute() or ".." in path.parts or member.issym() or member.islnk():
                        raise ValueError("unsafe archive member")
                    if member.isfile():
                        if member.name in files:
                            raise ValueError("duplicate archive member")
                        files[member.name] = tar.extractfile(member).read()
                record = json.loads(files["candidate.json"])
                validation = json.loads(files["release-validation.json"])
                if record["source_snapshot_sha256"] != digest(args.snapshot) or record["source_revisions"] != source_revisions:
                    raise ValueError("source inventory differs across platform receipts")
                if record["target"] != target or record["name"] != name or record["component"] != component:
                    raise ValueError("candidate identity mismatch")
                if versions is not None and versions != record["versions"]:
                    raise ValueError("product versions differ across platform receipts")
                versions = record["versions"]
                for asset in record["artifacts"]:
                    if hashlib.sha256(files[asset["name"]]).hexdigest() != asset["sha256"]:
                        raise ValueError("binary checksum mismatch")
                # Even red receipts require their complete inventory and log checksums.
                hashed = set()
                for line in files["SHA256SUMS"].decode().splitlines():
                    checksum, path = line.split("  ", 1)
                    if path in hashed:
                        raise ValueError("duplicate checksum entry")
                    hashed.add(path)
                    if hashlib.sha256(files[path]).hexdigest() != checksum:
                        raise ValueError("receipt checksum mismatch")
                if hashed != set(files) - {"SHA256SUMS"}:
                    raise ValueError("receipt checksum coverage incomplete")
                required = {"sources.json", "soft3-dependencies.json", "soft3-dependencies.md"}
                if not required.issubset(files):
                    raise ValueError("mandatory soft3 inventory missing")
                inventory_sources = json.loads(files["sources.json"])
                if json.loads(files["soft3-dependencies.json"]) != inventory_sources:
                    raise ValueError("dependency inventory differs from platform sources")
                if {r["name"]: r.get("revision") for r in inventory_sources["repositories"]} != source_revisions:
                    raise ValueError("inventory revisions differ from captured sources")
                row.update(result=validation["result"], gates=validation["gates"], archive=archive.name,
                           archive_sha256=digest(archive), artifacts=record["artifacts"])
                if not validation.get("complete") or (target != "stack" and not record["artifacts"]):
                    row.update(result="red", error="qualification incomplete or binary unavailable")
                if any(gate["result"] != "green" for gate in validation["gates"]):
                    row["result"] = "red"
                binaries.extend({"target": target, **asset} for asset in record["artifacts"] if target != "stack")
                if not details:
                    details = files["soft3-dependencies.md"].decode().splitlines()
                inventories[target] = inventory_sources
                shutil.copy2(archive, output / archive.name)
        except (ValueError, KeyError, OSError, tarfile.TarError) as error:
            row["error"] = str(error)
        results.append(row)
    verdict = "green" if all(r["result"] == "green" for r in results) else "red"
    write_json(output / "sources.json", sources)
    write_json(output / "soft3-dependencies.json", {"source_snapshot": sources, "platforms": inventories})
    write_json(output / "release-validation.json", {"result": verdict, "platforms": results,
               "source_snapshot_sha256": digest(args.snapshot)})
    write_json(output / "candidate.json", {"name": name, "component": component, "versions": versions,
               "result": verdict, "source_revisions": source_revisions, "available_binaries": binaries,
               "manager_revision": sources["manager_revision"], "promotion": "owner only"})
    (output / "soft3-dependencies.md").write_text("\n".join(details or ["Source capture is in sources.json; no platform resolved a package inventory."]) + "\n")
    (output / "release-notes.md").write_text(render_notes(output))
    checksums(output)
    print(f"{verdict.upper()} {component} {name}")


def draft(args):
    output = args.output.resolve()
    candidate = json.loads((output / "candidate.json").read_text())
    name = candidate_name(candidate["name"])
    component = candidate["component"]
    repo = PRODUCTS[component][0]
    assets = [str(path) for path in sorted(output.iterdir()) if path.is_file()]
    marker = "🟢" if candidate["result"] == "green" else "🔴"
    # gh keeps this tag name in draft metadata. No tag is pushed or published.
    result = command(["gh", "release", "create", name, *assets, "--repo", repo, "--draft", "--prerelease",
                      "--target", candidate["source_revisions"][component], "--latest=false",
                      "--title", f"{marker} {component} {name} — {candidate['result'].upper()}",
                      "--notes-file", str(output / "release-notes.md")], timeout=300)
    print(result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    prepare = sub.add_parser("snapshot")
    prepare.add_argument("--manager", type=Path, required=True)
    prepare.add_argument("--component", choices=PRODUCTS, required=True)
    prepare.add_argument("--candidate", type=candidate_name, required=True)
    prepare.add_argument("--output", type=Path, required=True)
    build_parser = sub.add_parser("build")
    build_parser.add_argument("--snapshot", type=Path, required=True)
    build_parser.add_argument("--checkout", type=Path, required=True)
    build_parser.add_argument("--target", choices=["stack", *TARGETS, "aarch64-linux-android"], required=True)
    build_parser.add_argument("--stack", action="store_true")
    build_parser.add_argument("--output", type=Path, required=True)
    gather = sub.add_parser("collect")
    gather.add_argument("--snapshot", type=Path, required=True)
    gather.add_argument("--artifacts", type=Path, required=True)
    gather.add_argument("--output", type=Path, required=True)
    publish = sub.add_parser("draft")
    publish.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.action == "snapshot":
        snapshot(args.manager.resolve(), args.output.resolve(), args.component, args.candidate)
    else:
        {"build": build, "collect": collect, "draft": draft}[args.action](args)


if __name__ == "__main__":
    main()
