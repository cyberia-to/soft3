//! soft3 CLI — default network is **space-pussy**.

use soft3::network::{self, Network};

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return;
    }

    // global flags
    let mut net = Network::DEFAULT;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--network" || args[i] == "-n" {
            i += 1;
            let name = args.get(i).map(|s| s.as_str()).unwrap_or("");
            net = Network::parse(name).unwrap_or_else(|| {
                eprintln!("unknown network `{name}` (use space-pussy|bostrom)");
                std::process::exit(2);
            });
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
            let n = args.get(1).and_then(|s| Network::parse(s)).unwrap_or(net);
            print_network(n);
        }
        "status" | "sync" => {
            // `sync` is the product verb: bootstrap light view of the default network
            match network::probe(net) {
                Ok(s) => {
                    println!("soft3 sync · {}", s.network);
                    println!("  chain_id        {}", s.chain_id);
                    println!("  moniker         {}", s.moniker);
                    println!("  latest_height   {}", s.latest_height);
                    println!("  earliest_height {}", s.earliest_height);
                    println!(
                        "  catching_up     {}",
                        if s.catching_up { "yes" } else { "no" }
                    );
                    println!("  rpc             {}", s.network.rpc());
                    if s.chain_id != s.network.chain_id() {
                        println!("  warn: expected chain_id `{}`", s.network.chain_id());
                    }
                    println!();
                    println!(
                        "default after install: {} (override: --network bostrom)",
                        Network::DEFAULT
                    );
                }
                Err(e) => {
                    eprintln!("soft3 sync failed: {e}");
                    eprintln!("rpc: {}", net.rpc());
                    std::process::exit(1);
                }
            }
        }
        "help" | "-h" | "--help" => print_help(),
        other => {
            eprintln!("unknown command `{other}`");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_network(n: Network) {
    println!("network {}", n.chain_id());
    println!("  prefix   {}", n.bech32_prefix());
    println!("  denom    {}", n.denom());
    println!("  rpc      {}", n.rpc());
    println!("  lcd      {}", n.lcd());
    println!("  index    {}", n.index());
    println!("  ws       {}", n.websocket());
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
    println!();
    println!("usage:");
    println!("  soft3 sync [--network space-pussy|bostrom]");
    println!("  soft3 status              # alias of sync");
    println!("  soft3 network [name]      # print endpoints");
    println!("  soft3 manifesto");
    println!("  soft3 version");
    println!();
    println!("install:");
    println!("  cargo install soft3");
    println!("  cargo install cyb         # runtime lib + bin `cy`");
    println!();
    println!("docs  https://cyber.page/soft3/");
    println!("site  https://soft3.org");
}
