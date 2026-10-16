//! Transaction root (§1.7): the wallet's Merkle functions, with the node's
//! invariant that a block always carries its coinbase.

pub use crate::wallet::tx::{leaf, node};

/// `tx_root` over transactions in body order; `txs` must be non-empty.
pub fn tx_root(txs: &[Vec<u8>]) -> [u8; 32] {
    crate::wallet::tx::tx_root(txs).expect("a block has at least its coinbase")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn conformance_section_6_leaf_and_node() {
        let cb = super::super::tx::tests::coinbase().encode();
        assert_eq!(cb.len(), 82);
        let l = leaf(&cb);
        assert_eq!(hex(&l), "514d5d57388046e2cd534c69a789bf09e329678cedbabcff4cc9a439e8c0fe9c");
        assert_eq!(tx_root(std::slice::from_ref(&cb)), l);
        assert_eq!(hex(&node(&l, &l)), "9d68ac9b03293ba25059ceb5d601e3c6ddae700e05343c190fdd4c8eea6c595b");
    }

    #[test]
    fn odd_nodes_promote_not_duplicate() {
        let a = vec![1u8; 10];
        let b = vec![2u8; 10];
        let c = vec![3u8; 10];
        let (la, lb, lc) = (leaf(&a), leaf(&b), leaf(&c));
        assert_eq!(tx_root(&[a.clone(), b.clone(), c.clone()]), node(&node(&la, &lb), &lc));
        assert_ne!(tx_root(&[a.clone(), b.clone(), c.clone()]), tx_root(&[a, b, c.clone(), c]));
    }
}
