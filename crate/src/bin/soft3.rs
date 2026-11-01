//! soft3 CLI — default network is **spacepussy-test** (soft3 chaosnet).

use soft3::chain::mine;
use soft3::network::{self, Network};
use soft3::node;
use std::path::PathBuf;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return;
    }

    let (names, rest) = extract_network_flags(&args);
    let mut net = Network::DEFAULT;
    for name in &names {
        net = parse_net(name);
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
        "mine" => cmd_mine(&args[1..]),
        "chain" => cmd_chain(&args[1..]),
        "genesis" => cmd_genesis(&args[1..]),
        "conformance" => cmd_conformance(&args[1..]),
        "help" | "-h" | "--help" => print_help(),
        other => {
            eprintln!("unknown command `{other}`");
            print_help();
            std::process::exit(2);
        }
    }
}

/// Split `--network`/`-n <value>` flags out of argv. Returns every requested
/// value in the order given (an empty string if a flag had no following
/// argument) alongside the remaining arguments, unvalidated — each name is
/// checked by [`parse_net`] in `main`, last one wins, matching argv order.
fn extract_network_flags(args: &[String]) -> (Vec<String>, Vec<String>) {
    let mut names = Vec::new();
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--network" || args[i] == "-n" {
            i += 1;
            names.push(args.get(i).cloned().unwrap_or_default());
        } else {
            rest.push(args[i].clone());
        }
        i += 1;
    }
    (names, rest)
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
    // phase-1 root chain (specs/chain.md); `--chain` selects it over spacepussy-test
    chain: Option<String>,
    home_set: bool,
    bind_set: bool,
    radio: Option<String>,
    peers: Option<PathBuf>,
    genesis: Option<PathBuf>,
    neuron: Option<String>,
    relay: bool,
    log_net: bool,
    corpus: Option<PathBuf>,
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
    let mut out = NodeArgs {
        home: default_home, bind: default_bind, moniker: default_moniker, help: false, chain: None, home_set: false, bind_set: false,
        radio: None, peers: None, genesis: None, neuron: None, relay: false, log_net: false, corpus: None,
    };
    let mut i = 0;
    while i < args.len() {
        let value = |i: &mut usize, what: &'static str| -> Result<String, NodeArgError> {
            *i += 1;
            args.get(*i).cloned().ok_or(NodeArgError::MissingValue(what))
        };
        match args[i].as_str() {
            "--home" => {
                out.home = PathBuf::from(value(&mut i, "--home needs a path")?);
                out.home_set = true;
            }
            "--bind" => {
                out.bind = value(&mut i, "--bind needs host:port")?;
                out.bind_set = true;
            }
            "--moniker" => out.moniker = value(&mut i, "--moniker needs a name")?,
            "--chain" => out.chain = Some(value(&mut i, "--chain needs a name (pussy|bostrom|dev…)")?),
            "--radio" => out.radio = Some(value(&mut i, "--radio needs host:udp-port")?),
            "--peers" => out.peers = Some(PathBuf::from(value(&mut i, "--peers needs a file")?)),
            "--genesis" => out.genesis = Some(PathBuf::from(value(&mut i, "--genesis needs a file")?)),
            "--neuron" => out.neuron = Some(value(&mut i, "--neuron needs a 64-hex neuron")?),
            "--relay" => out.relay = true,
            "--log-net" => out.log_net = true,
            "--corpus" => out.corpus = Some(PathBuf::from(value(&mut i, "--corpus needs a directory")?)),
            "-h" | "--help" => {
                out.help = true;
                return Ok(out);
            }
            other => return Err(NodeArgError::Unknown(other.to_string())),
        }
        i += 1;
    }
    Ok(out)
}

/// `soft3 node id --home DIR` — the radio endpoint id (ed25519 public key,
/// 64 lowercase hex), created on first call; what peers.toml wants.
fn cmd_node_id(args: &[String]) {
    let mut home = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--home" {
            i += 1;
            home = args.get(i).map(PathBuf::from);
        } else {
            eprintln!("usage: soft3 node id --home DIR");
            std::process::exit(2);
        }
        i += 1;
    }
    let Some(home) = home else {
        eprintln!("usage: soft3 node id --home DIR");
        std::process::exit(2);
    };
    match node::chain_node::endpoint_id(&home) {
        Ok(id) => println!("{id}"),
        Err(e) => {
            eprintln!("soft3 node id: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_node(args: &[String]) {
    if args.first().map(|s| s.as_str()) == Some("id") {
        return cmd_node_id(&args[1..]);
    }
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
        println!("soft3 node — run spacepussy-test (cybergraph + bbg) or a phase-1 root chain");
        println!();
        println!("  soft3 node [--home DIR] [--bind HOST:PORT] [--moniker NAME]");
        println!("  soft3 node --chain <pussy|bostrom|dev…> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME");
        println!("             [--genesis FILE] [--neuron NEURON] [--relay] [--log-net]");
        println!("             [--corpus DIR]   # record every block given, with its verdict and state root (blocks.jsonl)");
        println!("             (mining is `soft3 mine --node http://BIND --address NEURON`: /template + /submit)");
        println!("  soft3 node id --home DIR      # radio endpoint id (64 hex) for peers.toml");
    println!("  soft3 genesis build --chain pussy|bostrom|pussy-rc --snapshot DIR --out FILE");
    println!("                      [--cache DIR] [--export DIR] [--no-root] [--time UNIX]");
    println!("  soft3 genesis verify FILE [--root]");
    println!("  soft3 conformance bless|check chain [--genesis FILE]");
        println!();
        println!("engine: cybergraph processor + authenticated bbg state; chain: PoW root chain (specs/chain.md, chain-wire.md)");
        println!("API:    GET /status /stats /root  POST /v1/link (chaosnet)  ·  chain: GET /tip /block /tx /peers …  POST /tx /submit");
        println!();
        println!("defaults:");
        println!("  --home    ~/.spacepussy-test · chain: ~/.soft3/<chain>");
        println!("  --bind    {} · pussy 127.0.0.1:7781 · bostrom 127.0.0.1:7782 · dev 127.0.0.1:17781", Network::DEFAULT.local_bind());
        println!("  --radio   pussy 0.0.0.0:7791 · bostrom 0.0.0.0:7792 · dev 127.0.0.1:0");
        println!("  --moniker <hostname>");
        return;
    }
    if let Some(chain) = node_args.chain {
        let (http, radio) = match chain.as_str() {
            "pussy" => ("127.0.0.1:7781", "0.0.0.0:7791"),
            "bostrom" => ("127.0.0.1:7782", "0.0.0.0:7792"),
            "pussy-rc" => ("127.0.0.1:7783", "0.0.0.0:7793"),
            _ => ("127.0.0.1:17781", "127.0.0.1:0"),
        };
        let bind = if node_args.bind_set { node_args.bind.clone() } else { http.to_string() };
        let radio = node_args.radio.clone().unwrap_or_else(|| radio.to_string()).parse().unwrap_or_else(|e| {
            eprintln!("--radio: {e}");
            std::process::exit(2);
        });
        let neuron = node_args.neuron.as_deref().map(|m| soft3::chain::net::codec::unhex32(m).unwrap_or_else(|| {
            eprintln!("--neuron needs a 64-hex neuron id");
            std::process::exit(2);
        }));
        let opts = node::chain_node::ChainOpts {
            home: if node_args.home_set { node_args.home } else { node::chain_node::default_home(&chain) },
            chain, genesis: node_args.genesis, radio: Some(radio), peers: node_args.peers, moniker: node_args.moniker,
            neuron,
            relay: node_args.relay, log_net: node_args.log_net, corpus: node_args.corpus,
        };
        if let Err(e) = node::run_chain(opts, &bind) {
            eprintln!("soft3 node failed: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(e) = node::run(node_args.home, &node_args.bind, &node_args.moniker) {
        eprintln!("soft3 node failed: {e}");
        std::process::exit(1);
    }
}

/// Parsed `soft3 mine` flags.
#[derive(Debug, PartialEq, Eq)]
struct MineArgs {
    bench: bool,
    chain: String,
    node: Option<String>,
    address: Option<String>,
    backend: String,
    threads: usize,
    seconds: u64,
    help: bool,
}

/// Parse `soft3 mine`'s argv. Pure: every error is returned.
fn parse_mine_args(args: &[String]) -> Result<MineArgs, NodeArgError> {
    let mut a = MineArgs {
        bench: false,
        chain: "pussy".into(),
        node: None,
        address: None,
        backend: "auto".into(),
        threads: 0,
        seconds: 5,
        help: false,
    };
    let mut i = 0;
    let value = |args: &[String], i: usize, what: &'static str| -> Result<String, NodeArgError> {
        args.get(i).cloned().ok_or(NodeArgError::MissingValue(what))
    };
    while i < args.len() {
        match args[i].as_str() {
            "bench" if i == 0 => a.bench = true,
            "--chain" => {
                i += 1;
                a.chain = value(args, i, "--chain needs pussy|bostrom")?;
            }
            "--node" => {
                i += 1;
                a.node = Some(value(args, i, "--node needs a URL")?);
            }
            "--address" | "--to" => {
                i += 1;
                a.address = Some(value(args, i, "--address needs a neuron (64 hex)")?);
            }
            "--backend" => {
                i += 1;
                a.backend = value(args, i, "--backend needs auto|cpu|metal|webgpu")?;
            }
            "--threads" => {
                i += 1;
                a.threads = value(args, i, "--threads needs a number")?
                    .parse()
                    .map_err(|_| NodeArgError::Unknown("--threads: not a number".into()))?;
            }
            "--seconds" => {
                i += 1;
                a.seconds = value(args, i, "--seconds needs a number")?
                    .parse()
                    .map_err(|_| NodeArgError::Unknown("--seconds: not a number".into()))?;
            }
            "-h" | "--help" => a.help = true,
            other => return Err(NodeArgError::Unknown(other.to_string())),
        }
        i += 1;
    }
    Ok(a)
}

/// The node of a chain when `--node` is absent (chain-wire §7 binds).
fn default_node(chain: &str) -> Option<String> {
    match chain {
        "pussy" => Some("http://127.0.0.1:7781".into()),
        "bostrom" => Some("http://127.0.0.1:7782".into()),
        _ => None,
    }
}

fn cmd_mine(args: &[String]) {
    let a = match parse_mine_args(args) {
        Ok(a) => a,
        Err(NodeArgError::MissingValue(msg)) => {
            eprintln!("{msg}");
            std::process::exit(2);
        }
        Err(NodeArgError::Unknown(flag)) => {
            eprintln!("unknown mine flag `{flag}`");
            std::process::exit(2);
        }
    };
    if a.help {
        println!("soft3 mine — reference miner of the phase-1 root chain (specs/chain.md)");
        println!();
        println!("  soft3 mine --address NEURON [--chain pussy|bostrom] [--node URL]");
        println!("             [--backend auto|cpu|metal|webgpu] [--threads N]");
        println!(
            "  soft3 mine bench [--backend auto|cpu|metal|webgpu] [--threads N] [--seconds S]"
        );
        println!();
        println!(
            "grinds hemera(header) < target over the 16-byte nonce; GET /template, POST /submit"
        );
        println!("(chain-wire §7); the template is refreshed when GET /tip moves.");
        println!();
        println!("defaults:");
        println!("  --chain    pussy");
        println!("  --node     http://127.0.0.1:7781 (pussy) · :7782 (bostrom)");
        println!("  --backend  auto (metal, else webgpu, else cpu)");
        println!("  --threads  every core (cpu)");
        println!("  --seconds  5 (bench)");
        return;
    }
    let choice: mine::Choice = match a.backend.parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if a.bench {
        let backends = match choice {
            mine::Choice::Auto => mine::all(a.threads),
            c => match mine::select(c, a.threads) {
                Ok(b) => vec![b],
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            },
        };
        let mut rows = Vec::new();
        for b in &backends {
            eprintln!("measuring {} for {}s…", b.name(), a.seconds);
            match mine::bench::measure(b.as_ref(), a.seconds as f64) {
                Ok(r) => rows.push(r),
                Err(e) => eprintln!("{}: {e}", b.name()),
            }
        }
        print!("{}", mine::bench::table(&rows));
        if rows.iter().any(|r| !r.vector) {
            std::process::exit(1);
        }
        return;
    }
    let Some(address) = a.address else {
        eprintln!("--address NEURON is required (the coinbase goes to it)");
        std::process::exit(2);
    };
    if address.len() != 64 || !address.bytes().all(|b| b.is_ascii_hexdigit()) {
        eprintln!("--address must be a neuron id: 64 hex characters");
        std::process::exit(2);
    }
    let Some(node) = a.node.or_else(|| default_node(&a.chain)) else {
        eprintln!(
            "unknown chain `{}` (pussy|bostrom), or pass --node",
            a.chain
        );
        std::process::exit(2);
    };
    let backend = match mine::select(choice, a.threads) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let settings = mine::run::Settings {
        chain: a.chain,
        node,
        address: address.to_lowercase(),
        tip_poll: std::time::Duration::from_secs(2),
        refresh: std::time::Duration::from_secs(30),
        report: std::time::Duration::from_secs(5),
    };
    let stop = std::sync::atomic::AtomicBool::new(false);
    if let Err(e) = mine::run::run(backend.as_ref(), &settings, &stop, &|m| println!("{m}")) {
        eprintln!("soft3 mine failed: {e}");
        std::process::exit(1);
    }
}

/// `soft3 chain …` — helpers for test chains and the local harness: a test
/// genesis (any name but a launch chain's, the easiest target the kernel
/// accepts), a key's neuron, a signed transfer or cyberlink (the wallet's
/// codec, `soft3::wallet`).
fn cmd_chain(args: &[String]) {
    use soft3::chain::net::codec::{hex, unhex32};
    use soft3::genesis::file::{Balance, GenesisFile, Params};
    let mut flags = std::collections::HashMap::new();
    let mut funds = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let key = args[i].trim_start_matches("--").to_string();
        let value = args.get(i + 1).cloned().unwrap_or_default();
        if key == "fund" { funds.push(value); } else { flags.insert(key, value); }
        i += 2;
    }
    let die = |msg: String| -> ! {
        eprintln!("{msg}");
        std::process::exit(2);
    };
    let wallet = |flags: &std::collections::HashMap<String, String>| -> soft3::wallet::Signer {
        let seed = flags.get("seed").and_then(|s| unhex32(s)).unwrap_or_else(|| die("--seed <64 hex> required".into()));
        soft3::wallet::Signer::from_secret(&seed).unwrap_or_else(|e| die(format!("seed: {e}")))
    };
    let num = |flags: &std::collections::HashMap<String, String>, k: &str, d: u64| -> u64 {
        flags.get(k).map(|v| v.parse().unwrap_or_else(|_| die(format!("--{k}: not a number")))).unwrap_or(d)
    };
    match args.first().map(|s| s.as_str()) {
        Some("genesis-dev") => {
            let name = flags.get("name").cloned().unwrap_or_else(|| "dev".into());
            if node::chain_node::LAUNCH_CHAINS.contains(&name.as_str()) {
                if name == "pussy-rc" {
                    die("`pussy-rc` is a launch chain; its genesis comes from `soft3 genesis build --chain pussy-rc --time T`".into());
                }
                die(format!("`{name}` is a launch chain; its genesis comes from `soft3 genesis build`"));
            }
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
            // two hours of ASERT slack: the first ~240 blocks stay at the floor target
            let time = num(&flags, "time", now.saturating_sub(7200));
            let bits = flags.get("bits").map(|b| u32::from_str_radix(b.trim_start_matches("0x"), 16).unwrap_or_else(|_| die("--bits: 8 hex".into())))
                .unwrap_or(soft3::chain::params::POW_LIMIT_BITS);
            let native = soft3::chain::sigs::token(&name);
            let mut balances: Vec<Balance> = funds.iter().map(|f| {
                let (n, a) = f.split_once('=').unwrap_or_else(|| die(format!("--fund {f}: NEURON=AMOUNT")));
                Balance { owner: unhex32(n).unwrap_or_else(|| die(format!("--fund {f}: neuron is 64 hex"))), token: native, amount: a.parse().unwrap_or_else(|_| die(format!("--fund {f}: amount"))) }
            }).collect();
            balances.sort();
            let s0: u64 = balances.iter().map(|b| b.amount).sum();
            let file = GenesisFile {
                params: Params {
                    version: 1, name: name.clone(), hrp: name.clone(), denom: name.clone(), time, bits,
                    subsidy: soft3::chain::params::subsidy(s0), checkpoint_key: [0; 33], checkpoint_sunset: soft3::chain::params::CHECKPOINT_SUNSET,
                    pool: soft3::chain::pool::Pool::genesis(),
                },
                balances,
                links: vec![],
            };
            let genesis = soft3::chain::genesis::Genesis::from_file(&file).unwrap_or_else(|e| die(format!("genesis: {e}")));
            let out = flags.get("out").cloned().unwrap_or_else(|| "genesis.tade".into());
            std::fs::write(&out, &genesis.bytes).unwrap_or_else(|e| die(format!("{out}: {e}")));
            println!("{}", hex(&genesis.chain_id));
        }
        Some("key") => println!("{}", wallet(&flags).neuron_hex()),
        Some("tx") => {
            let w = wallet(&flags);
            let bytes = flags.get("genesis").and_then(|g| std::fs::read(g).ok()).unwrap_or_else(|| die("--genesis <file> required (the chain id is its particle)".into()));
            let genesis = soft3::chain::genesis::Genesis::decode(&bytes).unwrap_or_else(|e| die(format!("genesis: {e}")));
            let token = genesis.native_token();
            let nonce = num(&flags, "nonce", 0);
            let amount = num(&flags, "amount", 0);
            let fee = flags.get("fee").map(|_| num(&flags, "fee", 0));
            let signed = match flags.get("kind").map(|k| k.as_str()).unwrap_or("transfer") {
                // chain-economy §3: amount > 0 locks native stake with the valence
                "link" => {
                    let p = flags.get("p").and_then(|p| unhex32(p)).unwrap_or([1; 32]);
                    let q = flags.get("q").and_then(|q| unhex32(q)).unwrap_or([2; 32]);
                    let valence: i8 = flags.get("valence").map(|v| v.parse().unwrap_or_else(|_| die("--valence: -1, 0 or 1".into()))).unwrap_or(0);
                    w.cyberlink(&genesis.chain_id, nonce, p, q, valence, token, amount, fee)
                }
                "unlock" => {
                    let p = flags.get("p").and_then(|p| unhex32(p)).unwrap_or([1; 32]);
                    let q = flags.get("q").and_then(|q| unhex32(q)).unwrap_or([2; 32]);
                    w.unlock(&genesis.chain_id, nonce, p, q, amount, fee)
                }
                "payout" => {
                    let p = flags.get("p").and_then(|p| unhex32(p)).unwrap_or([1; 32]);
                    let q = flags.get("q").and_then(|q| unhex32(q)).unwrap_or([2; 32]);
                    let to = flags.get("payout").and_then(|t| unhex32(t)).unwrap_or_else(|| die("--payout <64 hex> required".into()));
                    w.payout(&genesis.chain_id, nonce, p, q, to, fee)
                }
                _ => {
                    let to = flags.get("to").and_then(|t| unhex32(t)).unwrap_or([7; 32]);
                    w.transfer(&genesis.chain_id, nonce, token, amount, to, fee)
                }
            }
            .unwrap_or_else(|e| die(format!("sign: {e}")));
            println!("{}", hex(&signed.bytes));
            eprintln!("txid {}", signed.txid_hex());
        }
        _ => {
            eprintln!("usage: soft3 chain genesis-dev [--name dev] [--time T] [--bits HEX] [--fund NEURON=AMOUNT]… --out FILE");
            eprintln!("       soft3 chain key --seed HEX32");
            eprintln!("       soft3 chain tx --seed HEX32 --genesis FILE --nonce N [--kind transfer|link|unlock|payout] [--to N] [--amount A] [--fee F] [--p P --q Q] [--valence V] [--payout OWNER]");
            std::process::exit(2);
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
    println!("  soft3 mine --address NEURON [--chain pussy|bostrom] [--node URL]");
    println!("             [--backend auto|cpu|metal|webgpu] [--threads N]");
    println!("  soft3 mine bench [--backend ...] [--threads N] [--seconds S]");
    println!("  soft3 node --chain <pussy|bostrom|dev> --home DIR --bind HOST:PORT --radio HOST:UDP --peers FILE --moniker NAME");
    println!("  soft3 node id --home DIR");
    println!("  soft3 chain genesis-dev|key|tx …   # dev helpers");
    println!("  soft3 genesis build --chain pussy|bostrom|pussy-rc --snapshot DIR --out FILE");
    println!("                      [--cache DIR] [--export DIR] [--no-root] [--time UNIX]");
    println!("  soft3 genesis verify FILE [--root]");
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
    fn mine_args_parse() {
        let a = parse_mine_args(&[]).ok().unwrap();
        assert_eq!(a.chain, "pussy");
        assert_eq!(a.backend, "auto");
        assert!(!a.bench);
        let args: Vec<String> = [
            "bench",
            "--backend",
            "cpu",
            "--threads",
            "3",
            "--seconds",
            "2",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let a = parse_mine_args(&args).ok().unwrap();
        assert!(a.bench && a.backend == "cpu" && a.threads == 3 && a.seconds == 2);
        let args: Vec<String> = ["--address"].iter().map(|s| s.to_string()).collect();
        assert!(matches!(
            parse_mine_args(&args),
            Err(NodeArgError::MissingValue(_))
        ));
        let args: Vec<String> = ["--bogus"].iter().map(|s| s.to_string()).collect();
        assert!(matches!(
            parse_mine_args(&args),
            Err(NodeArgError::Unknown(_))
        ));
        assert_eq!(
            default_node("bostrom").as_deref(),
            Some("http://127.0.0.1:7782")
        );
        assert_eq!(default_node("x"), None);
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

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_flag_leaves_all_args_as_rest() {
        let args = v(&["node", "--home", "/tmp/x"]);
        let (names, rest) = extract_network_flags(&args);
        assert_eq!(names, Vec::<String>::new());
        assert_eq!(rest, args);
    }

    #[test]
    fn long_flag_is_extracted_from_the_middle() {
        let args = v(&["sync", "--network", "spacepussy-test", "--verbose"]);
        let (names, rest) = extract_network_flags(&args);
        assert_eq!(names, v(&["spacepussy-test"]));
        assert_eq!(rest, v(&["sync", "--verbose"]));
    }

    #[test]
    fn short_flag_form_is_equivalent() {
        let args = v(&["sync", "-n", "pussy"]);
        let (names, rest) = extract_network_flags(&args);
        assert_eq!(names, v(&["pussy"]));
        assert_eq!(rest, v(&["sync"]));
    }

    #[test]
    fn flag_with_no_following_argument_yields_an_empty_name() {
        let args = v(&["sync", "--network"]);
        let (names, rest) = extract_network_flags(&args);
        assert_eq!(names, v(&[""]));
        assert_eq!(rest, v(&["sync"]));
    }

    #[test]
    fn repeated_flags_are_all_returned_in_argv_order() {
        let args = v(&["--network", "bostrom", "sync", "-n", "pussy"]);
        let (names, rest) = extract_network_flags(&args);
        assert_eq!(names, v(&["bostrom", "pussy"]));
        assert_eq!(rest, v(&["sync"]));
    }
}

/// `soft3 conformance bless|check chain [--genesis FILE]` — the blessed
/// vectors of chain-conformance.md in `conformance/chain.snap`
/// (`chain::conformance`). `bless` writes, `check` compares; both take the
/// pinned genesis file for the §10 entries (default
/// `~/.cache/cyber/genesis/pussy.tade`, kept as committed when absent).
fn cmd_conformance(args: &[String]) {
    use soft3::chain::conformance;
    let sub = args.first().map(|s| s.as_str()).unwrap_or("");
    let target = args.get(1).map(|s| s.as_str()).unwrap_or("");
    if !matches!(sub, "bless" | "check") || target != "chain" {
        eprintln!("usage: soft3 conformance bless|check chain [--genesis FILE] [--snap FILE]");
        std::process::exit(2);
    }
    let value = |flag: &str| -> Option<String> { args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned() };
    let snap = value("--snap").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance/chain.snap")));
    let genesis = value("--genesis").map(PathBuf::from).unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".cache/cyber/genesis/pussy.tade")
    });
    let committed = std::fs::read_to_string(&snap).map(|s| conformance::parse(&s)).unwrap_or_default();
    let mut current = conformance::entries();
    match std::fs::read(&genesis) {
        Ok(bytes) => match conformance::genesis_entries("pussy", &bytes) {
            Ok(e) => current.extend(e),
            Err(e) => {
                eprintln!("conformance: {} is not a genesis: {e}", genesis.display());
                std::process::exit(1);
            }
        },
        Err(_) => eprintln!("conformance: no genesis file at {} — §10 entries kept as committed", genesis.display()),
    }
    let drift = conformance::drift(&committed, &current);
    for (label, was, now) in &drift {
        let was = was.map(|w| soft3::wallet::hex(&w)).unwrap_or_else(|| "(new)".into());
        println!("  {label}: {was} -> {}", soft3::wallet::hex(now));
    }
    match sub {
        "check" => {
            if drift.is_empty() {
                println!("chain.snap: {} entries, no drift", current.len());
            } else {
                eprintln!("chain.snap: {} entries moved; re-bless and review", drift.len());
                std::process::exit(1);
            }
        }
        _ => {
            let merged = conformance::merged(&committed, &current);
            let date = conformance::utc_date();
            if let Err(e) = std::fs::write(&snap, conformance::render(&merged, &date)) {
                eprintln!("conformance: cannot write {}: {e}", snap.display());
                std::process::exit(1);
            }
            println!("blessed {} entries into {} ({} moved)", merged.len(), snap.display(), drift.len());
        }
    }
}

/// `soft3 genesis build|verify` — burial snapshot → genesis file (tade
/// stream, `specs/chain.md` §4) → block-0 state root; `specs/genesis.md`.
fn cmd_genesis(args: &[String]) {
    use soft3::genesis::{self, Chain, Options};
    let sub = args.first().map(|s| s.as_str()).unwrap_or("");
    let value = |flag: &str| -> Option<String> {
        args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
    };
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let print_root = |root: &Option<String>, block0: &Option<String>, blocked: &Option<std::collections::BTreeMap<String, String>>| {
        match (root, blocked) {
            (Some(r), _) => {
                println!("  state_root {r}");
                println!("  block0_id  {}", block0.as_deref().unwrap_or("-"));
            }
            (None, Some(blocked)) => {
                println!("  state_root BLOCKED — dimensions over the lens commitment limit:");
                for (d, why) in blocked {
                    println!("    {d:<12} {why}");
                }
            }
            (None, None) => println!("  state_root skipped (--no-root)"),
        }
    };
    match sub {
        "build" => {
            let Some(chain) = value("--chain").and_then(|c| Chain::parse(&c)) else {
                eprintln!("genesis build: --chain pussy|bostrom|pussy-rc is required");
                std::process::exit(2);
            };
            let Some(snapshot) = value("--snapshot") else {
                eprintln!("genesis build: --snapshot DIR (holding manifest.json) is required");
                std::process::exit(2);
            };
            let Some(out) = value("--out") else {
                eprintln!("genesis build: --out FILE (the genesis .tade) is required");
                std::process::exit(2);
            };
            let time = match value("--time") {
                Some(t) => match t.parse::<u64>() {
                    Ok(t) => Some(t),
                    Err(_) => {
                        eprintln!("genesis build: --time must be unix seconds");
                        std::process::exit(2);
                    }
                },
                None => None,
            };
            let options = Options {
                cache: value("--cache").map(PathBuf::from).unwrap_or_else(|| Options::default_cache(&chain)),
                root: !has("--no-root"),
                export: value("--export").map(PathBuf::from),
                time,
            };
            let started = std::time::Instant::now();
            match genesis::build(chain, &PathBuf::from(snapshot), &PathBuf::from(&out), &options) {
                Ok(r) => {
                    println!("genesis {} · particle {}", r.chain, r.particle);
                    println!("  file       {out} ({} bytes, sha256 {})", r.file_bytes, r.file_sha256);
                    println!("  time       {}", r.time);
                    println!("  S0         {}  subsidy {}", r.s0, r.subsidy);
                    println!("  pool       format {} depth {} anchor_window {} digest {} tags {}", r.pool.format, r.pool.depth, r.pool.anchor_window, r.pool.digest, r.pool.tags);
                    print_root(&r.state_root, &r.block0_id, &r.root_blocked);
                    for (k, v) in &r.counts {
                        println!("  {k:<34} {v}");
                    }
                    for (denom, (token, sum)) in &r.tokens {
                        println!("  token:{denom:<28} {sum}  {token}");
                    }
                    if let Some(c) = &r.state_counts {
                        for (k, v) in c {
                            println!("  bbg:{k:<30} {v}");
                        }
                    }
                    if let Some(f) = &r.state_fields {
                        for (k, v) in f {
                            println!("  fields:{k:<27} {v}");
                        }
                    }
                    for (k, v) in &r.gaps {
                        println!("  gap:{k} {v}");
                    }
                    println!("  elapsed {} s", started.elapsed().as_secs());
                }
                Err(e) => {
                    eprintln!("genesis build failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "verify" => {
            let Some(path) = args.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("genesis verify: FILE is required");
                std::process::exit(2);
            };
            match genesis::verify(&PathBuf::from(path), has("--root")) {
                Ok(v) => {
                    println!("genesis particle {}", v.particle);
                    println!("  S0 {}  subsidy {}  b {}  l {}", v.s0, v.subsidy, v.b_frames, v.l_frames);
                    println!(
                        "  pool format {} depth {} anchor_window {} digest {} tags {}",
                        v.pool.format, v.pool.depth, v.pool.anchor_window, v.pool.digest, v.pool.tags
                    );
                    println!("  pool empty_root {}", v.pool.empty_root);
                    if has("--root") {
                        print_root(&v.state_root, &v.block0_id, &v.root_blocked);
                    }
                }
                Err(e) => {
                    eprintln!("genesis verify failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage: soft3 genesis build --chain pussy|bostrom|pussy-rc --snapshot DIR --out FILE [--cache DIR] [--export DIR] [--no-root] [--time UNIX]");
            eprintln!("       soft3 genesis verify FILE [--root]");
            std::process::exit(2);
        }
    }
}
