---
title: the director of soft3
tags: soft3, release, director, agent, role
crystal-type: process
crystal-domain: cyber
alias: soft3 director, product director soft3, the director
status: proposed
date: 2026-10-24
---
# the director of soft3

the owner, 2026-10-24: a cron is not a daemon. someone must think every few minutes whether the *whole* soft3 still builds after a day of work, see that every conflict is resolved and every branch merged by the rules, and hand the owner a green candidate at noon. that someone is a standing role of the soft3 domain — the keeper, runner and counter of `cyberia/dev.md` § the roles folded into one body that never sleeps — and it runs on the agent machine ([[the cloud]] §2.1) as the first consumer of it.

## the mandate

| the director | |
|---|---|
| owns | the default branches of soft3 and its phase-1 siblings as *one buildable stack*; the pins `release/phase1.toml`; the daily candidate at 12:00 UTC; the day's report |
| answers | "does the stack build right now, from origin default branches, with the pins?" — every tick, with a receipt, never from memory |
| acts | opens the PRs that make it build: pin bumps, lockfile refreshes, a sibling path that moved, a conflict resolved on a rebased branch; merges what the rules let it merge; labels what the owner must decide |
| reports | `audit/release-<date>/director.md`: green or red, why, what it did, what it asks; the dots on `status.md` |
| never | force-pushes, tags, publishes, promotes, edits `release/phase1.toml` by hand (a bump is a PR with the receipt), merges a `decision` PR, touches a specification of rungs 1–7, compiles on its own host |

## the loop — one tick every 15 minutes

`tick.sh` is the deterministic half and spends no tokens:

1. **sense** — fetch every component's default branch (`release/components.toml` × `phase1.toml`), the open PRs of soft3 and its siblings, the last candidate's receipts. the state is one JSON, `state/tick-<time>.json`, with a `changed` flag: a HEAD moved, a PR opened or closed, a run finished.
2. **qualify** — when something changed since the last green: dispatch the build service (`gh workflow run release-train.yml -f cut=false` — the same gates as the candidate, no draft) and, when it finishes, pull `release-validation.json` from every leg. compilation happens there, never here.
3. **think** — only if the tick found a red gate, a conflict, a stale pin or an unlabelled PR does it call the body: `goose run --recipe director.yaml` with the tick's JSON as the task. the body diagnoses from the receipts and acts within the mandate.
4. **report** — at 12:00 UTC the scheduled cut runs (the daily train); the first tick after it writes the day's report and updates `status.md`'s release row.

the budget is a mechanism: one `goose run` per tick at most, `--max-turns 40`, and the tick refuses to think if the previous think is still running or if the day's call count passed its allowance (`ALLOWANCE`, default 60). every action the body takes is a PR, a label or a comment — things the repository records.

## what it merges, and what it does not

by `cyberia/processes/deliver.md` § lanes: **contained** (one repository, one part, covered by tests, gates green) — the director merges when the PR carries `ready` and the gates of the PR are green; **wide** (a contract, a shared crate, anything a rung above depends on) and **irreversible** — the director labels `decision` and waits. until the machine user `cyberia-apparatus` has merge rights on the component repositories, the director merges nothing and every mergeable PR gets `ready` for the owner's click — the lane is computed from the files touched, never declared.

## what it needs to exist

| need | state 2026-10-24 |
|---|---|
| a body signed in on agent-a (claude: `claude` → `/login`) | **the owner's hand** |
| the GitHub machine user `cyberia-apparatus`: token with `repo` (PRs, labels, comments) and `actions` (dispatch, download artifacts) on soft3 and the component repositories; later merge rights for the contained lane | **the owner's hand** — a GitHub account, 2FA, org membership |
| the files here on agent-a under `/srv/forge/claude/director/` and the loop under `tmux` | installed by `install.sh`, waits for the two above |
| the daily cut at 12:00 UTC | soft3#246, cyber#153 |

## the measurement

on the director's own report, monthly: days the noon candidate was green / days; hours from a red tick to the next green; PRs merged by rule and PRs escalated; tokens per green day. the day the stack has been red for seven days with no PR of the director's that would make it green, the role has failed and this page says so.

## today's red, for the first tick to chew on

the last candidates (2026-10-23) are red on every leg: soft3's crate depends on `../../lens/gpu`, which lens master lacks (lens#28 merged into `release/0.2`, not master); `phase1-pins` drifts on ~25 siblings; `release-notes-source` is red (cyb's HEAD has no merged PR); cyber's tests and release fall with soft3; optica builds but the protocol graph does not. the first director's PRs are these.
