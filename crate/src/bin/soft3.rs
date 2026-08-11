fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("soft3 {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.iter().any(|a| a == "manifesto") {
        for line in soft3::manifesto() {
            println!("{line}");
        }
        return;
    }
    println!("soft3 {} — type 2 civilization software", env!("CARGO_PKG_VERSION"));
    println!();
    println!("  cargo install soft3");
    println!("  cargo install cyb          # runtime + binary `cy`");
    println!("  site   https://soft3.org");
    println!("  docs   https://cyber.page/soft3/");
    println!("  code   https://github.com/cyberia-to/soft3");
    println!();
    println!("Commands (growing): soft3 manifesto | soft3 --version");
}
