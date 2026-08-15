//! cyber-conformance — stability harness for the soft3 stack.
//!
//! See `specs/README.md` for the full specification and `docs/README.md`
//! for the design rationale.
//!
//! Scaffold: trait surface drafted, hemera dependency stubbed until the
//! hemera crate reaches stable output. Snapshot file I/O and the
//! `cargo conformance` subcommand land in subsequent crates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

/// 32-byte hemera fingerprint of canonical bytes.
///
/// This is the only thing a snapshot ever stores. Equality of two
/// [`Fingerprint`] values implies equality of the underlying canonical
/// encoding to within hemera's 256-bit collision resistance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fingerprint(pub [u8; 32]);

impl Fingerprint {
    /// Hex-encoded form used inside `.snap` files: `h{64 hex chars}`.
    pub fn to_snap_string(self) -> String {
        let mut s = String::with_capacity(65);
        s.push('h');
        for b in self.0 {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Tier
// ---------------------------------------------------------------------------

/// Stability tier — determines the ceremony required to move a snapshot.
///
/// Promotion is one-way. Demotion requires the same ceremony as breaking
/// a snapshot at the target tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// freely mutable; snapshot regenerates silently
    Alpha,
    /// tracked; drift logs to CI but does not fail
    Beta,
    /// enforced; drift fails CI; bless requires explicit flag
    Gamma,
    /// locked; bless requires `[conformance:delta]` commit tag
    Delta,
    /// governed; bless requires detached signature appended to the .snap file
    Epsilon,
}

impl Tier {
    /// Whether drift at this tier should fail CI.
    pub const fn enforces(self) -> bool {
        matches!(self, Self::Gamma | Self::Delta | Self::Epsilon)
    }

    /// Whether mutation at this tier requires governance signature.
    pub const fn governed(self) -> bool {
        matches!(self, Self::Epsilon)
    }
}

// ---------------------------------------------------------------------------
// Conformant
// ---------------------------------------------------------------------------

/// A type whose canonical encoding has a stable fingerprint.
///
/// Implementing this trait is a commitment: the canonical encoding of any
/// representative instance hashes to the same fingerprint across versions
/// until a tier-appropriate bless ceremony occurs.
pub trait Conformant {
    /// Stability tier of this type's encoding.
    const TIER: Tier;

    /// Stable identifier — `crate::path::Type@vN`.
    ///
    /// The `@vN` suffix permits parallel encodings during migration:
    /// `Share@v1` and `Share@v2` can coexist in the snapshot file
    /// while readers transition.
    const NAME: &'static str;

    /// Canonical encoding of `self`.
    ///
    /// Two values that are equal under domain equality must produce
    /// byte-identical encodings. The encoding must not depend on
    /// iteration order, system time, or any other source of nondeterminism.
    fn canonical_encoding(&self) -> Vec<u8>;

    /// Fingerprint = hemera over canonical_encoding.
    ///
    /// Default implementation is the one true definition; do not override.
    fn fingerprint(&self) -> Fingerprint {
        Fingerprint(hemera_hash(&self.canonical_encoding()))
    }

    /// One representative instance whose fingerprint anchors the snapshot.
    fn snapshot_instance() -> Self
    where
        Self: Sized;
}

// ---------------------------------------------------------------------------
// Snapshot records
// ---------------------------------------------------------------------------

/// One line in `conformance/encoding.snap`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingSnapshot {
    /// stable identifier — `crate::path::Type@vN`
    pub name: String,
    /// tier as of the last bless
    pub tier: Tier,
    /// hemera fingerprint of canonical_encoding(snapshot_instance())
    pub fingerprint: Fingerprint,
}

/// One line in `conformance/mechanism.snap`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanismSnapshot {
    /// stable identifier — `crate::path::mechanism@vN`
    pub mechanism: String,
    /// scenario lookup key — names a fixed input for the mechanism
    pub scenario: String,
    /// tier as of the last bless
    pub tier: Tier,
    /// hemera fingerprint of the nox trace output for this scenario
    pub fingerprint: Fingerprint,
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Union of every conformance snapshot across a workspace.
///
/// The manifest's own fingerprint is the protocol stability root for a
/// given git revision. A [[zheng]] proof can be produced over this root,
/// attesting that the protocol revision conforms to manifest M without a
/// verifier re-running the harness.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    /// every encoding snapshot in the workspace
    pub encodings: Vec<EncodingSnapshot>,
    /// every mechanism snapshot in the workspace
    pub mechanisms: Vec<MechanismSnapshot>,
}

impl Manifest {
    /// hemera fingerprint of the canonical ordering of all snapshots.
    ///
    /// Canonical ordering: sort encodings by `name`, mechanisms by
    /// `(mechanism, scenario)`, concatenate canonical encodings, hash.
    pub fn root(&self) -> Fingerprint {
        let mut buf: Vec<u8> = Vec::new();
        let mut encs = self.encodings.clone();
        encs.sort_by(|a, b| a.name.cmp(&b.name));
        for e in &encs {
            buf.extend_from_slice(e.name.as_bytes());
            buf.push(0);
            buf.push(e.tier as u8);
            buf.extend_from_slice(&e.fingerprint.0);
        }
        let mut mechs = self.mechanisms.clone();
        mechs.sort_by(|a, b| {
            a.mechanism
                .cmp(&b.mechanism)
                .then_with(|| a.scenario.cmp(&b.scenario))
        });
        for m in &mechs {
            buf.extend_from_slice(m.mechanism.as_bytes());
            buf.push(0);
            buf.extend_from_slice(m.scenario.as_bytes());
            buf.push(0);
            buf.push(m.tier as u8);
            buf.extend_from_slice(&m.fingerprint.0);
        }
        Fingerprint(hemera_hash(&buf))
    }
}

// ---------------------------------------------------------------------------
// hemera bridge
// ---------------------------------------------------------------------------

/// Placeholder for the cyber-hemera dependency.
///
/// Replace with `cyber_hemera::hash(bytes).into()` once hemera output
/// reaches the stable release. The signature stays identical.
fn hemera_hash(_bytes: &[u8]) -> [u8; 32] {
    [0u8; 32]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct Example {
        n: u64,
    }

    impl Conformant for Example {
        const TIER: Tier = Tier::Alpha;
        const NAME: &'static str = "cyber_conformance::tests::Example@v1";

        fn canonical_encoding(&self) -> Vec<u8> {
            self.n.to_le_bytes().to_vec()
        }

        fn snapshot_instance() -> Self {
            Example { n: 42 }
        }
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let a = Example::snapshot_instance().fingerprint();
        let b = Example::snapshot_instance().fingerprint();
        assert_eq!(a, b);
    }

    #[test]
    fn tier_enforcement_policy() {
        assert!(!Tier::Alpha.enforces());
        assert!(!Tier::Beta.enforces());
        assert!(Tier::Gamma.enforces());
        assert!(Tier::Delta.enforces());
        assert!(Tier::Epsilon.enforces());
        assert!(Tier::Epsilon.governed());
        assert!(!Tier::Delta.governed());
    }

    #[test]
    fn manifest_root_is_deterministic() {
        let m = Manifest {
            encodings: vec![EncodingSnapshot {
                name: "x".into(),
                tier: Tier::Gamma,
                fingerprint: Fingerprint([1u8; 32]),
            }],
            mechanisms: vec![],
        };
        assert_eq!(m.root(), m.root());
    }
}
