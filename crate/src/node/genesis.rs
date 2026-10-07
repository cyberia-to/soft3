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

/// Chains the launch build accepts, each with its own protocol string set.
/// `spacepussy-test` keeps its v1/v2 grandfather; the three launch chains
/// (row 45 of the launch registry) take v1 only until a genesis subcommand
/// exists to load their real burial-snapshot data.
const ALLOWED_CHAINS: &[(&str, &[&str])] = &[
    (
        "spacepussy-test",
        &["soft3/spacepussy-test/v1", "soft3/spacepussy-test/v2"],
    ),
    ("pussy-rc", &["soft3/pussy-rc/v1"]),
    ("pussy", &["soft3/pussy/v1"]),
    ("bostrom", &["soft3/bostrom/v1"]),
];

fn protocol_allowed(chain_id: &str, protocol: &str) -> bool {
    ALLOWED_CHAINS
        .iter()
        .any(|(id, protocols)| *id == chain_id && protocols.contains(&protocol))
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
    if value.engine != "cybergraph+bbg" || !protocol_allowed(&value.chain_id, &value.protocol) {
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
    use std::fs;

    fn write_genesis(home: &Path, chain_id: &str, engine: &str, protocol: &str) {
        let value = Genesis {
            chain_id: chain_id.into(),
            genesis_time: 1,
            engine: engine.into(),
            protocol: protocol.into(),
        };
        fs::write(home.join("genesis.json"), serde_json::to_vec(&value).unwrap()).unwrap();
    }

    fn tmp_home(name: &str) -> std::path::PathBuf {
        let home = std::env::temp_dir().join(format!("soft3-genesis-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).unwrap();
        home
    }

    #[test]
    fn load_creates_default_spacepussy_test() {
        let home = tmp_home("default");
        let loaded = load(&home, true).unwrap();
        assert!(!loaded.canonical.is_empty());
    }

    #[test]
    fn load_accepts_pussy_rc_genesis() {
        let home = tmp_home("pussy-rc");
        write_genesis(&home, "pussy-rc", "cybergraph+bbg", "soft3/pussy-rc/v1");
        assert!(load(&home, false).is_ok());
    }

    #[test]
    fn load_accepts_bostrom_genesis() {
        let home = tmp_home("bostrom");
        write_genesis(&home, "bostrom", "cybergraph+bbg", "soft3/bostrom/v1");
        assert!(load(&home, false).is_ok());
    }

    #[test]
    fn load_accepts_pussy_genesis() {
        let home = tmp_home("pussy");
        write_genesis(&home, "pussy", "cybergraph+bbg", "soft3/pussy/v1");
        assert!(load(&home, false).is_ok());
    }

    #[test]
    fn load_rejects_unknown_chain() {
        let home = tmp_home("unknown-chain");
        write_genesis(&home, "totally-unknown", "cybergraph+bbg", "soft3/totally-unknown/v1");
        assert!(load(&home, false).is_err());
    }

    #[test]
    fn load_rejects_cross_chain_protocol() {
        let home = tmp_home("cross-chain");
        write_genesis(&home, "bostrom", "cybergraph+bbg", "soft3/pussy/v1");
        assert!(load(&home, false).is_err());
    }

    #[test]
    fn load_rejects_wrong_engine() {
        let home = tmp_home("wrong-engine");
        write_genesis(&home, "bostrom", "not-the-real-engine", "soft3/bostrom/v1");
        assert!(load(&home, false).is_err());
    }
}
