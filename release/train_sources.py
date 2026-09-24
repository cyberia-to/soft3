"""Capture and materialize origin-only release inputs."""
import base64
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

from train_components import capture as component_inputs
from train_build import contract as build_contract, fetch as fetch_build, stack_sources

PRODUCTS = {
    "soft3": ("cyberia-to/soft3", "crate/Cargo.toml"),
    "cyber": ("cyberia-to/cyber", "Cargo.toml"),
    "cyb": ("cyberia-to/cyb", "shell/Cargo.toml"),
}
TARGETS = ["aarch64-apple-darwin", "x86_64-apple-darwin",
           "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"]


def command(args, cwd=None, timeout=120):
    result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, timeout=timeout,
                            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"})
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr.strip()}")
    return result.stdout.strip()


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def resolve(entry):
    name, repo = entry["name"], entry["repo"]
    if not re.fullmatch(r"[a-z][a-z0-9_-]*", name) or not re.fullmatch(r"[\w.-]+/[\w.-]+", repo):
        raise ValueError("invalid source repository")
    row = {**entry, "url": f"https://github.com/{repo}.git"}
    try:
        remote = command(["git", "ls-remote", "--symref", row["url"], "HEAD"], timeout=45)
        branch = re.search(r"^ref: refs/heads/(\S+)\s+HEAD$", remote, re.M)
        revision = re.search(r"^([0-9a-f]{40})\s+HEAD$", remote, re.M)
        if not branch or not revision:
            raise ValueError("origin has no default-branch HEAD")
        actual = revision[1]
        if entry.get("source") == "upstream":
            # An upstream release is pinned in default-branch history, not updated
            # whenever its maintainers advance HEAD.
            if not re.fullmatch(r"[0-9a-f]{40}", entry.get("rev", "")):
                raise ValueError("upstream source requires a full revision")
            status = command(["gh", "api", f"repos/{repo}/compare/{entry['rev']}...{actual}", "--jq", ".status"])
            if status not in {"ahead", "identical"}:
                raise ValueError("upstream pin is not in default-branch history")
            row["origin_head_revision"] = actual
            actual = entry["rev"]
        row.update(branch=branch[1], revision=actual, available=True)
        row["pin_matches"] = (entry.get("rev") == actual and
                              entry.get("branch", branch[1]) == branch[1]) if "rev" in entry else None
    except (RuntimeError, ValueError, subprocess.TimeoutExpired) as error:
        row.update(available=False, error=str(error), pin_matches=False)
    return row


def snapshot(manager, output, component, candidate):
    if command(["git", "status", "--porcelain"], cwd=manager):
        raise ValueError("release manager must use committed clean inputs")
    manifest = manager / "release/phase1.toml"
    inherited = None
    if component == "soft3":
        entries = tomllib.loads(manifest.read_text())["sibling"]
        repo, path = PRODUCTS["soft3"]
        entries += [{"name": "soft3", "repo": repo, "manifest": path}]
        names = [entry["name"] for entry in entries]
        if len(names) != len(set(names)):
            raise ValueError("duplicate source name")
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            rows = list(pool.map(resolve, entries))
        phase1_sha256 = digest(manifest)
    else:
        # Capture only this product. The stack is inherited byte-for-byte from
        # one authenticated build; no component origin HEAD is resolved again.
        repo, path = PRODUCTS[component]
        product = resolve({"name": component, "repo": repo, "manifest": path})
        if not product["available"]:
            raise ValueError(product["error"])
        encoded = command(["gh", "api", f"repos/{repo}/contents/release/soft3.toml?ref={product['revision']}",
                           "--jq", ".content"])
        pin = build_contract(base64.b64decode(encoded).decode())
        inherited, upstream = fetch_build(pin, output / "soft3-build")
        rows = [*stack_sources(upstream), product]
        phase1_sha256 = upstream["phase1_sha256"]
    rows.sort(key=lambda row: row["name"])
    result = {"component": component, "candidate": candidate,
              "manager_revision": command(["git", "rev-parse", "HEAD"], cwd=manager),
              "phase1_sha256": phase1_sha256, "repositories": rows,
              "changes": [], "change_errors": []}
    if inherited:
        result["soft3_build"] = inherited
    for row in rows:
        if row["name"] != component or not row["available"]:
            continue
        try:
            changes = json.loads(command(["gh", "api", f"repos/{row['repo']}/commits/{row['revision']}/pulls",
                                          "--jq", '[.[] | select(.merged_at != null) | {title, html_url}]']))
            if not changes:
                raise ValueError("captured product HEAD has no merged pull request")
            result["changes"].extend({"component": row["name"], **change} for change in changes)
        except (RuntimeError, ValueError, subprocess.TimeoutExpired) as error:
            result["change_errors"].append({"component": row["name"], "error": str(error)})
    write_json(output / "sources.json", result)
    return result


def materialize(sources, destination):
    destination.mkdir(parents=True, exist_ok=False)

    def fetch(row):
        if not row["available"]:
            return {"name": row["name"], "result": "red", "error": row["error"]}
        try:
            path = destination / row["name"]
            command(["git", "init", "--quiet", str(path)])
            command(["git", "-C", str(path), "remote", "add", "origin", row["url"]])
            command(["git", "-C", str(path), "fetch", "--quiet", "--depth", "1", "origin", row["revision"]], timeout=300)
            command(["git", "-C", str(path), "switch", "--detach", "FETCH_HEAD"])
            # Submodule objects are pinned by the captured tree, never copied locally.
            command(["git", "-C", str(path), "submodule", "update", "--init", "--recursive", "--depth", "1"], timeout=300)
            if command(["git", "-C", str(path), "status", "--porcelain"]):
                raise ValueError("checkout is dirty")
            return {"name": row["name"], "result": "green"}
        except (RuntimeError, ValueError, subprocess.TimeoutExpired) as error:
            return {"name": row["name"], "result": "red", "error": str(error)}

    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        return list(pool.map(fetch, sources["repositories"]))


def component_bindings(metadata, repositories):
    owned = {p["name"]: row["name"] for row in repositories
             if row["name"] not in {"cyber", "cyb"} for p in row["declared_packages"]}
    escaped = [{"component": owned[p["name"]], "package": p["name"], "version": p["version"], "source": p["source"]}
               for p in metadata["packages"] if p["source"] is not None and p["name"] in owned]
    return {"result": "red" if escaped else "green", "outside_build": escaped}


def inventory(sources, directory, output):
    result = json.loads(json.dumps(sources))
    for row in result["repositories"]:
        root = directory / row["name"]
        row["packages"] = []
        row["declared_packages"] = []
        for manifest in sorted(root.rglob("Cargo.toml")) if root.is_dir() else []:
            if any(part in {"target", ".git"} for part in manifest.relative_to(root).parts):
                continue
            try:
                package = tomllib.loads(manifest.read_text()).get("package", {})
                if package.get("name"):
                    row["declared_packages"].append({"name": package["name"], "version": package.get("version"),
                                                    "manifest": str(manifest.relative_to(root))})
            except (OSError, tomllib.TOMLDecodeError):
                continue
        if row.get("manifest") and (root / row["manifest"]).is_file():
            package = tomllib.loads((root / row["manifest"]).read_text()).get("package", {})
            row["version"] = package.get("version")
    result["component_inputs"] = component_inputs(sources, directory)
    # Preserve failed package resolution explicitly; source inventory always exists.
    component = sources["component"]
    manifest = directory / component / PRODUCTS[component][1]
    try:
        metadata = json.loads(command(["cargo", "metadata", "--locked", "--format-version", "1",
                                       "--manifest-path", str(manifest)], timeout=180))
        by_name = {row["name"]: row for row in result["repositories"]}
        result["component_bindings"] = component_bindings(metadata, result["repositories"])
        for package in metadata["packages"]:
            if package["source"] is not None:
                continue
            path = Path(package["manifest_path"]).resolve().relative_to(directory.resolve())
            row = by_name[path.parts[0]]
            row["packages"].append({"name": package["name"], "version": package["version"],
                                    "manifest": str(path), "vendored": "vendor" in path.parts})
        result["package_resolution"] = {"result": "green"}
    except (RuntimeError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        result["package_resolution"] = {"result": "red", "error": str(error)}
    write_json(output / "sources.json", result)
    lines = ["## soft3 source inventory", "", "| component | source revision | pin | packages |", "|---|---|---|---|"]
    for row in result["repositories"]:
        revision = row.get("revision", "unavailable")
        link = f"[{revision}](https://github.com/{row['repo']}/commit/{revision})" if row["available"] else revision
        pin = "product" if row["name"] in PRODUCTS else ("matches" if row["pin_matches"] else "RED")
        packages = ", ".join(f"{p['name']}@{p['version']}" + (" (vendored)" if p["vendored"] else "") for p in row["packages"])
        if not packages:
            declarations = ", ".join(f"{p['name']}@{p['version']}" for p in row["declared_packages"]
                                     if isinstance(p["version"], str))
            packages = f"unresolved; declared: {declarations}" if declarations else "unresolved"
        lines.append(f"| {row['name']} | {link} | {pin} | {packages} |")
    lines += ["", "Package resolution: " + result["package_resolution"]["result"] + ". See sources.json for the exact result.", ""]
    (output / "soft3-dependencies.md").write_text("\n".join(lines))
    write_json(output / "soft3-dependencies.json", result)
    return result
