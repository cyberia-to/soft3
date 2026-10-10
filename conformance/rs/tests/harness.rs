//! The harness against itself: the committed snapshot is reproduced, and
//! every kind of tampering — with a snapshot line, the manifest, the file
//! bytes, or a proof fixture — turns the check red.

use std::path::{Path, PathBuf};

use cyber_conformance::Tier;
use cyber_conformance::harness::{self, Report};
use cyber_conformance::surfaces::{Context, Fixtures};

fn real() -> Context {
    Context::from_manifest_dir(Fixtures::Committed)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

/// A scratch copy of `conformance/{snapshots,fixtures}`; the stack stays the real one.
fn scratch(name: &str) -> Context {
    let ctx = real();
    let dir: PathBuf = std::env::temp_dir().join(format!("cyber-conformance-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&ctx.conformance.join("snapshots"), &dir.join("snapshots"));
    copy_dir(&ctx.conformance.join("fixtures"), &dir.join("fixtures"));
    Context { conformance: dir, ..ctx }
}

fn snap_path(ctx: &Context, file: &str) -> PathBuf {
    ctx.conformance.join("snapshots").join(file)
}

fn edit(ctx: &Context, file: &str, f: impl FnOnce(String) -> String) {
    let p = snap_path(ctx, file);
    let s = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, f(s)).unwrap();
}

/// Flip the last hex digit of the first entry line containing `key`.
fn flip_fingerprint(s: String, key: &str) -> String {
    let mut lines: Vec<String> = s.lines().map(str::to_string).collect();
    let line = lines.iter_mut().find(|l| !l.starts_with('#') && l.contains(key)).expect("entry");
    let last = line.pop().unwrap();
    line.push(if last == '0' { '1' } else { '0' });
    lines.join("\n") + "\n"
}

/// Change the tier on the entry line containing `key` (same column width).
fn retier(s: String, key: &str, tier: &str) -> String {
    s.lines()
        .map(|l| {
            let l = if !l.starts_with('#') && l.contains(key) {
                l.replacen("tier=gamma", &format!("tier={tier:<5}"), 1)
            } else {
                l.to_string()
            };
            l + "\n"
        })
        .collect()
}

fn has(r: &Report, needle: &str) -> bool {
    r.lines.iter().any(|l| l.contains(needle))
}

fn cleanup(ctx: &Context) {
    let _ = std::fs::remove_dir_all(&ctx.conformance);
}

#[test]
fn committed_snapshot_reproduces() {
    let r = harness::check(&real());
    assert!(r.green, "{:#?}", r.lines);
}

#[test]
fn untouched_scratch_copy_is_green() {
    let ctx = scratch("clean");
    let r = harness::check(&ctx);
    assert!(r.green, "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn tampered_fingerprint_is_detected() {
    let ctx = scratch("fingerprint");
    edit(&ctx, "mechanism.snap", |s| flip_fingerprint(s, "nox::reduce@v1 :: \"spec-add\""));
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "FAIL mechanism.snap: drift    nox::reduce@v1 :: \"spec-add\""), "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn tampered_encoding_and_removed_line_are_detected() {
    let ctx = scratch("encoding");
    edit(&ctx, "encoding.snap", |s| flip_fingerprint(s, "nebu::Goldilocks@v1"));
    edit(&ctx, "mechanism.snap", |s| s.lines().filter(|l| !l.contains("hemera::hash@v1 :: \"hello\"")).map(|l| format!("{l}\n")).collect());
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "drift    nebu::Goldilocks@v1"), "{:#?}", r.lines);
    assert!(has(&r, "added    hemera::hash@v1 :: \"hello\""), "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn hand_edited_bytes_and_manifest_are_detected() {
    let ctx = scratch("bytes");
    edit(&ctx, "encoding.snap", |s| s.replacen("tier=gamma    h", "tier=gamma h", 1));
    edit(&ctx, "manifest.snap", |s| flip_fingerprint(s, "root"));
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "entries match but the bytes differ"), "{:#?}", r.lines);
    cleanup(&ctx);

    let ctx = scratch("manifest");
    edit(&ctx, "manifest.snap", |s| flip_fingerprint(s, "root"));
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "root does not match the entries"), "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn tampered_proof_fixture_is_rejected() {
    let ctx = scratch("proof");
    let p = ctx.fixture("zheng/quote-add.proof");
    let mut b = std::fs::read(&p).unwrap();
    let mid = b.len() / 2;
    b[mid] ^= 1;
    std::fs::write(&p, b).unwrap();
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "rejects the committed fixture zheng/quote-add.proof"), "{:#?}", r.lines);
    assert!(has(&r, "drift    zheng::verify@v1 :: \"quote-add\""), "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn stale_fixture_is_detected() {
    let ctx = scratch("stale");
    std::fs::write(ctx.fixture("zheng/left-over.proof"), b"x").unwrap();
    let r = harness::check(&ctx);
    assert!(!r.green);
    assert!(has(&r, "fixture zheng/left-over.proof is not read by any surface"), "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn beta_drift_reports_without_failing() {
    let ctx = scratch("beta");
    let key = "tade::varint@v1";
    edit(&ctx, "encoding.snap", |s| retier(flip_fingerprint(s, key), key, "beta"));
    let r = harness::check(&ctx);
    assert!(has(&r, &format!("note encoding.snap: drift    {key}  tier=beta")), "{:#?}", r.lines);
    assert!(r.green, "{:#?}", r.lines);
    cleanup(&ctx);
}

#[test]
fn bless_restores_and_reports_the_diff() {
    let ctx = scratch("bless");
    let original = std::fs::read(snap_path(&ctx, "mechanism.snap")).unwrap();
    edit(&ctx, "mechanism.snap", |s| flip_fingerprint(s, "zheng::commit@v1 :: \"hash-42\""));
    std::fs::remove_file(ctx.fixture("zheng/pcs-eval.proof")).unwrap();
    let r = harness::bless(&ctx, Tier::Gamma);
    assert!(r.green, "{:#?}", r.lines);
    assert!(has(&r, "bless  mechanism.snap: drift    zheng::commit@v1 :: \"hash-42\""), "{:#?}", r.lines);
    assert_eq!(std::fs::read(snap_path(&ctx, "mechanism.snap")).unwrap(), original);
    assert!(ctx.fixture("zheng/pcs-eval.proof").is_file());
    assert!(harness::check(&ctx).green);
    cleanup(&ctx);
}

#[test]
fn bless_refuses_delta_without_its_ceremony() {
    let ctx = scratch("delta");
    let key = "cybergraph::private_network@v1";
    edit(&ctx, "mechanism.snap", |s| retier(flip_fingerprint(s, key), key, "delta"));
    let before = std::fs::read(snap_path(&ctx, "mechanism.snap")).unwrap();
    let r = harness::bless(&ctx, Tier::Gamma);
    assert!(!r.green, "{:#?}", r.lines);
    assert!(has(&r, "REFUSE mechanism.snap"), "{:#?}", r.lines);
    assert_eq!(std::fs::read(snap_path(&ctx, "mechanism.snap")).unwrap(), before, "nothing written");
    let r = harness::bless(&ctx, Tier::Delta);
    assert!(r.green, "{:#?}", r.lines);
    cleanup(&ctx);
}
