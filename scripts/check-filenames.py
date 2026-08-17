#!/usr/bin/env python3
"""Fail if git-tracked paths are invalid on Windows NTFS.

Usage:
  python3 soft3/scripts/check-filenames.py
  python3 soft3/scripts/check-filenames.py --root ~/cyber
  python3 soft3/scripts/check-filenames.py --root ~/cyber --fs
"""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

FORBIDDEN = set('<>:"/\\|?*')
RESERVED = {
    "CON", "PRN", "AUX", "NUL",
    *[f"COM{i}" for i in range(1, 10)],
    *[f"LPT{i}" for i in range(1, 10)],
}
SKIP_DIR = {
    ".git", "node_modules", "target", "dist", "build", ".next",
    "vendor", ".vendor", "__pycache__", ".cache", "coverage",
}


def check_component(name: str) -> list[str]:
    probs: list[str] = []
    bad = [c for c in name if c in FORBIDDEN or ord(c) < 32]
    if bad:
        probs.append(f"forbidden chars {bad!r}")
    if name.endswith(" ") or name.endswith("."):
        probs.append("trailing space or dot")
    if name.startswith(" "):
        probs.append("leading space")
    stem = name.split(".", 1)[0]
    if stem.upper() in RESERVED:
        probs.append(f"reserved device name {stem.upper()}")
    return probs


def check_rel(rel: str) -> list[tuple[str, str, list[str]]]:
    out = []
    for part in rel.split("/"):
        if part in ("", ".", ".."):
            continue
        probs = check_component(part)
        if probs:
            out.append((rel, part, probs))
    return out


def git_repos(root: Path) -> list[Path]:
    if (root / ".git").exists():
        return [root]
    repos = []
    for dirpath, dirnames, _ in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIR]
        if ".git" in dirnames:
            repos.append(Path(dirpath))
            dirnames.remove(".git")
    return repos


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument(
        "--fs",
        action="store_true",
        help="also scan filesystem (skips .git/node_modules/target/build)",
    )
    args = ap.parse_args()
    root = args.root.expanduser().resolve()
    issues: list[tuple[str, str, str, list[str]]] = []

    for repo in git_repos(root):
        try:
            out = subprocess.check_output(
                ["git", "-C", str(repo), "ls-files", "-z"],
                stderr=subprocess.DEVNULL,
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            continue
        files = [f.decode("utf-8", "surrogateescape") for f in out.split(b"\0") if f]
        # case collisions
        lower: dict[str, list[str]] = {}
        for f in files:
            lower.setdefault(f.lower(), []).append(f)
            for rel, part, probs in check_rel(f):
                issues.append((str(repo), rel, part, probs))
        for paths in lower.values():
            if len(set(paths)) > 1:
                issues.append(
                    (str(repo), " | ".join(sorted(set(paths))), "(case)", ["case-insensitive clash"])
                )

    if args.fs:
        for dirpath, dirnames, filenames in os.walk(root, followlinks=False):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIR]
            if any(p in Path(dirpath).parts for p in SKIP_DIR):
                dirnames.clear()
                continue
            for name in list(dirnames) + list(filenames):
                probs = check_component(name)
                if probs:
                    rel = str((Path(dirpath) / name).relative_to(root))
                    issues.append((str(root), rel, name, probs))

    if not issues:
        print(f"OK — no NTFS-unsafe paths under {root}")
        return 0

    print(f"FAIL — {len(issues)} NTFS-unsafe path(s):")
    for repo, rel, part, probs in issues:
        print(f"  {repo} :: {rel}  [{part}]  {', '.join(probs)}")
    print("see soft3/specs/filenames.md")
    return 1


if __name__ == "__main__":
    sys.exit(main())
