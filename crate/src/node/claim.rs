//! Genesis claims: bind a legacy Cosmos key to a native neuron by verifying
//! a [`mudra::Claim`] (the ADR-036 bridge, row 15, closed) and recording the
//! binding at `$home/claims.jsonl`.

use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use mudra::Claim;
use serde::{Deserialize, Serialize};

/// A line of `$home/claims.jsonl`: one legacy address bound to one neuron.
#[derive(Serialize, Deserialize)]
struct ClaimRecord {
    address: String,
    pubkey: String,
    neuron: String,
}

/// Longest line this reads back; a claims file is operator-controlled, not
/// adversarial input, but a corrupt or truncated file should fail loudly
/// rather than allocate without bound.
const MAX_LINE: usize = 4096;

/// Verify `encoded` (a [`Claim::encode`]d line) under `hrp` and record the
/// binding in `$home/claims.jsonl`. Returns the bound neuron id, hex-encoded.
///
/// A legacy account binds exactly one neuron: replaying the same claim is a
/// no-op, but a second claim for an already-claimed address that names a
/// different neuron is rejected.
pub(super) fn claim_account(home: &Path, encoded: &str, hrp: &str) -> io::Result<String> {
    let claim = Claim::decode(encoded)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "malformed claim"))?;
    if !mudra::claim::verify(&claim, hrp) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "claim signature does not verify",
        ));
    }
    let neuron_hex = to_hex(&claim.neuron);
    let path = home.join("claims.jsonl");
    if let Some(existing) = find_existing(&path, &claim.address)? {
        if existing != neuron_hex {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} already claimed neuron {existing}, cannot rebind to {neuron_hex}",
                    claim.address
                ),
            ));
        }
        return Ok(neuron_hex);
    }
    std::fs::create_dir_all(home)?;
    let record = ClaimRecord {
        address: claim.address,
        pubkey: to_hex(&claim.pubkey),
        neuron: neuron_hex.clone(),
    };
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::to_string(&record)?)?;
    file.sync_all()?;
    Ok(neuron_hex)
}

fn find_existing(path: &Path, address: &str) -> io::Result<Option<String>> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.len() > MAX_LINE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "claims record too long"));
        }
        let record: ClaimRecord = serde_json::from_str(&line)?;
        if record.address == address {
            return Ok(Some(record.neuron));
        }
    }
    Ok(None)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mudra::SigningKey;

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes((&[seed; 32]).into()).unwrap()
    }

    fn tmp_home() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("soft3-claim-test-{}", rand_suffix()));
        dir
    }

    fn rand_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    #[test]
    fn valid_claim_binds_neuron() {
        let home = tmp_home();
        let k = key(1);
        let neuron = mudra::claim::neuron_of(&mudra::cosmos::compressed(k.verifying_key()));
        let claim = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron).unwrap();
        let got = claim_account(&home, &claim.encode(), mudra::cosmos::PUSSY).unwrap();
        assert_eq!(got, to_hex(&neuron));
        let contents = std::fs::read_to_string(home.join("claims.jsonl")).unwrap();
        assert_eq!(contents.lines().count(), 1);
    }

    #[test]
    fn malformed_claim_is_rejected() {
        let home = tmp_home();
        assert!(claim_account(&home, "not a claim", mudra::cosmos::PUSSY).is_err());
        assert!(!home.join("claims.jsonl").exists());
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let home = tmp_home();
        let k = key(2);
        let neuron = mudra::claim::neuron_of(&mudra::cosmos::compressed(k.verifying_key()));
        let mut claim = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron).unwrap();
        claim.neuron[0] ^= 0x01;
        assert!(claim_account(&home, &claim.encode(), mudra::cosmos::PUSSY).is_err());
        assert!(!home.join("claims.jsonl").exists());
    }

    #[test]
    fn replaying_the_same_claim_is_idempotent() {
        let home = tmp_home();
        let k = key(3);
        let neuron = mudra::claim::neuron_of(&mudra::cosmos::compressed(k.verifying_key()));
        let claim = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron).unwrap();
        claim_account(&home, &claim.encode(), mudra::cosmos::PUSSY).unwrap();
        claim_account(&home, &claim.encode(), mudra::cosmos::PUSSY).unwrap();
        let contents = std::fs::read_to_string(home.join("claims.jsonl")).unwrap();
        assert_eq!(contents.lines().count(), 1, "replay must not duplicate the record");
    }

    #[test]
    fn conflicting_neuron_for_the_same_address_is_rejected() {
        let home = tmp_home();
        let k = key(4);
        let neuron_a = mudra::claim::neuron_of(&mudra::cosmos::compressed(k.verifying_key()));
        let mut neuron_b = neuron_a;
        neuron_b[0] ^= 0x01;
        let claim_a = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron_a).unwrap();
        let claim_b = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron_b).unwrap();
        claim_account(&home, &claim_a.encode(), mudra::cosmos::PUSSY).unwrap();
        let err = claim_account(&home, &claim_b.encode(), mudra::cosmos::PUSSY).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let contents = std::fs::read_to_string(home.join("claims.jsonl")).unwrap();
        assert_eq!(contents.lines().count(), 1, "the rejected claim must not be recorded");
    }

    #[test]
    fn wrong_hrp_is_rejected() {
        let home = tmp_home();
        let k = key(5);
        let neuron = mudra::claim::neuron_of(&mudra::cosmos::compressed(k.verifying_key()));
        let claim = mudra::claim::create(&k, mudra::cosmos::PUSSY, neuron).unwrap();
        assert!(claim_account(&home, &claim.encode(), mudra::cosmos::BOSTROM).is_err());
        assert!(!home.join("claims.jsonl").exists());
    }
}
