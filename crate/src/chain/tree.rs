//! Transaction root: a domain-separated hemera Merkle tree, odd nodes
//! promoted unchanged (§1.7).

use super::header::hash32;
use super::params::{DOMAIN_LEAF, DOMAIN_NODE};

pub fn leaf(tx_bytes: &[u8]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(DOMAIN_LEAF.len() + tx_bytes.len());
    buf.extend_from_slice(DOMAIN_LEAF);
    buf.extend_from_slice(tx_bytes);
    hash32(&buf)
}

pub fn node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(DOMAIN_NODE.len() + 64);
    buf.extend_from_slice(DOMAIN_NODE);
    buf.extend_from_slice(left);
    buf.extend_from_slice(right);
    hash32(&buf)
}

/// `tx_root` over transactions in body order; `txs` must be non-empty
/// (a block always carries its coinbase).
pub fn tx_root(txs: &[Vec<u8>]) -> [u8; 32] {
    assert!(!txs.is_empty(), "a block has at least its coinbase");
    let mut level: Vec<[u8; 32]> = txs.iter().map(|t| leaf(t)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            if pair.len() == 2 {
                next.push(node(&pair[0], &pair[1]));
            } else {
                next.push(pair[0]);
            }
        }
        level = next;
    }
    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn coinbase_vector() -> Vec<u8> {
        let mut tx = vec![1u8, 0];
        tx.extend_from_slice(&1u64.to_le_bytes());
        tx.extend_from_slice(&[0x55; 32]);
        tx.extend_from_slice(&10_000u64.to_le_bytes());
        tx.extend_from_slice(&[0; 32]);
        tx
    }

    #[test]
    fn conformance_section_6_leaf_and_node() {
        let cb = coinbase_vector();
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
