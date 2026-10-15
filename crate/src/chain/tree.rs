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
        assert_eq!(hex(&l), "c2cb6adb74ed69c5672c495ccfb659e12b8f33db3be8caf6167a0d50b41893e6");
        assert_eq!(tx_root(std::slice::from_ref(&cb)), l);
        assert_eq!(hex(&node(&l, &l)), "3685e0a5b8bce069a4ef54da15f56393cd5e64dc6d10701d247a4663b7d922c1");
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
