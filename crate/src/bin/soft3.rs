//! soft3 CLI — the stack facade. The operational verbs belong to the node:
//! `soft3 node|run|mine|sync|status|genesis|chain|conformance|network` forward
//! to `cybernode` for this release (0.11) and go away in the next
//! (soft3/proposals/cybernode.md § 3.7). `cargo install cybernode` is the
//! direct way.

const FORWARDED: &[&str] = &[
    "node", "run", "mine", "sync", "status", "genesis", "chain", "conformance", "network", "net",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verb = args
        .iter()
        .find(|a| !a.starts_with('-') && a.as_str() != "-n")
        .map(|s| s.as_str())
        .unwrap_or("help");
    match verb {
        "-V" | "--version" | "version" => println!("soft3 {} (node: cybernode {})", env!("CARGO_PKG_VERSION"), cybernode::VERSION),
        "manifesto" => {
            for line in soft3::manifesto() {
                println!("{line}");
            }
        }
        v if FORWARDED.contains(&v) || args.iter().any(|a| a == "--network" || a == "-n") => {
            eprintln!("note: `soft3 {v}` is `cybernode {v}` since soft3 0.11; this forwarding goes away in 0.12");
            cybernode::cli::run(args);
        }
        _ => {
            println!("soft3 {} — the type 2 civilization software stack", env!("CARGO_PKG_VERSION"));
            println!();
            println!("the node's verbs are cybernode's: cybernode node|run|mine|sync|status|genesis|chain|conformance|network");
            println!("soft3 forwards them for this release. `cargo install cybernode` for the node alone.");
            println!();
            println!("  soft3 manifesto");
            println!("  soft3 version");
        }
    }
}
