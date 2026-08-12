//! soft3 — stack facade and CLI product crate.
//!
//! Follow-up releases re-export published train crates (cyb, foculus, …).

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Stack identity.
pub fn identity() -> &'static str {
    "soft3"
}

/// Manifest lines for the landing story.
pub fn manifesto() -> &'static [&'static str] {
    &[
        "one mind — shared, provable, self-improving",
        "many languages — write, compute, mean",
        "open world — no schema, no gatekeeper",
        "light client validates all of history in roughly ~100 ns",
        "interplanetary consensus — no vote; sync in cyberspace",
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn manifesto_nonempty() {
        assert!(!super::manifesto().is_empty());
    }
}

// Re-exports of the published train (product facade).
pub use cyb;
pub use cybergraph;
pub use foculus;
