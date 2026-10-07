//! Radio: the sovereign transport family, wired as a dependency.
//!
//! `feature = "radio"` is soft3's first dependency on the radio fork —
//! critical dependency #3 in the launch tracker named radio as "a
//! dependency of nothing in the phase-1 binaries." This slice depends
//! only on `iroh-gossip`'s `proto` module (no `net` feature, no quinn or
//! tokio), and derives the gossip topic a chain's signals travel on: the
//! one thing every node subscribing to a chain's settle gossip must
//! agree on before the transport itself is wired in.

use iroh_gossip::TopicId;

/// Deterministic gossip topic for a chain's settlement signals.
///
/// The same chain id always yields the same topic; distinct chain ids
/// never collide, since this hashes the id with the same Poseidon2
/// (hemera) construction the rest of the stack already trusts for
/// particle identity.
pub fn chain_topic_id(chain_id: &str) -> TopicId {
    let hash = hemera::hash(chain_id.as_bytes());
    TopicId::from_bytes(*hash.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_the_same_chain_id() {
        assert_eq!(
            chain_topic_id("spacepussy-test"),
            chain_topic_id("spacepussy-test")
        );
    }

    #[test]
    fn distinct_chains_get_distinct_topics() {
        assert_ne!(chain_topic_id("bostrom"), chain_topic_id("pussy"));
        assert_ne!(chain_topic_id("bostrom"), chain_topic_id("spacepussy-test"));
    }

    #[test]
    fn empty_chain_id_is_still_a_valid_topic() {
        // Not a real chain id, but the function must not panic on it.
        let _ = chain_topic_id("");
    }
}
