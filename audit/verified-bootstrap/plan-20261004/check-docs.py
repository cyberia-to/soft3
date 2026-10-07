#!/usr/bin/env python3
"""Validate this documentation delivery; this accepts no bootstrap gate."""
import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote

parser = argparse.ArgumentParser()
parser.add_argument("--soft3", required=True, type=Path)
parser.add_argument("--rs", required=True, type=Path)
parser.add_argument("--trident", required=True, type=Path)
args = parser.parse_args()
roots = {name: getattr(args, name).resolve() for name in ("soft3", "rs", "trident")}
bases = {
    "soft3": "4820c3d032c3c336d725f18e7b3fb13816eeb128",
    "rs": "01d82825d93129401df064ff3e0c3adae04d852a",
    "trident": "68b5d5dcf2f0b38ea42e043e6c5556b276b88dc5",
}
branches = {
    "soft3": "docs/verified-bootstrap-ceremony",
    "rs": "docs/verified-bootstrap-rs",
    "trident": "docs/0.4-dual-implementation-bootstrap",
}
documents = {
    "soft3": ["docs/verified-bootstrap.md", "README.md", "docs/README.md"],
    "rs": ["roadmap/verified-bootstrap.md", "roadmap/README.md", "README.md",
           "reference/compiler.md", ".claude/plans/pure-rust-toolchain.md"],
    "trident": ["roadmap/verified-bootstrap.md", "roadmap/bounded-metaprogramming.md",
                "roadmap/README.md", "reference/roadmap.md",
                "audit/self-hosting-progress.md", ".claude/plans/verified-bootstrap.md"],
}
receipt_dir = "audit/verified-bootstrap/plan-20261004"
link_pattern = re.compile(r"\[[^\]\n]*\]\(([^\s)]+)\)")
prefixes = {
    f"https://github.com/cyberia-to/{name}/blob/{branches[name]}/": root
    for name, root in roots.items()
}


def run(root, *command):
    result = subprocess.run(command, cwd=root, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f"{root}: {command}: {result.stdout}{result.stderr}")
    return result.stdout


for name, root in roots.items():
    run(root, "git", "diff", "--check")
    tracked = run(root, "git", "diff", "--name-only", bases[name]).splitlines()
    untracked = run(root, "git", "ls-files", "--others", "--exclude-standard").splitlines()
    assert all(p.endswith(".md") or p.startswith(receipt_dir + "/")
               for p in tracked + untracked), (name, tracked, untracked)
    links = []
    inherited = []
    hashes = {}
    for relative in documents[name]:
        path = root / relative
        body = path.read_text()
        hashes[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
        old = subprocess.run(["git", "show", f"{bases[name]}:{relative}"],
                             cwd=root, capture_output=True, text=True).stdout
        old_links = set(link_pattern.findall(old))
        for destination in link_pattern.findall(body):
            clean = unquote(destination.split("#", 1)[0])
            if not clean:
                continue
            target = None
            for prefix, other_root in prefixes.items():
                if clean.startswith(prefix):
                    target = other_root / clean[len(prefix):]
                    break
            if target is None:
                if "://" in clean or clean.startswith("mailto:"):
                    continue
                target = path.parent / clean
            if not target.exists():
                assert destination in old_links, (name, relative, destination)
                inherited.append({"document": relative, "link": destination})
            else:
                links.append({"document": relative, "link": destination})
    milestone = root / ("docs" if name == "soft3" else "roadmap") / "verified-bootstrap.md"
    assert "status: accepted\nmilestone-status: open" in milestone.read_text()
    gate_rows = re.findall(r"^\| (?:VB[0-8]|RS[0-9]) \|.*$", milestone.read_text(), re.M)
    assert all(row.endswith("| open |") for row in gate_rows)
    if name != "soft3":
        assert len(gate_rows) == (10 if name == "rs" else 9)
    memory = root / ".claude"
    memory_lines = sum(len(p.read_text().splitlines())
                       for p in memory.rglob("*.md")) if memory.exists() else 0
    assert memory_lines <= 1000, (name, memory_lines)
    result = {
        "status": "passed-documentation-validation",
        "command": ["python3", str(Path(__file__).resolve()), *sys.argv[1:]],
        "base_revision": bases[name],
        "head_at_validation": run(root, "git", "rev-parse", "HEAD").strip(),
        "documents_sha256": hashes,
        "checks": ["whitespace", "documentation-and-audit-only diff",
                   "new local links and coordinated branch targets resolve locally",
                   "accepted design with all implementation gates open",
                   "agent memory within 1000 lines"],
        "resolved_links": links,
        "inherited_missing_links_unchanged": inherited,
        "claude_markdown_lines": memory_lines,
        "new_cargo_build_or_test_run": False,
        "bootstrap_acceptance": "none; this validates planning documents only",
    }
    if name == "trident":
        command = ["git", "diff", "--quiet",
                   "a3cef15f6474c02363a042e648a39369785f2dbf", "--", ".",
                   ":(exclude)*.md",
                   ":(exclude)audit/self-hosting/bootstrap-results/verified-bootstrap-dual-plan/**",
                   ":(exclude)audit/self-hosting/bootstrap-results/verified-bootstrap-own-root/**",
                   ":(exclude)audit/verified-bootstrap/plan-20261004/**"]
        run(root, *command)
        result["executable_input_equivalence_command"] = command
        result["reused_green_tests"] = "audit/self-hosting/bootstrap-results/verified-bootstrap-dual-plan/release-check.json"
    output = root / receipt_dir / "validation.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"{name}: passed; {len(hashes)} documents; {len(links)} resolved links; "
          f"{len(inherited)} inherited missing links; .claude {memory_lines}/1000")
