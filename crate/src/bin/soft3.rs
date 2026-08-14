//! soft3 CLI — default network is **spacepussy-test** (soft3 chaosnet).

use soft3::network::{self, Network};

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
            if let Some(code) = s.http_status {
                println!("  http             {code}");
            }
            if !s.body_preview.is_empty() {
                println!("  body             {}", s.body_preview.replace('\n', " "));
            }
        }
        Err(e) => {
            println!("  reachable        no");
            println!("  detail           {e}");
            println!();
            println!("spacepussy-test is the soft3 chaosnet.");
            println!(
                "cosmos `space-pussy` / `bostrom` on cybernode are bootloader chains — different networks."
            );
            std::process::exit(1);
        }
    }
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
    println!("  soft3 manifesto");
    println!("  soft3 version");
    println!();
    println!("not soft3 networks (cosmos bootloader on cybernode):");
    println!("  space-pussy · bostrom     # migration sources, not product defaults");
    println!();
    println!("docs  https://cyber.page/soft3/");
    println!("site  https://soft3.org");
}
