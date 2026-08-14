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
            if s.latest_height > 0 {
                println!("  latest_height    {}", s.latest_height);
            }
            if s.earliest_height > 0 {
                println!("  earliest_height  {}", s.earliest_height);
            }
            println!(
                "  catching_up      {}",
                if s.catching_up { "yes" } else { "no" }
            );
            if let Some(code) = s.http_status {
                println!("  http             {code}");
            }
        }
        Err(e) => {
            println!("  reachable        no");
            println!("  detail           {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_node(args: &[String]) {
    let mut home = node::default_home();
    let mut bind = Network::DEFAULT.local_bind().to_string();
    let mut moniker = hostname_fallback();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--home" => {
                i += 1;
                home = PathBuf::from(args.get(i).expect("--home needs a path"));
            }
            "--bind" => {
                i += 1;
                bind = args.get(i).expect("--bind needs host:port").clone();
            }
            "--moniker" => {
                i += 1;
                moniker = args.get(i).expect("--moniker needs a name").clone();
            }
            "-h" | "--help" => {
                println!("soft3 node — run the spacepussy-test chaosnet surface");
                println!();
                println!("  soft3 node [--home DIR] [--bind HOST:PORT] [--moniker NAME]");
                println!();
                println!("defaults:");
                println!("  --home    ~/.spacepussy-test");
                println!("  --bind    {}", Network::DEFAULT.local_bind());
                println!("  --moniker <hostname>");
                return;
            }
            other => {
                eprintln!("unknown node flag `{other}`");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    if let Err(e) = node::run(home, &bind, &moniker) {
        eprintln!("soft3 node failed: {e}");
        std::process::exit(1);
    }
}

fn hostname_fallback() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
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
    println!("  soft3 manifesto");
    println!("  soft3 version");
    println!();
    println!("not soft3 networks (cosmos bootloader on cybernode):");
    println!("  space-pussy · bostrom");
    println!();
    println!("docs  https://cyber.page/soft3/docs/launch");
    println!("site  https://soft3.org");
}
