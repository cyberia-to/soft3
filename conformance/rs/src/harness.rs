//! check and bless: compare the current build's snapshot with the committed
//! files, or rewrite them with a diff report.

use std::path::{Path, PathBuf};

use crate::Tier;
use crate::snap::{self, Change};
use crate::surfaces::{self, Context, Out};

/// Directory of the committed `.snap` files.
pub fn snapshots_dir(ctx: &Context) -> PathBuf {
    ctx.conformance.join("snapshots")
}

/// Result of a harness run: report lines and verdict.
#[derive(Debug, Default)]
pub struct Report {
    /// human-readable lines, in order
    pub lines: Vec<String>,
    /// whether the run passes
    pub green: bool,
}

impl Report {
    fn say(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

/// Generate the snapshot on a thread with room for the nox arenas.
pub fn generate(ctx: &Context) -> Out {
    let ctx = ctx.clone();
    std::thread::Builder::new()
        .name("conformance".into())
        .stack_size(256 << 20)
        .spawn(move || surfaces::generate(&ctx))
        .expect("spawn conformance thread")
        .join()
        .unwrap_or_else(|_| {
            let mut out = Out::default();
            out.fail("a surface panicked");
            out
        })
}

/// Entry-level changes between a committed file and the current entries.
fn file_changes(committed: &str, current: &str) -> Result<Vec<Change>, String> {
    Ok(snap::diff(&snap::parse(committed)?, &snap::parse(current)?))
}

/// `cargo conformance --check`.
pub fn check(ctx: &Context) -> Report {
    let out = generate(ctx);
    check_against(ctx, &out)
}

/// Compare an already generated run with the committed files.
pub fn check_against(ctx: &Context, out: &Out) -> Report {
    let mut r = Report { green: true, ..Default::default() };
    for f in &out.failures {
        r.say(format!("FAIL {f}"));
        r.green = false;
    }
    let dir = snapshots_dir(ctx);
    let mut entry_changes = 0usize;
    let mut manifest_differs = false;
    for (name, expected) in out.snapshot.render() {
        let path = dir.join(name);
        let committed = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                r.say(format!("FAIL {name}: {e} (run `cargo conformance --bless`)"));
                r.green = false;
                continue;
            }
        };
        if committed == expected {
            let n = snap::parse(&expected).map(|v| v.len()).unwrap_or(0);
            r.say(format!("ok   {name}{}", if name == "manifest.snap" { String::new() } else { format!(" ({n} entries)") }));
            continue;
        }
        if name == "manifest.snap" {
            manifest_differs = true;
            continue;
        }
        match file_changes(&committed, &expected) {
            Err(e) => {
                r.say(format!("FAIL {name}: committed file does not parse: {e}"));
                r.green = false;
            }
            Ok(changes) if changes.is_empty() => {
                r.say(format!("FAIL {name}: entries match but the bytes differ (hand-edited); run --bless"));
                r.green = false;
            }
            Ok(changes) => {
                entry_changes += changes.len();
                for c in &changes {
                    if c.fails() {
                        r.say(format!("FAIL {name}: {}", c.describe()));
                        r.green = false;
                    } else {
                        r.say(format!("note {name}: {}", c.describe()));
                    }
                }
            }
        }
    }
    if manifest_differs {
        if entry_changes == 0 {
            r.say("FAIL manifest.snap: root does not match the entries (hand-edited); run --bless");
            r.green = false;
        } else {
            r.say("note manifest.snap: root moves with the entry changes above");
        }
    }
    // fixtures on disk must be exactly the set the surfaces read
    let wanted: Vec<&str> = out.fixtures.iter().map(|(p, _)| p.as_str()).collect();
    for rel in stale_fixtures(ctx, &wanted) {
        r.say(format!("FAIL fixture {rel} is not read by any surface; run --bless"));
        r.green = false;
    }
    let root = out.snapshot.manifest().root().to_snap_string();
    r.say(format!(
        "conformance: {} — {} encodings, {} mechanisms, root {root}",
        if r.green { "green" } else { "red" },
        out.snapshot.encodings.len(),
        out.snapshot.mechanisms.len()
    ));
    r
}

fn stale_fixtures(ctx: &Context, wanted: &[&str]) -> Vec<String> {
    let dir = ctx.conformance.join("fixtures/zheng");
    surfaces::list(&dir, ".proof")
        .unwrap_or_default()
        .into_iter()
        .map(|n| format!("zheng/{n}"))
        .filter(|rel| !wanted.contains(&rel.as_str()))
        .collect()
}

/// `cargo conformance --bless [--tier <t>]`: regenerate snapshots and
/// proof fixtures, report the diff, and refuse changes above `allow`.
pub fn bless(ctx: &Context, allow: Tier) -> Report {
    let ctx = Context { fixtures: surfaces::Fixtures::Fresh, ..ctx.clone() };
    let out = generate(&ctx);
    let mut r = Report { green: true, ..Default::default() };
    if !out.failures.is_empty() {
        for f in &out.failures {
            r.say(format!("FAIL {f}"));
        }
        r.say("bless refused: invariants fail on the current build; nothing written");
        r.green = false;
        return r;
    }
    let dir = snapshots_dir(&ctx);
    let mut refused = false;
    let mut changed = 0usize;
    for (name, expected) in out.snapshot.render() {
        if name == "manifest.snap" {
            continue;
        }
        let committed = std::fs::read_to_string(dir.join(name)).unwrap_or_default();
        let changes = match file_changes(&committed, &expected) {
            Ok(c) => c,
            Err(_) => snap::diff(&[], &snap::parse(&expected).unwrap_or_default()),
        };
        for c in &changes {
            changed += 1;
            let tier = c.ceremony();
            let blocked = tier.governed() || tier > allow;
            r.say(format!("{} {name}: {}", if blocked { "REFUSE" } else { "bless " }, c.describe()));
            if tier.governed() {
                r.say("       epsilon entries move only with a detached governance signature");
            } else if blocked {
                r.say(format!("       needs `--tier {}` and a `[conformance:{}]` commit tag", tier.name(), tier.name()));
            }
            refused |= blocked;
        }
    }
    if refused {
        r.say("bless refused: nothing written");
        r.green = false;
        return r;
    }
    if let Err(e) = write_all(&ctx, &out) {
        r.say(format!("FAIL writing snapshots: {e}"));
        r.green = false;
        return r;
    }
    r.say(format!(
        "blessed: {changed} change(s); {} encodings, {} mechanisms, {} fixtures, root {}",
        out.snapshot.encodings.len(),
        out.snapshot.mechanisms.len(),
        out.fixtures.len(),
        out.snapshot.manifest().root().to_snap_string()
    ));
    r
}

fn write_all(ctx: &Context, out: &Out) -> std::io::Result<()> {
    let dir = snapshots_dir(ctx);
    std::fs::create_dir_all(&dir)?;
    for (name, text) in out.snapshot.render() {
        std::fs::write(dir.join(name), text)?;
    }
    let wanted: Vec<&str> = out.fixtures.iter().map(|(p, _)| p.as_str()).collect();
    for rel in stale_fixtures(ctx, &wanted) {
        std::fs::remove_file(ctx.fixture(&rel))?;
    }
    for (rel, bytes) in &out.fixtures {
        let path = ctx.fixture(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
    }
    std::fs::write(dir.join("provenance.toml"), provenance(ctx))
}

/// Sibling repositories the surfaces close over.
pub const SOURCES: &[&str] = &[
    "hemera", "strata", "nox", "lens", "zheng", "tade", "bbg", "foculus", "cybergraph", "inf", "tru", "tok", "soft3",
];

/// `provenance.toml`: the revisions a bless ran against. Informational;
/// `--check` never reads it.
pub fn provenance(ctx: &Context) -> String {
    let mut s = String::from(
        "# written by `cargo conformance --bless`: the sibling revisions the snapshots were generated from.\n\
         # informational — `--check` compares the .snap files only.\n\n",
    );
    s.push_str(&format!("blessed = \"{}\"\nharness = \"cyber-conformance {}\"\n\n[sources]\n", utc_date(), env!("CARGO_PKG_VERSION")));
    for name in SOURCES {
        s.push_str(&format!("{name} = \"{}\"\n", git_rev(&ctx.stack.join(name)).unwrap_or_else(|| "unknown".into())));
    }
    s
}

fn git_rev(dir: &Path) -> Option<String> {
    let o = std::process::Command::new("git").arg("-C").arg(dir).args(["rev-parse", "HEAD"]).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Today's UTC date, `YYYY-MM-DD`, from the system clock (integer civil-date
/// arithmetic, Howard Hinnant's days-to-civil).
pub fn utc_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}
