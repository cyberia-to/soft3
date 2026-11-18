#!/usr/bin/env python3
"""The pure parts of the director's tick (`release/director/tick.sh`): reading
the sibling manifest and reducing a run's receipts. They live here, beside the
train engine, so the release suite covers them; tick.sh calls this file in the
soft3 mirror it has just reset to origin."""
import json
from pathlib import Path
import sys
import tomllib

KEYS = ("repo", "branch", "rev")


def siblings(manifest):
    """(sensed, unpinned) from release/phase1.toml.

    A sibling carries repo, branch and rev once it is pinned; one that does not
    (nu, which has no remote yet) cannot be sensed and is returned by name, so
    the caller reports it instead of dying on it or dropping it in silence."""
    sensed, unpinned = [], []
    for entry in tomllib.loads(Path(manifest).read_text())["sibling"]:
        if all(entry.get(key) for key in KEYS):
            sensed.append({key: entry[key] for key in KEYS})
        else:
            unpinned.append(entry.get("name") or entry.get("repo") or "<unnamed>")
    return sensed, unpinned


def reduce_receipts(documents):
    """One row per release-validation.json: its result and every gate that is
    not green, with whatever the gate said about why."""
    rows = []
    for document in documents:
        red = [{"name": gate.get("name"),
                "detail": str(gate.get("detail") or gate.get("error") or gate.get("reason") or gate.get("command") or "")}
               for gate in document.get("gates", []) if gate.get("result") != "green"]
        rows.append({"result": document.get("result"), "red": red})
    return rows


def main(argv):
    command, arguments = argv[0], argv[1:]
    if command == "siblings":
        sensed, unpinned = siblings(arguments[0])
        for row in sensed:
            print(row["repo"], row["branch"], row["rev"])
        if unpinned:
            print("unpinned, not sensed: " + ", ".join(unpinned), file=sys.stderr)
        return 0
    if command == "receipts":
        documents = [json.loads(Path(path).read_text()) for path in arguments]
        if not documents:
            raise ValueError("no receipt given")
        json.dump(reduce_receipts(documents), sys.stdout)
        return 0
    raise ValueError(f"unknown command {command!r}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
