---
title: Phase-one source manifest
tags: soft3, release, spec
---

# Phase-one source manifest

`release/phase1.toml` pins each shared component by checkout name, GitHub
repository, default branch and full Git revision. Checkout names preserve sibling
Cargo paths; `tok` maps to `cyberia-to/plumb`. Build tools and upstream inputs
belong in this inventory. Missing origins and revision drift are red evidence.

Soft3 cuts an assembly from these inputs and its own default-branch revision.
Cyber and Cyb choose a finished assembly through `release/soft3.toml`: build
identity, release ID and SHA256SUMS digest, with expected version/source revision.
They inherit its captured component revisions and verdict. Their release jobs
resolve only their own product source, as defined in [[specs/releases]].

Owned repositories use their default-branch HEAD at a new soft3 cut and must
match the declared pin. `source = "upstream"` selects an exact reviewed upstream
commit and checks that it belongs to the upstream default-branch history.
Nu is Nushell from `nushell/nushell`; its embedded library version is controlled
here independently of the CI command runner version.

Changing pins requires review. Existing assembly bytes and evidence remain fixed.
Local source edits and feature-branch work require a new qualified assembly before
products can select them as release inputs.
