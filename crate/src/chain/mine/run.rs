//! The mining loop: template → grind → submit, with template refresh on a
//! new tip, hashrate reporting and retry.
//!
//! Nonce space: the high 64 bits of every nonce are a per-session salt
//! (so two miners on the same template never retrace each other's range),
//! the low 64 bits count up from 0 — which also keeps every dispatch on one
//! side of the `2^64` boundary the kernels require.

use super::api::{Client, Template, Verdict};
use super::target::{target_of, verifies};
use super::{Backend, Job};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// `soft3 mine` settings.
#[derive(Clone, Debug)]
pub struct Settings {
    pub chain: String,
    pub node: String,
    pub address: String,
    /// Seconds between `GET /tip` polls.
    pub tip_poll: Duration,
    /// Seconds after which a template is refreshed even if the tip stayed.
    pub refresh: Duration,
    /// Seconds between hashrate lines.
    pub report: Duration,
}

/// What one mined template ended with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A block was accepted by the node.
    Accepted {
        height: u64,
        id: String,
        nonce: u128,
    },
    /// The tip moved or the template aged: refresh.
    Refresh,
    /// The node rejected the solved header (a bug on one side; logged).
    Rejected(String),
}

/// Hashrate for display only (never consensus): `n` hashes over `dt`.
pub fn rate(n: u64, dt: Duration) -> f64 {
    if dt.is_zero() {
        0.0
    } else {
        n as f64 / dt.as_secs_f64()
    }
}

pub fn fmt_rate(hs: f64) -> String {
    if hs >= 1e9 {
        format!("{:.2} GH/s", hs / 1e9)
    } else if hs >= 1e6 {
        format!("{:.2} MH/s", hs / 1e6)
    } else if hs >= 1e3 {
        format!("{:.1} kH/s", hs / 1e3)
    } else {
        format!("{hs:.0} H/s")
    }
}

/// A session salt: the high 64 bits of every nonce this process mines.
pub fn salt() -> u64 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut m = Vec::new();
    m.extend_from_slice(&t.as_nanos().to_le_bytes());
    m.extend_from_slice(&std::process::id().to_le_bytes());
    let h = hemera::hash(&m);
    u64::from_le_bytes(h.as_bytes()[..8].try_into().expect("8 bytes"))
}

/// Mine one template on `backend` until a block is found, the tip moves
/// or `stop` is raised. `log` receives progress lines.
pub fn mine_template(
    client: &Client,
    backend: &dyn Backend,
    template: &Template,
    salt: u64,
    s: &Settings,
    stop: &AtomicBool,
    log: &dyn Fn(&str),
) -> Result<Outcome, String> {
    let header = template.header_bytes()?;
    let bits = template.bits_u32()?;
    let target = target_of(bits)?;
    if target != template.target_words()? {
        return Err("template target disagrees with its bits".into());
    }
    let job = Job::new(&header, target);
    let started = Instant::now();
    let mut last_tip = started;
    let mut last_report = started;
    let mut hashes = 0u64;
    let mut low = 0u64;
    let base_hi = (salt as u128) << 64;
    log(&format!(
        "template height {} prev {}… bits {bits:#010x} ({} expected hashes)",
        template.height,
        &template.prev[..12],
        expected(&target)
    ));
    while !stop.load(Relaxed) {
        let count = backend
            .dispatch()
            .min((u64::MAX - low).min(u32::MAX as u64) as u32);
        if count == 0 {
            return Ok(Outcome::Refresh);
        }
        let base = base_hi | low as u128;
        let found = backend.search(&job, base, count)?;
        hashes += count as u64;
        low += count as u64;
        if let Some(i) = found {
            let nonce = base + i as u128;
            let solved = job.header_with(nonce);
            if !verifies(&solved, &target) {
                return Err(format!(
                    "{}: nonce {nonce:#x} does not verify natively",
                    backend.name()
                ));
            }
            log(&format!(
                "found nonce {nonce:#034x} after {} hashes, {:.1}s",
                hashes,
                started.elapsed().as_secs_f64()
            ));
            let verdict = Client::retry(
                || client.submit(&solved),
                5,
                Duration::from_secs(8),
                |m| log(m),
            )?;
            return Ok(match verdict {
                Verdict::Accepted(a) => Outcome::Accepted {
                    height: a.height,
                    id: a.id,
                    nonce,
                },
                Verdict::Stale => Outcome::Refresh,
                Verdict::Rejected(r) => Outcome::Rejected(r),
            });
        }
        let now = Instant::now();
        if now - last_report >= s.report {
            log(&format!(
                "{} · {} · {} hashes · height {}",
                backend.name(),
                fmt_rate(rate(hashes, now - started)),
                hashes,
                template.height
            ));
            last_report = now;
        }
        if now - last_tip >= s.tip_poll {
            last_tip = now;
            match client.tip() {
                Ok(t) if t.id != template.prev => {
                    log(&format!("tip moved to height {} — refreshing", t.height));
                    return Ok(Outcome::Refresh);
                }
                Ok(_) => {}
                Err(e) => log(&format!("{e} (still mining)")),
            }
        }
        if now - started >= s.refresh {
            return Ok(Outcome::Refresh);
        }
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if template.expires != 0 && unix >= template.expires {
            return Ok(Outcome::Refresh);
        }
    }
    Ok(Outcome::Refresh)
}

/// Expected hashes a block at `target`, `2^256 / (target + 1)`: exact as
/// a power of two when the target is one (the launch case), else the
/// order of magnitude (a target in `[2^(b−1), 2^b)` needs `2^(256−b)` to
/// `2^(257−b)` hashes).
fn expected(target: &[u64; 4]) -> String {
    let bits = 256 - leading_zeros(target);
    let pow = target.iter().map(|w| w.count_ones()).sum::<u32>() == 1;
    if pow {
        format!("2^{}", 257 - bits)
    } else {
        format!("≈2^{}", 256 - bits)
    }
}

fn leading_zeros(t: &[u64; 4]) -> u32 {
    let mut z = 0;
    for w in t.iter().rev() {
        if *w == 0 {
            z += 64;
        } else {
            return z + w.leading_zeros();
        }
    }
    z
}

/// `soft3 mine`: the loop over templates. Returns only on an unrecoverable
/// error or when `stop` is raised.
pub fn run(
    backend: &dyn Backend,
    s: &Settings,
    stop: &AtomicBool,
    log: &dyn Fn(&str),
) -> Result<(), String> {
    let client = Client::new(&s.node, &s.address);
    let salt = salt();
    log(&format!(
        "soft3 mine · chain {} · node {} · {} · to {}",
        s.chain,
        s.node,
        backend.name(),
        s.address
    ));
    let mut blocks = 0u64;
    while !stop.load(Relaxed) {
        let template = Client::retry(|| client.template(), 0, Duration::from_secs(30), |m| log(m))?;
        if template.chain != s.chain {
            return Err(format!(
                "node serves chain `{}`, not `{}`",
                template.chain, s.chain
            ));
        }
        match mine_template(&client, backend, &template, salt, s, stop, log)? {
            Outcome::Accepted { height, id, nonce } => {
                blocks += 1;
                log(&format!(
                    "block {height} accepted: {id} (nonce {nonce:#x}); {blocks} this session"
                ));
            }
            Outcome::Refresh => {}
            Outcome::Rejected(r) => log(&format!("rejected: {r}")),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_format() {
        assert_eq!(fmt_rate(64_400.0), "64.4 kH/s");
        assert_eq!(fmt_rate(13_200_000.0), "13.20 MH/s");
        assert_eq!(rate(100, Duration::from_secs(2)), 50.0);
        assert_eq!(expected(&super::super::target::pow2(232)), "2^24");
    }
}
