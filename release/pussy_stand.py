"""Render pussy's pinned stand (release/pussy-stand.toml) as a release-train `override` string."""
import re
import sys
import tomllib
from pathlib import Path

DEFAULT = Path(__file__).with_name("pussy-stand.toml")


def load(path):
    stand = tomllib.loads(Path(path).read_text())
    siblings = stand.get("sibling", [])
    if not siblings:
        raise ValueError("stand has no siblings")
    seen = set()
    for entry in siblings:
        name, repo, rev = (entry.get(key, "") for key in ("name", "repo", "rev"))
        if not name or name in seen:
            raise ValueError(f"missing or duplicate sibling name {name!r}")
        seen.add(name)
        if not re.fullmatch(r"cyberia-to/[A-Za-z0-9_.-]+", str(repo)):
            raise ValueError(f"{name}: invalid repo {repo!r}")
        if not re.fullmatch(r"[0-9a-f]{40}", str(rev)):
            raise ValueError(f"{name}: rev must be 40 lowercase hex, got {rev!r}")
    return stand


def override(stand):
    return " ".join(f"{entry['name']}={entry['rev']}" for entry in stand["sibling"])


def main(argv=None):
    args = list(sys.argv[1:] if argv is None else argv)
    check = "--check" in args
    paths = [arg for arg in args if arg != "--check"]
    stand = load(paths[0] if paths else DEFAULT)
    if not check:
        print(override(stand))


if __name__ == "__main__":
    try:
        main()
    except ValueError as error:
        sys.exit(f"pussy_stand: {error}")
