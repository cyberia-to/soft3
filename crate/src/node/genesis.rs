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

#[derive(Debug)]
pub(super) struct Loaded {
    pub canonical: Vec<u8>,
    pub time: u64,
    pub chain_id: String,
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
    let value = validate(&read_bounded(&path, MAX_GENESIS)?)?;
    Ok(Loaded {
        canonical: serde_json::to_vec(&value)?,
        time: value.genesis_time,
        chain_id: value.chain_id,
    })
}

fn validate(bytes: &[u8]) -> io::Result<Genesis> {
    let value: Genesis = serde_json::from_slice(bytes)?;
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
    Ok(value)
}

/// Adopt an external genesis file as `$home/genesis.json` before the
/// database exists. The caller (`node::install_genesis`) enforces that the
/// home has no `bbg` database yet; this function only guards the genesis
/// file itself, so a home that lost its database but kept its genesis is
/// not silently overwritten.
pub(super) fn install(home: &Path, file: &Path) -> io::Result<Loaded> {
    let target = home.join("genesis.json");
    if target.try_exists()? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} already has a genesis file", home.display()),
        ));
    }
    let bytes = read_bounded(file, MAX_GENESIS)?;
    let value = validate(&bytes)?;
    std::fs::create_dir_all(home)?;
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)?;
    out.write_all(&bytes)?;
    out.sync_all()?;
    File::open(home)?.sync_all()?;
    Ok(Loaded {
        canonical: serde_json::to_vec(&value)?,
        time: value.genesis_time,
        chain_id: value.chain_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> std::path::PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "soft3-genesis-install-test-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            unique,
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_genesis(path: &Path, chain_id: &str, protocol: &str) {
        let genesis = serde_json::json!({
            "chain_id": chain_id,
            "genesis_time": 7,
            "engine": "cybergraph+bbg",
            "protocol": protocol,
        });
        std::fs::write(path, serde_json::to_vec(&genesis).unwrap()).unwrap();
    }

    #[test]
    fn install_adopts_a_valid_external_genesis() {
        let source_dir = tempdir();
        let source = source_dir.join("bostrom-genesis.json");
        write_genesis(&source, "spacepussy-test", "soft3/spacepussy-test/v1");
        let home = tempdir();
        std::fs::remove_dir(&home).unwrap();

        let loaded = install(&home, &source).expect("valid genesis should install");

        assert_eq!(loaded.chain_id, "spacepussy-test");
        assert_eq!(loaded.time, 7);
        let installed = std::fs::read(home.join("genesis.json")).unwrap();
        let original = std::fs::read(&source).unwrap();
        assert_eq!(installed, original, "installed bytes must match the source file exactly");
    }

    #[test]
    fn install_refuses_to_overwrite_an_existing_genesis() {
        let source_dir = tempdir();
        let source = source_dir.join("genesis.json");
        write_genesis(&source, "spacepussy-test", "soft3/spacepussy-test/v1");
        let home = tempdir();
        load(&home, true).expect("default genesis should load");

        let err = install(&home, &source).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn install_rejects_an_invalid_genesis_without_touching_home() {
        let source_dir = tempdir();
        let source = source_dir.join("genesis.json");
        write_genesis(&source, "not-spacepussy", "soft3/spacepussy-test/v1");
        let home = tempdir();
        std::fs::remove_dir(&home).unwrap();

        let err = install(&home, &source).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(!home.join("genesis.json").try_exists().unwrap());
    }

    #[test]
    fn install_rejects_a_missing_source_file() {
        let home = tempdir();
        std::fs::remove_dir(&home).unwrap();

        let err = install(&home, &home.join("does-not-exist.json")).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
