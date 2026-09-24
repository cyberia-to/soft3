#!/usr/bin/env python3
"""Read-only source provenance and repository dispositions; never a gate runner.

Python >= 3.11, standard library only. --out is a directory. The only writes are
the three generated output files (atomic replacement) and their temporary files.
No source contents, environment values, Git remotes, credentials or build output
are copied. SHA-256 here checks source bytes; it is not a protocol commitment.
"""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import subprocess
import sys
import tomllib
import uuid

HERE = Path(__file__).resolve().parent
OUTPUT_NAMES = ("source-manifest.json", "final-dispositions.json", "final-repositories.md")
MAX_SOURCE_BYTES = 8 * 1024 * 1024
PRUNE = {
    ".git", ".hg", ".svn", ".cache", ".claude", ".grok", ".codex", ".idea",
    ".vscode", ".worktrees", "__pycache__", "target", "target-release", "build",
    "dist", "node_modules", ".next", ".nuxt", ".venv", "venv", "coverage",
    "audit", "generated", ".generated", "out", ".DS_Store",
}
SECRET_DIRS = {".ssh", ".aws", ".gnupg", "secrets", ".secrets", "keystore", "wallets"}
EXTENSIONS = {
    ".rs", ".toml", ".lock", ".md", ".mdx", ".rst", ".txt", ".html", ".css",
    ".scss", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx", ".json", ".jsonl",
    ".yaml", ".yml", ".py", ".nu", ".sh", ".bash", ".zsh", ".fish", ".go",
    ".mod", ".sum", ".c", ".h", ".cc", ".cpp", ".hpp", ".m", ".mm",
    ".metal", ".wgsl", ".glsl", ".slang", ".cu", ".cuh", ".swift", ".kt",
    ".java", ".proto", ".sql", ".nix", ".rune", ".inf", ".trit", ".wat",
    ".wit", ".sol", ".xml", ".svg", ".cmake", ".s", ".asm", ".j2",
}
NAMES = {"Makefile", "Dockerfile", "Justfile", "justfile", "LICENSE", "COPYING",
         "Cargo.toml", "Cargo.lock", "CMakeLists.txt", "rust-toolchain", ".gitignore",
         ".gitattributes", ".gitmodules", ".dockerignore"}
SECRET_NAME = re.compile(
    r"(?i)(^\.env(?:\.|$)|(?:^|[-_.])(?:credentials?|mnemonic|seed|secret|private[-_]?key|keys?|wallet)"
    r"(?:[-_.].*)?\.(?:json|toml|ya?ml|txt|csv|key)$|\.(?:pem|key|p12|pfx|jks|keystore)$"
    r"|^id_(?:rsa|dsa|ecdsa|ed25519)(?:\.|$)|^\.npmrc$|^\.netrc$)"
)
# Conservative content detection: only a reason is emitted, never the match.
SECRET_CONTENT = re.compile(
    rb"-----BEGIN (?:[A-Z0-9 ]*PRIVATE KEY|PGP PRIVATE KEY BLOCK)-----|"
    rb"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,}|"
    rb"AKIA[A-Z0-9]{16}|xox[baprs]-[A-Za-z0-9-]{20,}|sk-[A-Za-z0-9_-]{32,})\b|"
    rb"(?i:[\"']?(?:private[_-]?key|secret[_-]?key|mnemonic|api[_-]?key|access[_-]?token)"
    rb"[\"']?\s*[:=]\s*[\"'][A-Za-z0-9 /+_=-]{24,}[\"'])"
)
NEW = {
    ".worktrees/mudra-private-retrieval": (
        "preserved-worktree", "Separate docs/private-recovery-contract branch; preserve its Git "
        "state and contents under plan §10. Main-checkout migration does not rewrite this branch."),
    ".worktrees/soft3-thread-publication": (
        "preserved-worktree", "Separate docs/thread-publication branch; preserve its Git state "
        "and contents under plan §10. Main-checkout migration does not rewrite this branch."),
    "simulation": (
        "new-independent-source", "New emergence-simulation repository. The observed cell match "
        "is an orthogonalization benchmark label; no runtime subject rename is assigned. "
        "Source: simulation/README.md and examples/bench_kernels.rs."),
}


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def relative(value):
    p = PurePosixPath(value)
    if p.is_absolute() or ".." in p.parts or "\x00" in value:
        raise ValueError("unsafe relative path")
    return p


def safe_read(root, rel, limit=MAX_SOURCE_BYTES):
    """No-follow opens for every component, bounded regular-file read, race check."""
    parts = relative(str(rel)).parts
    if not parts:
        raise ValueError("not a file")
    fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in parts[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd)
            fd = child
        leaf = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
        try:
            before = os.fstat(leaf)
            if not stat.S_ISREG(before.st_mode):
                raise ValueError("non-regular-file")
            if before.st_size > limit:
                raise ValueError("file-size-limit")
            data = bytearray()
            while chunk := os.read(leaf, min(65536, limit + 1 - len(data))):
                data.extend(chunk)
                if len(data) > limit:
                    raise ValueError("file-size-limit")
            after = os.fstat(leaf)
            if (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
                    after.st_size, after.st_mtime_ns, after.st_ctime_ns):
                raise ValueError("changed-during-read")
            return bytes(data)
        finally:
            os.close(leaf)
    finally:
        os.close(fd)


def command(args, cwd, limit=32 * 1024 * 1024):
    # Read-only Git plumbing/status, no optional locks or lazy object downloads.
    env = dict(os.environ, GIT_OPTIONAL_LOCKS="0", GIT_NO_LAZY_FETCH="1",
               RUSTUP_AUTO_INSTALL="0")
    try:
        p = subprocess.run(args, cwd=cwd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.DEVNULL, timeout=30, check=False)
        if len(p.stdout) > limit:
            return None, "command-output-limit"
        if p.returncode:
            return None, "command-exit-" + str(p.returncode)
        return p.stdout, None
    except (OSError, subprocess.TimeoutExpired) as e:
        return None, type(e).__name__


def git_state(path, kind):
    if kind in {"non-git-source-tree", "generated-git-marker", "broken-git-checkout"}:
        return dict(head=None, branch=None, dirty=None, status_entries=None,
                    observation="not-readable-git-root", scanner_kind=kind)
    values, errors = {}, []
    for key, args in (
        ("head", ["rev-parse", "--verify", "HEAD"]),
        ("branch", ["symbolic-ref", "--quiet", "--short", "HEAD"]),
        ("status", ["status", "--porcelain=v1", "-z", "--untracked-files=all"]),
    ):
        out, error = command(["git", "--no-optional-locks", "-C", str(path), *args], path)
        if error:
            values[key] = None
            if key != "branch":
                errors.append(key + ":" + error)
        elif key == "status":
            # Rename/copy entries have an additional NUL path; do not expose filenames.
            tokens, i, n = out.split(b"\0"), 0, 0
            while i < len(tokens) and tokens[i]:
                code = tokens[i][:2]
                n += 1
                i += 2 if b"R" in code or b"C" in code else 1
            values[key] = n
        else:
            values[key] = out.decode("utf-8", errors="replace").strip()
    return dict(head=values["head"], branch=values["branch"],
                dirty=None if values["status"] is None else bool(values["status"]),
                status_entries=values["status"], observation="current-read-only",
                errors=errors)


def disposition(row, baseline):
    path, kind = row["path"], row["kind"]
    original = "cell" if path == "neuron" else path
    d = dict(baseline.get(original, {"kind": "new-source", "packages": "—",
                                  "action": "No action assigned by the immutable baseline."}))
    category = d["kind"]
    rationale = "Inherited baseline scope; assigned action is not evidence of completion."
    if path == "neuron":
        category, rationale = "renamed-owner", "Current neuron checkout inherits cell disposition; " \
            "directory rename preserves repository history. Git observation below records this checkout."
    elif path in NEW:
        category, rationale = NEW[path]
    elif kind == "generated-git-marker" or d["kind"] == "generated-copy":
        category, rationale = "generated-copy", "Generated Git marker in publication output; " \
            "canonical source owner and rebuild provenance apply, not edits to this marker."
    elif d["kind"] == "reference-partial":
        category, rationale = "source-gap", "Sparse upstream reference; locally unavailable blobs " \
            "remain an explicit gap. No conclusion about their unread contents."
    elif d["kind"] == "reference":
        category, rationale = "vendor-reference", "Independent embedded/vendor/reference root. " \
            "Preserve upstream semantics and original bytes; not a migrated subject owner."
    elif kind == "worktree":
        category, rationale = "preserved-worktree", "Separate branch/worktree retained under plan §10; " \
            "no bulk rewrite or claim that another checkout's edits are present here."
    elif kind == "broken-git-checkout":
        category, rationale = "source-without-git-metadata", "Available source can be checksummed, " \
            "but HEAD/branch/dirty are unknown; broken Git metadata is not a clean checkout."
    elif kind == "non-git-source-tree":
        category, rationale = "non-git-source-group", "Separately enumerated non-Git source group; " \
            "no repository revision can be inferred. See inherited scope."
    elif "/" in path:
        category, rationale = "nested-independent-repository", "Independent nested Git root, " \
            "with its own disposition/revision; parent checkout does not cover its contents."
    elif original not in baseline:
        category, rationale = "new-source-review-required", "New discovered root without a reviewed " \
            "baseline disposition. No automated assertion of migration or irrelevance."
    errors = Counter(e.get("reason", "unspecified") for e in row.get("errors", []))
    d.update(category=category, rationale=rationale,
             baseline_path=original if original in baseline else None,
             evidence_status="provenance-only; gates evaluated separately",
             scanner_kind=kind, scan_counts=row.get("counts", {}),
             scan_error_counts=dict(errors),
             scan_git={k: row.get(k) for k in ("head", "branch", "dirty", "status_entries")})
    return d


def path_exclusion(rel):
    p = relative(rel)
    parts = p.parts
    if any(x.lower() in SECRET_DIRS for x in parts) or SECRET_NAME.search(p.name):
        return "credential-or-secret-path"
    if any(x in PRUNE or x.startswith("target-") for x in parts):
        return "metadata-generated-build-or-audit"
    if p.name in OUTPUT_NAMES or p.name.startswith(".release-manifest-"):
        return "self-generated-output"
    if str(p) in {"ctx/ctx.md", "ctx.md"} or p.name.endswith((".log", ".snap", ".bin")):
        return "generated-bundle-log-or-binary-artifact"
    if p.name not in NAMES and p.suffix.lower() not in EXTENSIONS:
        return "outside-source-allowlist"
    return None


def enumerate_source(root, row, nested):
    repo = root / row["path"]
    if row["kind"] not in {"broken-git-checkout", "non-git-source-tree"}:
        out, error = command(["git", "-C", str(repo), "ls-files", "-z", "--cached",
                              "--others", "--exclude-standard"], repo)
        if not error:
            return sorted(set(os.fsdecode(x) for x in out.split(b"\0") if x)), [], \
                "tracked + nonignored untracked files"
        return [], [{"path": ".", "reason": "git-enumeration-" + error}], "unavailable"
    paths, excluded = [], []
    for current, dirs, files in os.walk(repo, followlinks=False):
        current = Path(current)
        for name in sorted(dirs[:]):
            child = current / name
            rel = child.relative_to(repo).as_posix()
            reason = "symlink" if child.is_symlink() else (
                "nested-independent-root" if child in nested else (
                    "credential-or-secret-path" if name.lower() in SECRET_DIRS else (
                        "metadata-generated-build-or-audit" if name in PRUNE or
                        name.startswith("target-") else None)))
            if reason:
                dirs.remove(name)
                excluded.append({"path": rel + "/", "reason": reason})
        paths.extend((current / name).relative_to(repo).as_posix() for name in files)
    return sorted(paths), excluded, "filesystem fallback, pruned before descent; Git unavailable"


def cargo_declaration(rel, data):
    try:
        d = tomllib.loads(data.decode("utf-8"))
    except (ValueError, UnicodeError):
        return {"path": rel, "error": "toml-parse-error"}
    p, w = d.get("package", {}), d.get("workspace", {})
    result = {"path": rel, "interpretation": "declared only; features not resolved or built",
              "package": {k: p[k] for k in ("name", "version", "edition", "rust-version") if k in p},
              "workspace_package": {k: w.get("package", {})[k] for k in
                                    ("version", "edition", "rust-version") if k in w.get("package", {})},
              "features": d.get("features", {}), "profiles": d.get("profile", {}),
              "workspace_members": w.get("members", []),
              "workspace_default_members": w.get("default-members", [])}
    result["declared_targets"] = [dict(kind=k, **{f: t[f] for f in
        ("name", "path", "crate-type", "required-features") if f in t})
        for k in ("lib", "bin", "example", "test", "bench")
        for t in ([d[k]] if isinstance(d.get(k), dict) else d.get(k, []))]
    if rel.endswith("/.cargo/config.toml") or rel.endswith("/.cargo/config"):
        result = {"path": rel, "interpretation": "declared target selectors; no env values copied",
                  "build_target": d.get("build", {}).get("target"),
                  "target_selectors": sorted(d.get("target", {})),
                  "target_rustflags": {k: v.get("rustflags") for k, v in d.get("target", {}).items()
                                       if isinstance(v, dict) and "rustflags" in v}}
    return result


def inventory(root, row, all_roots, required_manifests):
    repo = root / row["path"]
    nested = {p for p in all_roots if p != repo and p.is_relative_to(repo)}
    paths, excluded, coverage = enumerate_source(root, row, nested)
    paths = sorted(set(paths) | {str(p.relative_to(repo)) for p in required_manifests
                                if p.is_relative_to(repo) and not any(p.is_relative_to(n) for n in nested)})
    # A library's Cargo.lock is often Git-ignored but still affects its standalone
    # build. Include existing adjacent locks/configs explicitly, never create one.
    extra = set()
    for rel in paths:
        p = Path(rel)
        candidates = []
        if p.name == "Cargo.toml":
            candidates = [p.parent / "Cargo.lock", p.parent / ".cargo/config.toml"]
        elif p.name == "package.json":
            candidates = [p.parent / name for name in
                          ("package-lock.json", "npm-shrinkwrap.json", "pnpm-lock.yaml", "yarn.lock")]
        for candidate in candidates:
            if os.path.lexists(repo / candidate):
                extra.add(candidate.as_posix())
    paths = sorted(set(paths) | extra)
    coverage += "; recorded build manifests and existing manifest-adjacent locks/configs included"
    files, declarations = [], []
    for rel in paths:
        try:
            p = repo / relative(rel)
            reason = "nested-independent-root" if any(p == n or p.is_relative_to(n) for n in nested) \
                else path_exclusion(rel)
            if not reason:
                if any((repo / Path(*PurePosixPath(rel).parts[:i])).is_symlink()
                       for i in range(1, len(PurePosixPath(rel).parts) + 1)):
                    reason = "symlink"
                else:
                    data = safe_read(root, (PurePosixPath(row["path"]) / rel).as_posix())
                    if b"\0" in data:
                        reason = "binary-content"
                    elif SECRET_CONTENT.search(data):
                        reason = "credential-or-secret-content"
                    else:
                        data.decode("utf-8")
            if reason:
                excluded.append({"path": rel, "reason": reason})
                continue
            role = "lockfile" if (Path(rel).name.endswith("lock") or Path(rel).suffix == ".lock" or
                Path(rel).name in {"package-lock.json", "npm-shrinkwrap.json", "pnpm-lock.yaml"}) \
                else "source"
            if "vendor" in PurePosixPath(rel).parts or "forks" in PurePosixPath(rel).parts:
                role = "vendored-build-source" if role == "source" else "vendored-build-lockfile"
            files.append({"path": rel, "sha256": sha(data), "bytes": len(data), "role": role})
            full_rel = (PurePosixPath(row["path"]) / rel).as_posix()
            if Path(rel).name == "Cargo.toml" or full_rel.endswith("/.cargo/config.toml"):
                declarations.append(cargo_declaration(full_rel, data))
        except FileNotFoundError:
            excluded.append({"path": rel, "reason": "listed-working-tree-path-absent"})
        except (OSError, ValueError, UnicodeError) as e:
            reason = str(e) if type(e) is ValueError else type(e).__name__
            excluded.append({"path": rel, "reason": reason})
    digest = sha(json.dumps(files, sort_keys=True, separators=(",", ":")).encode())
    return {"coverage": coverage, "files": files, "source_inventory_sha256": digest,
            "file_count": len(files), "source_bytes": sum(f["bytes"] for f in files),
            "exclusions": excluded, "exclusion_counts": dict(Counter(e["reason"] for e in excluded)),
            "declarations": declarations}


def toolchain(cwd):
    observations = {}
    for label, args in (
        ("path_rustc", ["rustc", "-vV"]),
        ("rustup_stable_rustc", ["rustup", "run", "stable", "rustc", "-vV"]),
        ("rustup_stable_installed_targets", ["rustup", "target", "list", "--installed", "--toolchain", "stable"]),
    ):
        out, error = command(args, cwd, 65536)
        observations[label] = {"command": args, "executable": shutil.which(args[0]),
                               "error": error, "stdout": None if out is None else out.decode().strip()}
    return {"observations": observations, "platform": sys.platform,
            "machine": platform.machine(), "python": platform.python_version(),
            "meaning": "installed toolchain/target observation; no build run or target validation; "
                       "process environment values and compiler overrides are not captured"}


def output_dir(value, protected):
    path = Path(os.path.abspath(os.path.expanduser(value)))
    for p in [path, *path.parents]:
        if p.is_symlink():
            raise ValueError("output directory has a symlink component")
    if any(path == p or path.is_relative_to(p) for p in protected):
        raise ValueError("output directory would modify an immutable baseline or scan")
    for name in OUTPUT_NAMES:
        if (path / name).is_symlink():
            raise ValueError("output file is a symlink")
    path.mkdir(parents=True, exist_ok=True)
    return path


def write_outputs(out, values):
    fd = os.open(out, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for name, value in values.items():
            tmp = ".release-manifest-" + uuid.uuid4().hex
            f = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=fd)
            try:
                with os.fdopen(f, "w", encoding="utf-8") as stream:
                    stream.write(value)
                    stream.flush()
                    os.fsync(stream.fileno())
                os.replace(tmp, name, src_dir_fd=fd, dst_dir_fd=fd)
            finally:
                try:
                    os.unlink(tmp, dir_fd=fd)
                except FileNotFoundError:
                    pass
        os.fsync(fd)
    finally:
        os.close(fd)


def markdown(manifest, dispositions):
    def esc(s):
        return str(s if s is not None else "unknown").replace("|", "\\|").replace("\n", " ")
    lines = ["# Repository dispositions and source provenance", "",
             "Generated read-only at " + manifest["finished_utc"] + ". "
             "[Source manifest](source-manifest.json); [machine-readable dispositions](final-dispositions.json).", "",
             "Every row is one discovered Git root/marker or non-Git group from the supplied scan. "
             "Actions are inherited baseline scope, not an assertion that every repository changed. "
             "Current HEAD/branch/dirty are a non-atomic observation; dirty includes unrelated work.", "",
             "New `.worktrees/*` remain separate branches under plan §10. `neuron` inherits `cell`; "
             "generated copies, nested repositories, vendors and missing sources have separate categories.", "",
             "The original Omi upstream gap is 12,397 unavailable blobs. This scan reports "
             + str(manifest["source_gaps"]["omi_upstream"]["current_unavailable_blobs"])
             + "; no statement covers unread upstream content. Baseline files are retained unchanged.", "",
             "Source SHA-256 checksums identify selected current files, including all 24 recorded local "
             "cyb build siblings. They are not Hemera/protocol hashes, proof roots or a compatibility test. "
             "The historical conformance constant stub is not used. No builds, gates, deployment or "
             "Hermes parity are evaluated by this generator.", "",
             "## Discovered roots", "",
             "| Path | Scanner kind / category | Current HEAD / branch / dirty | Baseline scope | Rationale |",
             "|---|---|---|---|---|"]
    for path, d in dispositions.items():
        g = d["current_git"]
        revision = f'{g.get("head") or "unknown"} / {g.get("branch") or "detached or unavailable"} / {g.get("dirty")}'
        lines.append("| " + " | ".join(map(esc, [path, d["scanner_kind"] + " / " + d["category"],
            revision, d["kind"] + " · " + d["packages"] + ": " + d["action"], d["rationale"]])) + " |")
    lines += ["", "## Baseline roots absent from the supplied scan", ""]
    for row in manifest["baseline_roots_absent"]:
        lines.append(f'- `{row["path"]}`: {row["rationale"]}')
    lines += ["", "## Source selection and limits", "",
              f'{len(manifest["repositories"])} selected roots; '
              f'{sum(r["file_count"] for r in manifest["repositories"].values())} files hashed. '
              "Each selected root records its own files, exclusions and declarations. "
              "Unselected roots still have a disposition above.", "",
              "Tracked and nonignored untracked files are considered. Nested Git roots are independent. "
              "Broken Git roots use a bounded filesystem inventory; their revision remains unknown. "
              "Source extension/name allowlists and conservative credential detection exclude metadata, "
              "audit trees, generated/build directories, dependency installs, binaries, secrets and symlinks. "
              "Individual files above 8 MiB are excluded, with a reason. This is a selected source manifest, "
              "not an archive or a complete machine snapshot. Outputs self-exclude. No source blobs are copied.", "",
              "Cargo features/profiles/targets are declarations, not activated feature sets. Toolchain "
              "versions and installed targets are observed separately; no environment values are copied. "
              "Owner build/test evidence and the G01–G14 matrix determine acceptance separately.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scan", type=Path, required=True, help="latest scan.json (read-only)")
    parser.add_argument("--out", required=True, help="directory for three generated reports")
    args = parser.parse_args()
    started = utc()
    scan_path = args.scan.expanduser().absolute()
    baseline_path = HERE / "dispositions.json"
    siblings_path = HERE / "implementation-baseline/p12-cyb-build-siblings.json"
    inputs = {}
    for label, p in (("scan", scan_path), ("baseline_dispositions", baseline_path),
                     ("build_siblings", siblings_path)):
        data = safe_read(p.parent, p.name, 64 * 1024 * 1024)
        inputs[label] = {"path": str(p), "sha256": sha(data), "bytes": len(data)}
        inputs[label]["data"] = json.loads(data)
    scan, baseline, siblings = (inputs[k].pop("data") for k in
                                ("scan", "baseline_dispositions", "build_siblings"))
    root = Path(scan["root"])
    if not root.is_absolute() or root.is_symlink() or not root.is_dir():
        raise ValueError("scan root must be an existing absolute directory, not a symlink")
    rows = scan["repositories"] + scan["non_git_sources"]
    by_path = {r["path"]: r for r in rows}
    if len(by_path) != len(rows):
        raise ValueError("duplicate scan roots")
    for path in by_path:
        relative(path)
    build_repos = set(siblings["repositories"])
    if len(build_repos) != 24 or not build_repos <= by_path.keys():
        raise ValueError("all 24 recorded local build siblings must be present in the scan")
    protected = [HERE / "implementation-baseline", HERE / "final-scan"]
    if scan_path.parent != HERE:
        protected.append(scan_path.parent)
    out = output_dir(args.out, protected)
    dispositions = {p: disposition(r, baseline) for p, r in sorted(by_path.items())}
    for p, d in dispositions.items():
        d["current_git"] = git_state(root / p, d["scanner_kind"])
    selected = {p for p, d in dispositions.items() if d["packages"] != "—" and p != "." and
                d["category"] not in {"generated-copy", "preserved-worktree", "vendor-reference", "source-gap"}}
    selected |= build_repos
    all_roots = {root / relative(p) for p in by_path if p != "."}
    manifests = {root / relative(p) for p in siblings["manifests"]}
    repositories = {}
    for p in sorted(selected):
        item = inventory(root, by_path[p], all_roots, manifests)
        item["git"] = dispositions[p]["current_git"]
        item["selection"] = (["recorded-cyb-build-sibling"] if p in build_repos else []) + \
            (["baseline-migration-owner"] if dispositions[p]["packages"] != "—" else [])
        repositories[p] = item
    required = []
    hashed = {(PurePosixPath(p) / f["path"]).as_posix(): f["sha256"]
              for p, r in repositories.items() for f in r["files"]}
    for p in siblings["manifests"]:
        required.append({"path": p, "sha256": hashed.get(p),
                         "observation": "source-checksummed" if p in hashed else "excluded-or-unavailable"})
    missing = []
    for p in sorted(set(baseline) - by_path.keys()):
        missing.append({"path": p, "baseline_disposition": baseline[p], "rationale":
            "Renamed owner is observed at neuron; original cell baseline retained." if p == "cell" else
            "Absent from supplied discovery; no deletion, merge or successful migration inferred."})
    gap = dispositions.get("omi/refs/omi-upstream", {}).get("scan_error_counts", {})
    manifest = {
        "schema": "soft3/neuron-migration-source-manifest/1", "scope": "local-source-provenance-only",
        "started_utc": started, "finished_utc": utc(), "root": str(root), "inputs": inputs,
        "generator": {"path": str(Path(__file__).resolve()),
                      "sha256": sha(safe_read(HERE, Path(__file__).name))},
        "scan_observation": {k: scan.get(k) for k in ("started_utc", "finished_utc", "scanner_sha256")},
        "discovery_counts": {"repositories": len(scan["repositories"]),
                             "non_git_groups": len(scan["non_git_sources"]),
                             "dispositions": len(dispositions), "selected_source_roots": len(selected)},
        "hash_semantics": "SHA-256 over current source bytes / canonical JSON inventories only; "
                          "not protocol hashes, consensus commitments, conformance results or signatures",
        "compatibility_assertion": None, "builds_run": [], "gates_evaluated": [],
        "toolchains": toolchain(root / "soft3"),
        "declared_runtime_profiles": [
            {"name": "cyber-secp256k1-hemera-v1", "source": "neuron/specs/identity.md",
             "meaning": "native subject derivation/signature contract; no network deployment claim"},
            {"name": "neuron/signed-native/1", "source": "soft3/crate/src/node/signed.rs",
             "meaning": "explicitly activated local HTTP authentication; endpoint acceptance is not consensus finality"},
            {"name": "neuron durable prog + Rune", "source": "neuron/rune/src/lib.rs",
             "meaning": "local execution source; workers and observations have separate evidence bounds"},
            {"name": "Soma local provider composition", "source": "soma/specs/local-provider.md",
             "meaning": "declared local model/task adapter; complete provider/Hermes parity not asserted"}],
        "build_sibling_repositories": sorted(build_repos), "build_sibling_manifests": required,
        "repositories": repositories, "baseline_roots_absent": missing,
        "source_gaps": {"omi_upstream": {"path": "omi/refs/omi-upstream",
                         "baseline_unavailable_blobs": 12397,
                         "current_unavailable_blobs": gap.get("sparse_index_blob_unavailable"),
                         "meaning": "Unseen blobs remain uncovered; no fetch attempted"}},
        "scan_exclusions": {"pruned_directories": scan.get("pruned_directories", []),
                            "prune_names": scan.get("prune_names", []),
                            "alias_count": len(scan.get("aliases", [])),
                            "discovery_error_count": len(scan.get("discovery_errors", []))},
        "source_policy": {"max_file_bytes": MAX_SOURCE_BYTES, "pruned_names": sorted(PRUNE),
                          "allowed_extensions": sorted(EXTENSIONS), "allowed_names": sorted(NAMES),
                          "secret_path_pattern": SECRET_NAME.pattern,
                          "source_contents_copied": False, "symlinks_followed": False,
                          "git_mutations": False, "network_operations": False,
                          "self_excluded_output_names": list(OUTPUT_NAMES),
                          "limitations": ["Not an atomic multi-repository snapshot",
                              "Ignored files outside recorded manifests and adjacent locks/configs are not enumerated",
                              "Per-file exclusions and compiler declarations are not successful builds",
                              "Credential detection is conservative; excluded fixtures are not validated",
                              "Historical conformance constant stub is not used"]},
    }
    manifest["finished_utc"] = utc()
    values = {"source-manifest.json": json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
              "final-dispositions.json": json.dumps(dispositions, ensure_ascii=False, indent=2) + "\n",
              "final-repositories.md": markdown(manifest, dispositions)}
    write_outputs(out, values)
    print(json.dumps({"out": str(out), "dispositions": len(dispositions),
        "selected_source_roots": len(selected), "files_hashed": len(hashed),
        "required_build_manifests_unavailable": sum(x["sha256"] is None for x in required),
        "outputs": list(values), "gates_evaluated": []}))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        # Avoid echoing source snippets, environment values or subprocess stderr.
        detail = str(error) if type(error) is ValueError else type(error).__name__
        print("release_manifest: " + detail + "; no acceptance result emitted", file=sys.stderr)
        sys.exit(2)
