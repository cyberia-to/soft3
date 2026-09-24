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
    pub retired: bool,
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
    let source = cybergraph::legacy_file::LegacyFile::load(&path, "soft3/native-genesis/1", MAX_GENESIS)?
        .ok_or_else(|| io::Error::other("missing native genesis"))?;
    if source.is_retired() && create {
        return Err(io::Error::other("retired genesis requires its original database"));
    }
    let value: Genesis = serde_json::from_slice(source.bytes())?;
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
        retired: source.is_retired(),
    })
}

/// Original JSON bytes remain under genesis.json/source. Old flat-file executables
/// fail their mandatory genesis read instead of silently starting an empty graph.
pub(super) fn retire(home: &Path) -> io::Result<()> {
    let mut source = cybergraph::legacy_file::LegacyFile::load(
        &home.join("genesis.json"), "soft3/native-genesis/1", MAX_GENESIS,
    )?.ok_or_else(|| io::Error::other("missing native genesis"))?;
    source.retire()
}
