//! soft3 CLI — default network is **spacepussy-test** (soft3 chaosnet).

use soft3::network::{self, Network};
use soft3::node;
use std::path::PathBuf;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return;
    }

    let mut net = Network::DEFAULT;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--network" || args[i] == "-n" {
            i += 1;
            let name = args.get(i).map(|s| s.as_str()).unwrap_or("");
            net = parse_net(name);
        } else {
            rest.push(args[i].clone());
        }
        i += 1;
    }
    args = rest;

    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    match cmd {
        "-V" | "--version" | "version" => {
            println!("soft3 {}", env!("CARGO_PKG_VERSION"));
        }
        "manifesto" => {
            for line in soft3::manifesto() {
                println!("{line}");
            }
        }
        "network" | "net" => {
            let n = args.get(1).map(|s| parse_net(s)).unwrap_or(net);
            print_network(n);
        }
        "status" | "sync" => cmd_sync(net),
        "node" => cmd_node(&args[1..]),
        "claim" => cmd_claim(&args[1..], net),
        "help" | "-h" | "--help" => print_help(),
        other => {
            eprintln!("unknown command `{other}`");
            print_help();
            std::process::exit(2);
        }
    }
}

fn parse_net(name: &str) -> Network {
    if let Some(n) = Network::parse(name) {
        return n;
    }
    if Network::is_bootloader_name(name) {
        eprintln!("`{name}` is a cosmos bootloader chain on cybernode — not a soft3 network.");
        eprintln!(
            "product default: {} ({})",
            Network::DEFAULT,
            Network::DEFAULT.role()
        );
        eprintln!("use: soft3 sync   # spacepussy-test");
        std::process::exit(2);
    }
    eprintln!("unknown network `{name}` (use spacepussy-test|test|soft3)");
    std::process::exit(2);
}

fn cmd_sync(net: Network) {
    println!("soft3 sync · {}", net.chain_id());
    println!("  role             {}", net.role());
    println!("  rpc              {}", net.rpc());
    match network::probe(net) {
        Ok(s) => {
            println!(
                "  reachable        {}",
                if s.reachable { "yes" } else { "no" }
            );
            if !s.chain_id.is_empty() {
                println!("  chain_id         {}", s.chain_id);
            }
            if !s.moniker.is_empty() {
                println!("  moniker          {}", s.moniker);
            }
            if !s.engine.is_empty() {
                println!("  engine           {}", s.engine);
            }
            println!("  latest_height    {}", s.latest_height);
            if s.earliest_height > 0 {
                println!("  earliest_height  {}", s.earliest_height);
            }
            if !s.bbg_root.is_empty() {
                println!("  bbg_root         {}", s.bbg_root);
            }
            if s.signals > 0 || s.particles > 0 {
                println!("  signals          {}", s.signals);
                println!("  particles        {}", s.particles);
                println!("  axons            {}", s.axons);
            }
            println!(
                "  catching_up      {}",
                if s.catching_up { "yes" } else { "no" }
            );
        }
        Err(e) => {
            println!("  reachable        no");
            println!("  detail           {e}");
            std::process::exit(1);
        }
    }
}

/// Parsed `soft3 node` flags, before `--home`/`--bind`/`--moniker` defaults
/// are resolved into a running node.
struct NodeArgs {
    home: PathBuf,
    bind: String,
    moniker: String,
    help: bool,
}

/// Why [`parse_node_args`] could not resolve a flag set.
enum NodeArgError {
    /// A flag that takes a value (named by its usage string) was the last
    /// argument, with nothing after it.
    MissingValue(&'static str),
    /// An argument matched none of the known flags.
    Unknown(String),
}

/// Parse `soft3 node`'s argv into [`NodeArgs`], starting from the caller's
/// defaults. Pure and side-effect free — every error is returned, never a
/// panic or a process exit, so the operator sees the same clean message a
/// mistyped `--home`/`--bind`/`--moniker` already got before this refactor,
/// instead of a panic unwind.
fn parse_node_args(
    args: &[String],
    default_home: PathBuf,
    default_bind: String,
    default_moniker: String,
) -> Result<NodeArgs, NodeArgError> {
    let mut home = default_home;
    let mut bind = default_bind;
    let mut moniker = default_moniker;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--home" => {
                i += 1;
                home = PathBuf::from(
                    args.get(i)
                        .ok_or(NodeArgError::MissingValue("--home needs a path"))?,
                );
            }
            "--bind" => {
                i += 1;
                bind = args
                    .get(i)
                    .ok_or(NodeArgError::MissingValue("--bind needs host:port"))?
                    .clone();
            }
            "--moniker" => {
                i += 1;
                moniker = args
                    .get(i)
                    .ok_or(NodeArgError::MissingValue("--moniker needs a name"))?
                    .clone();
            }
            "-h" | "--help" => {
                return Ok(NodeArgs {
                    home,
                    bind,
                    moniker,
                    help: true,
                });
            }
            other => return Err(NodeArgError::Unknown(other.to_string())),
        }
        i += 1;
    }
    Ok(NodeArgs {
        home,
        bind,
        moniker,
        help: false,
    })
}

fn cmd_node(args: &[String]) {
    let parsed = parse_node_args(
        args,
        node::default_home(),
        Network::DEFAULT.local_bind().to_string(),
        hostname_fallback(),
    );
    let node_args = match parsed {
        Ok(a) => a,
        Err(NodeArgError::MissingValue(msg)) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
        Err(NodeArgError::Unknown(flag)) => {
            eprintln!("unknown node flag `{flag}`");
            std::process::exit(2);
        }
    };
    if node_args.help {
        println!("soft3 node — run spacepussy-test (cybergraph + bbg)");
        println!();
        println!("  soft3 node [--home DIR] [--bind HOST:PORT] [--moniker NAME]");
        println!();
        println!("engine: cybergraph processor + authenticated bbg state");
        println!("API:    GET /status /stats /root  POST /v1/link /v1/finalize");
        println!();
        println!("defaults:");
        println!("  --home    ~/.spacepussy-test");
        println!("  --bind    {}", Network::DEFAULT.local_bind());
        println!("  --moniker <hostname>");
        return;
    }
    if let Err(e) = node::run(node_args.home, &node_args.bind, &node_args.moniker) {
        eprintln!("soft3 node failed: {e}");
        std::process::exit(1);
    }
}

fn cmd_claim(args: &[String], net: Network) {
    let mut home = node::default_home();
    let mut hrp = net.bech32_prefix().to_string();
    let mut claim: Option<String> = None;
    let mut seen_claim = false;
    let mut seen_hrp = false;
    let mut seen_home = false;
    let parsed = (|| -> Result<(), &'static str> {
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--claim" if !seen_claim => {
                    i += 1;
                    claim = Some(args.get(i).ok_or("missing claim")?.clone());
                    seen_claim = true;
                }
                "--hrp" if !seen_hrp => {
                    i += 1;
                    hrp = args.get(i).ok_or("missing hrp")?.clone();
                    seen_hrp = true;
                }
                "--home" if !seen_home => {
                    i += 1;
                    home = PathBuf::from(args.get(i).ok_or("missing home")?);
                    seen_home = true;
                }
                _ => return Err("unknown or duplicate argument"),
            }
            i += 1;
        }
        Ok(())
    })();
    let claim = match (parsed, claim) {
        (Ok(()), Some(claim)) => claim,
        (Ok(()), None) => {
            eprintln!("missing --claim; usage: soft3 claim --claim ENCODED [--hrp HRP] [--home DIR]");
            std::process::exit(2);
        }
        (Err(e), _) => {
            eprintln!("{e}; usage: soft3 claim --claim ENCODED [--hrp HRP] [--home DIR]");
            std::process::exit(2);
        }
    };
    match node::claim_account(&home, &claim, &hrp) {
        Ok(neuron) => println!("claim verified; neuron {neuron}; home {}", home.display()),
        Err(e) => {
            eprintln!("claim failed: {e}");
            std::process::exit(1);
        }
    }
}

/// Real-environment hostname fallback: `$HOSTNAME`, else `$HOST`, else the
/// literal default. Thin wrapper over [`hostname_fallback_from`] so the
/// fallback chain itself is testable without mutating process-wide env vars.
fn hostname_fallback() -> String {
    hostname_fallback_from(|k: &str| std::env::var(k))
}

fn hostname_fallback_from<F: Fn(&str) -> Result<String, std::env::VarError>>(get: F) -> String {
    get("HOSTNAME")
        .or_else(|_| get("HOST"))
        .unwrap_or_else(|_| "soft3-node".into())
}

fn print_network(n: Network) {
    println!("network {}", n.chain_id());
    println!("  role     {}", n.role());
    println!("  prefix   {}", n.bech32_prefix());
    println!("  denom    {}", n.denom());
    println!("  rpc      {}", n.rpc());
    println!("  lcd      {}", n.lcd());
    println!("  index    {}", n.index());
    println!("  ws       {}", n.websocket());
    println!("  bind     {}", n.local_bind());
    if n == Network::DEFAULT {
        println!("  (product default)");
    }
}

fn print_help() {
    println!(
        "soft3 {} — type 2 civilization software",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!(
        "default network: {}  ({})",
        Network::DEFAULT,
        Network::DEFAULT.rpc()
    );
    println!("  {}", Network::DEFAULT.role());
    println!();
    println!("usage:");
    println!("  soft3 sync [--network spacepussy-test]");
    println!("  soft3 status              # alias of sync");
    println!("  soft3 network [name]      # print endpoints");
    println!("  soft3 node [--home DIR] [--bind HOST:PORT] [--moniker NAME]");
    println!("  soft3 claim --claim ENCODED [--hrp HRP] [--home DIR]");
    println!("  soft3 manifesto");
    println!("  soft3 version");
    println!();
    println!("not soft3 networks (cosmos bootloader on cybernode):");
    println!("  space-pussy · bostrom");
    println!();
    println!("docs  https://cyber.page/soft3/docs/launch");
    println!("site  https://soft3.org");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn defaults() -> (PathBuf, String, String) {
        (
            PathBuf::from("/default/home"),
            "127.0.0.1:1717".to_string(),
            "default-moniker".to_string(),
        )
    }

    #[test]
    fn node_args_no_flags_keeps_every_default() {
        let (home, bind, moniker) = defaults();
        let parsed = parse_node_args(&[], home.clone(), bind.clone(), moniker.clone())
            .ok()
            .unwrap();
        assert_eq!(parsed.home, home);
        assert_eq!(parsed.bind, bind);
        assert_eq!(parsed.moniker, moniker);
        assert!(!parsed.help);
    }

    #[test]
    fn node_args_all_three_flags_override_defaults() {
        let (home, bind, moniker) = defaults();
        let args = vec![
            "--home".to_string(),
            "/tmp/x".to_string(),
            "--bind".to_string(),
            "0.0.0.0:9000".to_string(),
            "--moniker".to_string(),
            "node-a".to_string(),
        ];
        let parsed = parse_node_args(&args, home, bind, moniker).ok().unwrap();
        assert_eq!(parsed.home, PathBuf::from("/tmp/x"));
        assert_eq!(parsed.bind, "0.0.0.0:9000");
        assert_eq!(parsed.moniker, "node-a");
        assert!(!parsed.help);
    }

    #[test]
    fn node_args_help_flag_short_circuits() {
        let (home, bind, moniker) = defaults();
        let args = vec!["--help".to_string()];
        let parsed = parse_node_args(&args, home, bind, moniker).ok().unwrap();
        assert!(parsed.help);
    }

    #[test]
    fn node_args_home_missing_value_errors_instead_of_panicking() {
        let (home, bind, moniker) = defaults();
        let args = vec!["--home".to_string()];
        let err = parse_node_args(&args, home, bind, moniker).err().unwrap();
        assert!(matches!(
            err,
            NodeArgError::MissingValue("--home needs a path")
        ));
    }

    #[test]
    fn node_args_bind_missing_value_errors_instead_of_panicking() {
        let (home, bind, moniker) = defaults();
        let args = vec!["--bind".to_string()];
        let err = parse_node_args(&args, home, bind, moniker).err().unwrap();
        assert!(matches!(
            err,
            NodeArgError::MissingValue("--bind needs host:port")
        ));
    }

    #[test]
    fn node_args_moniker_missing_value_errors_instead_of_panicking() {
        let (home, bind, moniker) = defaults();
        let args = vec!["--moniker".to_string()];
        let err = parse_node_args(&args, home, bind, moniker).err().unwrap();
        assert!(matches!(
            err,
            NodeArgError::MissingValue("--moniker needs a name")
        ));
    }

    #[test]
    fn node_args_unknown_flag_is_rejected() {
        let (home, bind, moniker) = defaults();
        let args = vec!["--bogus".to_string()];
        let err = parse_node_args(&args, home, bind, moniker).err().unwrap();
        match err {
            NodeArgError::Unknown(flag) => assert_eq!(flag, "--bogus"),
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn node_args_last_flag_wins_on_repeats() {
        let (home, bind, moniker) = defaults();
        let args = vec![
            "--moniker".to_string(),
            "first".to_string(),
            "--moniker".to_string(),
            "second".to_string(),
        ];
        let parsed = parse_node_args(&args, home, bind, moniker).ok().unwrap();
        assert_eq!(parsed.moniker, "second");
    }

    #[test]
    fn hostname_fallback_prefers_hostname_env() {
        let mut env = HashMap::new();
        env.insert("HOSTNAME", "from-hostname");
        env.insert("HOST", "from-host");
        let got = hostname_fallback_from(|k| {
            env.get(k)
                .map(|v| v.to_string())
                .ok_or(std::env::VarError::NotPresent)
        });
        assert_eq!(got, "from-hostname");
    }

    #[test]
    fn hostname_fallback_falls_back_to_host_env() {
        let mut env = HashMap::new();
        env.insert("HOST", "from-host");
        let got = hostname_fallback_from(|k| {
            env.get(k)
                .map(|v| v.to_string())
                .ok_or(std::env::VarError::NotPresent)
        });
        assert_eq!(got, "from-host");
    }

    #[test]
    fn hostname_fallback_defaults_when_neither_env_is_set() {
        let got = hostname_fallback_from(|_: &str| Err(std::env::VarError::NotPresent));
        assert_eq!(got, "soft3-node");
    }
}
