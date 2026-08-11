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
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn manifesto_nonempty() {
        assert!(!super::manifesto().is_empty());
    }
}
