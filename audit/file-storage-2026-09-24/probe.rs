#[allow(dead_code)]
mod content;
use cyber_bao::{io::pre_order::PreOrderMemOutboard, tree::BlockSize};
use file::Particle;
fn main() {
    let root = std::path::PathBuf::from(std::env::var("CYBER_AUDIT_FIXTURE_ROOT").expect("synthetic root required"));
    assert!(!root.exists(), "fresh fixture required");
    for n in [0, 5, 4096, 4097, 65537] {
        let data: Vec<_> = (0..n).map(|i| (i % 251) as u8).collect();
        let raw = Particle::hash(&data);
        let bao = PreOrderMemOutboard::create(&data, BlockSize::from_chunk_log(4)).root;
        assert_ne!(raw.as_bytes(), bao.as_bytes());
        println!("identity len={n} particle={raw} radio_bao={bao} equal=false");
    }
    content::remember("ordinary fixture");
    let id = content::particle_of("ordinary fixture");
    assert_eq!(content::lookup(&id).as_deref(), Some("ordinary fixture"));
    println!("normal text roundtrip=true");
    let path = root.join("cyb/particles.jsonl");
    // Substitute bytes under the original declared hash, then invalidate via a legitimate append.
    let forged = serde_json::json!({"particle": format!("{}", Particle::from_bytes(id)), "text": "substituted fixture"});
    std::fs::write(&path, format!("{forged}\n")).unwrap();
    content::remember("cache invalidator");
    let loaded = content::lookup(&id).unwrap();
    assert_eq!(loaded, "substituted fixture");
    assert_ne!(content::particle_of(&loaded), id);
    println!("substituted bytes accepted under original particle=true");
    let malformed_hex = "é".repeat(32);
    let malformed = serde_json::json!({"particle": malformed_hex,"text":"boundary fixture"});
    std::fs::write(&path, format!("{malformed}\n")).unwrap();
    content::remember("invalidate malformed fixture");
    let panic = std::panic::catch_unwind(content::load).is_err();
    println!("unicode hex panic={panic}");
    // Use a string with a codepoint straddling one of the two-byte slices.
    let malformed_hex = format!("aé{}", "0".repeat(61));
    assert_eq!(malformed_hex.len(),64);
    std::fs::write(&path, format!("{}\n",serde_json::json!({"particle":malformed_hex,"text":"bad"}))).unwrap();
    content::remember("invalidate boundary fixture");
    assert!(std::panic::catch_unwind(content::load).is_err());
    println!("misaligned utf8 hash panics=true");
    // Replace the expected file with a directory; the API still returns ().
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    content::remember("cannot persist");
    println!("write-to-directory error reported=false");
    assert_eq!(content::json_field(r#"{"text":"a\tb"}"#, "text").as_deref(),Some("atb"));
    println!("standard JSON tab escape is misdecoded=true");
}
