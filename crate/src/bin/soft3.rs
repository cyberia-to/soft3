//! soft3 CLI — the stack facade. The operational verbs belong to the node:
//! `soft3 node|run|mine|sync|status|genesis|chain|conformance|network` forward
//! to `cybernode` for this release (0.11) and go away in the next
//! (soft3/projects/cybernode.md § 3.7). installing the `cybernode` crate (from its repository until it is published) is the
//! direct way.

const FORWARDED: &[&str] = &[
    "node", "run", "mine", "sync", "status", "genesis", "chain", "conformance", "network", "net",
];

/// What an argument vector asks of the facade. The forwarding is this crate's
/// only new behaviour in 0.11, so the routing is a function over argv and is
/// tested below rather than only exercised by running the binary.
#[derive(Debug, PartialEq, Eq)]
enum Route {
    Version,
    Manifesto,
    /// Hand the whole vector to `cybernode`; the verb is carried for the note.
    Forward(String),
    Help,
    /// A verb this facade does not know: the help, and exit 2 as before the carve-out.
    Unknown(String),
}

fn route(args: &[String]) -> Route {
    let network = args.iter().any(|a| a == "--network" || a == "-n");
    match args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str()) {
        Some("version") => Route::Version,
        Some("manifesto") => Route::Manifesto,
        Some(verb) if FORWARDED.contains(&verb) || network => Route::Forward(verb.to_string()),
        Some(verb) => Route::Unknown(verb.to_string()),
        None if args.iter().any(|a| a == "-V" || a == "--version") => Route::Version,
        None if network => Route::Forward("help".to_string()),
        None => Route::Help,
    }
}

fn print_help() {
    println!("soft3 {} — the type 2 civilization software stack", env!("CARGO_PKG_VERSION"));
    println!();
    println!("the node's verbs are cybernode's: cybernode node|run|mine|sync|status|genesis|chain|conformance|network");
    println!("soft3 forwards them for this release. install the cybernode crate from github.com/cyberia-to/cybernode for the node alone.");
    println!();
    println!("  soft3 manifesto");
    println!("  soft3 version");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match route(&args) {
        Route::Version => println!("soft3 {} (node: cybernode {})", env!("CARGO_PKG_VERSION"), cybernode::VERSION),
        Route::Manifesto => {
            for line in soft3::manifesto() {
                println!("{line}");
            }
        }
        Route::Forward(verb) => {
            eprintln!("note: `soft3 {verb}` is `cybernode {verb}` since soft3 0.11; this forwarding goes away in 0.12");
            cybernode::cli::run(args);
        }
        Route::Help => print_help(),
        Route::Unknown(verb) => {
            eprintln!("unknown command `{verb}`");
            print_help();
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{route, Route, FORWARDED};

    fn argv(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn every_forwarded_verb_goes_to_the_node() {
        for &verb in FORWARDED {
            assert_eq!(route(&argv(&[verb])), Route::Forward(verb.to_string()), "{verb}");
            assert_eq!(route(&argv(&[verb, "--json"])), Route::Forward(verb.to_string()), "{verb} --json");
        }
    }

    #[test]
    fn a_network_flag_before_the_verb_still_forwards() {
        assert!(matches!(route(&argv(&["--network", "pussy", "status"])), Route::Forward(_)));
        assert!(matches!(route(&argv(&["-n", "pussy", "node"])), Route::Forward(_)));
        assert!(matches!(route(&argv(&["--network"])), Route::Forward(_)));
    }

    #[test]
    fn an_unknown_verb_is_not_forwarded() {
        assert_eq!(route(&argv(&["frobnicate"])), Route::Unknown("frobnicate".to_string()));
    }

    #[test]
    fn the_facade_keeps_its_own_verbs() {
        assert_eq!(route(&argv(&["manifesto"])), Route::Manifesto);
        assert_eq!(route(&argv(&["version"])), Route::Version);
        assert_eq!(route(&argv(&["--version"])), Route::Version);
        assert_eq!(route(&argv(&["-V"])), Route::Version);
        assert_eq!(route(&argv(&[])), Route::Help);
        assert_eq!(route(&argv(&["--help"])), Route::Help);
    }
}
