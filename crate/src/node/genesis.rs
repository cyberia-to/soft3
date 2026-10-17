use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub(super) const MAX_LEGACY_LOG: usize = 64 * 1024 * 1024;
const MAX_GENESIS: usize = 16 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Genesis {
    chain_id: String,
    genesis_time: u64,
    engine: String,
    protocol: String,
}

pub(super) struct Loaded {
    pub canonical: Vec<u8>,
    pub time: u64,
}

pub(super) fn read_bounded(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected a regular file",
        ));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} exceeds {limit} bytes", path.display()),
        ));
    }
    Ok(bytes)
}

pub(super) fn load(home: &Path, create: bool) -> io::Result<Loaded> {
    let path = home.join("genesis.json");
    if create && !path.try_exists()? {
        let value = Genesis {
            chain_id: "spacepussy-test".into(),
            genesis_time: super::now_unix(),
            engine: "cybergraph+bbg".into(),
            protocol: "soft3/spacepussy-test/v1".into(),
        };
        let mut bytes = serde_json::to_vec_pretty(&value)?;
        bytes.push(b'\n');
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        File::open(home)?.sync_all()?;
    }
    let value: Genesis = serde_json::from_slice(&read_bounded(&path, MAX_GENESIS)?)?;
    if value.chain_id != "spacepussy-test"
        || value.engine != "cybergraph+bbg"
        || !matches!(
            value.protocol.as_str(),
            "soft3/spacepussy-test/v1" | "soft3/spacepussy-test/v2"
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported genesis chain, engine or protocol",
        ));
    }
    Ok(Loaded {
        canonical: serde_json::to_vec(&value)?,
        time: value.genesis_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_err(home: &Path) -> io::Error {
        match load(home, false) {
            Err(e) => e,
            Ok(_) => panic!("expected load() to fail"),
        }
    }

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "soft3-genesis-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_creates_default_genesis_on_fresh_home() {
        let home = tempdir();
        let loaded = load(&home, true).expect("default genesis should load");
        assert!(home.join("genesis.json").try_exists().unwrap());
        let value: Genesis = serde_json::from_slice(&loaded.canonical).unwrap();
        assert_eq!(value.chain_id, "spacepussy-test");
        assert_eq!(value.engine, "cybergraph+bbg");
        assert_eq!(value.protocol, "soft3/spacepussy-test/v1");
        assert_eq!(loaded.time, value.genesis_time);
    }

    #[test]
    fn load_without_create_fails_on_missing_genesis() {
        let home = tempdir();
        let err = load_err(&home);
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn load_rejects_wrong_chain_id() {
        let home = tempdir();
        let genesis = serde_json::json!({
            "chain_id": "not-spacepussy",
            "genesis_time": 0,
            "engine": "cybergraph+bbg",
            "protocol": "soft3/spacepussy-test/v1",
        });
        std::fs::write(
            home.join("genesis.json"),
            serde_json::to_vec(&genesis).unwrap(),
        )
        .unwrap();
        let err = load_err(&home);
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("unsupported"));
    }

    #[test]
    fn load_rejects_wrong_engine() {
        let home = tempdir();
        let genesis = serde_json::json!({
            "chain_id": "spacepussy-test",
            "genesis_time": 0,
            "engine": "not-the-engine",
            "protocol": "soft3/spacepussy-test/v1",
        });
        std::fs::write(
            home.join("genesis.json"),
            serde_json::to_vec(&genesis).unwrap(),
        )
        .unwrap();
        let err = load_err(&home);
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn load_accepts_v2_protocol() {
        let home = tempdir();
        let genesis = serde_json::json!({
            "chain_id": "spacepussy-test",
            "genesis_time": 42,
            "engine": "cybergraph+bbg",
            "protocol": "soft3/spacepussy-test/v2",
        });
        std::fs::write(
            home.join("genesis.json"),
            serde_json::to_vec(&genesis).unwrap(),
        )
        .unwrap();
        let loaded = load(&home, false).expect("v2 protocol should be accepted");
        assert_eq!(loaded.time, 42);
    }

    #[test]
    fn load_rejects_unsupported_protocol() {
        let home = tempdir();
        let genesis = serde_json::json!({
            "chain_id": "spacepussy-test",
            "genesis_time": 0,
            "engine": "cybergraph+bbg",
            "protocol": "soft3/spacepussy-test/v99",
        });
        std::fs::write(
            home.join("genesis.json"),
            serde_json::to_vec(&genesis).unwrap(),
        )
        .unwrap();
        let err = load_err(&home);
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn load_rejects_unknown_fields() {
        let home = tempdir();
        let genesis = serde_json::json!({
            "chain_id": "spacepussy-test",
            "genesis_time": 0,
            "engine": "cybergraph+bbg",
            "protocol": "soft3/spacepussy-test/v1",
            "extra_field": "not allowed",
        });
        std::fs::write(
            home.join("genesis.json"),
            serde_json::to_vec(&genesis).unwrap(),
        )
        .unwrap();
        let err = load_err(&home);
        assert!(err.to_string().contains("unknown field") || err.kind() == io::ErrorKind::InvalidData);
    }

    #[test]
    fn read_bounded_rejects_oversized_file() {
        let home = tempdir();
        let path = home.join("big.json");
        std::fs::write(&path, vec![b'a'; 100]).unwrap();
        let err = read_bounded(&path, 50).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("exceeds"));
    }

    #[test]
    fn read_bounded_accepts_file_at_exact_limit() {
        let home = tempdir();
        let path = home.join("exact.json");
        std::fs::write(&path, vec![b'a'; 50]).unwrap();
        let bytes = read_bounded(&path, 50).expect("exact-limit file should be accepted");
        assert_eq!(bytes.len(), 50);
    }

    #[test]
    fn read_bounded_rejects_directory() {
        let home = tempdir();
        let err = read_bounded(&home, MAX_GENESIS).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("regular file"));
    }
}
