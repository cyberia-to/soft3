//! `cargo conformance` — check or bless the soft3 stack's conformance snapshot.
//!
//! ```text
//! cargo conformance --check                 compare the current build with the committed snapshots
//! cargo conformance --bless [--tier <t>]    regenerate snapshots + proof fixtures, print the diff
//! cargo conformance show <substring>        print current fingerprints whose key contains <substring>
//! cargo conformance manifest                print the protocol stability root of the current build
//!
//! --stack <dir>        directory holding the sibling checkouts (default: beside soft3)
//! --conformance <dir>  soft3/conformance directory (default: this crate's parent)
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use cyber_conformance::Tier;
use cyber_conformance::harness;
use cyber_conformance::surfaces::{Context, Fixtures};

enum Mode {
    Check,
    Bless,
    Show(String),
    Manifest,
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: cargo conformance (--check | --bless [--tier <alpha|beta|gamma|delta>] | show <key> | manifest) \
         [--stack <dir>] [--conformance <dir>]"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // invoked as `cargo conformance …`, cargo passes the subcommand name first
    if args.first().map(String::as_str) == Some("conformance") {
        args.remove(0);
    }
    let mut mode = None;
    let mut tier = Tier::Gamma;
    let mut ctx = Context::from_manifest_dir(Fixtures::Committed);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--check" | "check" => mode = Some(Mode::Check),
            "--bless" | "bless" => mode = Some(Mode::Bless),
            "show" => match it.next() {
                Some(k) => mode = Some(Mode::Show(k)),
                None => return usage(),
            },
            "manifest" => mode = Some(Mode::Manifest),
            "--tier" => match it.next().as_deref().and_then(Tier::parse) {
                Some(t) => tier = t,
                None => return usage(),
            },
            "--stack" => match it.next() {
                Some(d) => ctx.stack = PathBuf::from(d),
                None => return usage(),
            },
            "--conformance" => match it.next() {
                Some(d) => ctx.conformance = PathBuf::from(d),
                None => return usage(),
            },
            "-h" | "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    let report = match mode {
        None => return usage(),
        Some(Mode::Check) => harness::check(&ctx),
        Some(Mode::Bless) => harness::bless(&ctx, tier),
        Some(Mode::Show(key)) => {
            let out = harness::generate(&ctx);
            let snap = out.snapshot;
            for e in snap.encoding_entries().into_iter().chain(snap.mechanism_entries()) {
                if e.key.contains(&key) {
                    println!("{}  tier={}  {}", e.key, e.tier.name(), e.fingerprint.to_snap_string());
                }
            }
            return ExitCode::SUCCESS;
        }
        Some(Mode::Manifest) => {
            let out = harness::generate(&ctx);
            println!("{}", out.snapshot.manifest().root().to_snap_string());
            return if out.failures.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE };
        }
    };
    for line in &report.lines {
        println!("{line}");
    }
    if report.green { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
