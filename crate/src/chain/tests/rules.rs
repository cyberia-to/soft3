//! Transaction and block rules on the engine (§2, §3, chain-wire §4).

use super::*;
use crate::chain::api::{BlockVerdict, ChainError};
use crate::chain::apply::{BlockError, TxError};
use crate::chain::state::StateBackend;
use crate::chain::tx;

#[test]
fn extends_tip_pays_subsidy_and_keeps_conservation() {
    let net = Net::new(2);
    let mut c = net.open("extend");
    let miner = net.neurons[1];
    assert_eq!(c.tip().height, 0);
    let b1 = mine(&mut c, miner, 0, &[]);
    assert_eq!(c.tip().height, 1);
    assert_eq!(c.tip().id, b1.id());
    assert_eq!(c.balance(&miner, &net.native), FUNDS + net.subsidy());
    assert_eq!(c.supply(&net.native), net.genesis.s0() + net.subsidy());
    assert!(c.status().conserved);
    assert_eq!(c.chainwork(&b1.id()), Some(crate::chain::U256::from_u64(1)), "2^256/(2^255+1) = 1");
    assert_eq!(c.accept_block(&b1, NOW), Ok(BlockVerdict::Known));
    assert_eq!(c.headers(0, 10).len(), 2);
    assert_eq!(c.blocks(1, 10), vec![b1.clone()]);
    assert_eq!(c.locator(), vec![b1.id(), c.header_at(0).unwrap().id()]);
}

#[test]
fn transfer_moves_value_and_the_nonce_advances() {
    let net = Net::new(3);
    let mut c = net.open("transfer");
    let (a, b, miner) = (net.neurons[0], net.neurons[1], net.neurons[2]);
    let t = net.transfer(0, 0, b, 500, 224);
    mine(&mut c, miner, 0, &[t.clone()]);
    assert_eq!(c.balance(&a, &net.native), FUNDS - 500 - 224);
    assert_eq!(c.balance(&b, &net.native), FUNDS + 500);
    assert_eq!(c.balance(&miner, &net.native), FUNDS + net.subsidy() + 224);
    assert_eq!(c.nonce(&a), 1);
    assert!(c.status().conserved);
    // replay of the same tx: nonce 0 is spent
    let tpl = template(&mut c, miner, 0, &[t.clone()]);
    assert_eq!(tpl.txids.len(), 1, "stale nonce excluded from the template");
    assert_eq!(c.validate_tx(&t, &[]), Err(TxError::Nonce { expected: 1, actual: 0 }));
    // the next nonce works; nonce 3 is inside the window but needs 1 and 2 first
    let t1 = net.transfer(0, 1, b, 1, 224);
    assert!(c.validate_tx(&t1, &[]).is_ok());
    let t3 = net.transfer(0, 3, b, 1, 224);
    let t2 = net.transfer(0, 2, b, 1, 224);
    assert_eq!(c.validate_tx(&t3, &[]).unwrap_err(), TxError::Nonce { expected: 1, actual: 3 }, "inside the window, but 1 and 2 are not pending");
    let pending = [Tx::decode(&t1).unwrap(), Tx::decode(&t2).unwrap()];
    assert_eq!(c.validate_tx(&t3, &pending).unwrap().nonce, 3);
    let t17 = net.transfer(0, 17, b, 1, 224);
    assert_eq!(c.validate_tx(&t17, &[]), Err(TxError::Nonce { expected: 1, actual: 17 }));
}

#[test]
fn fee_floor_signature_and_balance_rules() {
    let net = Net::new(2);
    let mut c = net.open("rules");
    let (b, miner) = (net.neurons[1], net.neurons[1]);
    assert_eq!(c.validate_tx(&net.transfer(0, 0, b, 1, 223), &[]), Err(TxError::Fee { min: 224, actual: 223 }));
    assert!(matches!(c.validate_tx(&net.transfer(0, 0, b, FUNDS, 224), &[]), Err(TxError::Insufficient { .. })));
    let mut bad_sig = net.transfer(0, 0, b, 1, 224);
    let last = bad_sig.len() - 1;
    bad_sig[last] ^= 1;
    assert_eq!(c.validate_tx(&bad_sig, &[]), Err(TxError::Signature));
    let mut bad_version = net.transfer(0, 0, b, 1, 224);
    bad_version[0] = 2;
    assert!(matches!(c.validate_tx(&bad_version, &[]), Err(TxError::Decode(_))));
    let cb = Tx { body: Body::Coinbase { height: 1, to: b, amount: 1, extra: [0; 32] }, sig: vec![] }.encode();
    assert_eq!(c.validate_tx(&cb, &[]), Err(TxError::CoinbaseInBody));
    // a block carrying a bad tx is invalid and its header is rejected afterwards
    let mut tpl = template(&mut c, miner, 0, &[]);
    let cb = Tx { body: Body::Coinbase { height: 1, to: miner, amount: net.subsidy() + 223, extra: [0; 32] }, sig: vec![] }.encode();
    let raws = vec![cb, net.transfer(0, 0, b, 1, 223)];
    tpl.body = tx::join_body(&raws);
    tpl.header.tx_root = crate::chain::tree::tx_root(&raws);
    let block = block_of(&tpl);
    assert!(matches!(c.accept_block(&block, NOW), Err(ChainError::Block(BlockError::Tx { index: 1, error: TxError::Fee { .. } }))));
    assert_eq!(c.accept_header(&block.header, NOW), Err(ChainError::Rejected(block.id())));
    assert_eq!(c.tip().height, 0);
}

#[test]
fn cyberlink_rules_and_effects() {
    let net = Net::new(2);
    let mut c = net.open("link");
    let miner = net.neurons[1];
    let (p, q) = (token("boot"), token("pussy"));
    assert_eq!(c.validate_tx(&net.link(0, 0, p, p, 0, 1, 257), &[]), Err(TxError::SelfLink));
    assert_eq!(c.validate_tx(&net.link(0, 0, p, q, 7, 1, 257), &[]), Err(TxError::Box(7)));
    assert_eq!(c.validate_tx(&net.link(0, 0, p, q, 0, 1, 256), &[]), Err(TxError::Fee { min: 257, actual: 256 }));
    let l = net.link(0, 0, p, q, 0, 1, 257);
    assert_eq!(l.len(), 257);
    let before = c.state().axon_count();
    mine(&mut c, miner, 0, &[l]);
    assert_eq!(c.state().axon_count(), before + 1);
    assert_eq!(c.nonce(&net.neurons[0]), 1);
    assert_eq!(c.balance(&net.neurons[0], &net.native), FUNDS - 257);
    assert!(c.state().bbg.axon_edges.contains_key(&bbg::state::axon_id(&p, &q)));
    assert!(c.status().conserved);
}

#[test]
fn claim_moves_every_dormant_balance_to_the_key() {
    let net = Net::new(2);
    let mut c = net.open("claim");
    let miner = net.neurons[1];
    let k = key(net.dormant_key as u8);
    let subject = neuron_of(&pubkey(&k));
    let dormant = legacy_owner("pussy", &mudra::cosmos::account_id(&pubkey(&k)));
    assert_eq!(c.balance(&dormant, &net.native), 10_000);
    // wrong key for the account id
    let mut wrong = Tx { body: Body::Claim { account_id: [0x99; 20], nonce: 0, fee: 140 }, sig: vec![] };
    sigs::sign(&net.genesis.chain_id, &mut wrong, &k).unwrap();
    assert_eq!(c.validate_tx(&wrong.encode(), &[]), Err(TxError::ClaimKey));
    let claim = net.claim(net.dormant_key as u8, 0, 140);
    assert_eq!(claim.len(), 140);
    mine(&mut c, miner, 0, &[claim.clone()]);
    assert_eq!(c.balance(&dormant, &net.native), 0);
    assert_eq!(c.balance(&dormant, &token("other")), 0);
    assert_eq!(c.balance(&subject, &net.native), 10_000 - 140);
    assert_eq!(c.balance(&subject, &token("other")), 5);
    assert_eq!(c.nonce(&subject), 1);
    assert!(c.status().conserved);
    // a second claim: nothing left to pay the fee from
    let again = net.claim(net.dormant_key as u8, 1, 140);
    assert!(matches!(c.validate_tx(&again, &[]), Err(TxError::Insufficient { .. })));
}

#[test]
fn coinbase_maturity_100_rejected_101_accepted() {
    let net = Net::new(2);
    let mut c = net.open("maturity");
    let miner = net.neurons[1];
    // the miner's liquid funds are exactly FUNDS; block 1's subsidy is locked below height 101
    mine(&mut c, miner, 0, &[]);
    let spend_all = |nonce: u64| net.transfer(1, nonce, net.neurons[0], FUNDS + net.subsidy() - 224, 224);
    for _ in 2..=99 {
        mine(&mut c, net.neurons[0], 0, &[]);
    }
    assert_eq!(c.tip().height, 99);
    // the next block is 100: still immature, in the mempool and in a block
    let err = c.validate_tx(&spend_all(0), &[]).unwrap_err();
    assert!(matches!(err, TxError::Immature { .. }), "{err:?}");
    let mut tpl = template(&mut c, net.neurons[0], 0, &[]);
    let cb = Tx { body: Body::Coinbase { height: 100, to: net.neurons[0], amount: net.subsidy() + 224, extra: [0; 32] }, sig: vec![] }.encode();
    let raws = vec![cb, spend_all(0)];
    tpl.body = tx::join_body(&raws);
    tpl.header.tx_root = crate::chain::tree::tx_root(&raws);
    let early = block_of(&tpl);
    assert!(matches!(c.accept_block(&early, NOW), Err(ChainError::Block(BlockError::Tx { index: 1, error: TxError::Immature { .. } }))));
    mine(&mut c, net.neurons[0], 0, &[]); // height 100 without the spend
    // the next block is 101 = 1 + COINBASE_MATURITY: spendable
    assert!(c.validate_tx(&spend_all(0), &[]).is_ok());
    mine(&mut c, net.neurons[0], 0, &[spend_all(0)]);
    assert_eq!(c.tip().height, 101);
    assert_eq!(c.balance(&miner, &net.native), 0);
    assert!(c.status().conserved);
}

#[test]
fn template_orders_by_fee_rate_and_keeps_nonce_order() {
    let net = Net::new(3);
    let mut c = net.open("template");
    let miner = net.neurons[2];
    let b = net.neurons[1];
    let cheap0 = net.transfer(0, 0, b, 1, 224);
    let rich1 = net.transfer(0, 1, b, 1, 10_000);
    let other0 = net.transfer(1, 0, net.neurons[0], 1, 500);
    let tpl = template(&mut c, miner, 0, &[rich1.clone(), other0.clone(), cheap0.clone()]);
    let chain = net.genesis.chain_id;
    let ids: Vec<_> = tpl.txids[1..].to_vec();
    assert_eq!(ids, vec![sigs::txid(&chain, &other0), sigs::txid(&chain, &cheap0), sigs::txid(&chain, &rich1)]);
    assert_eq!(tpl.fees, 224 + 10_000 + 500);
    let block = block_of(&tpl);
    assert_eq!(c.accept_block(&block, NOW), Ok(BlockVerdict::Extended));
    assert_eq!(c.nonce(&net.neurons[0]), 2);
    assert_eq!(c.balance(&miner, &net.native), FUNDS + net.subsidy() + tpl.fees);
}

#[test]
fn reopen_replays_bodies_and_verifies_roots() {
    let net = Net::new(2);
    let mut c = net.open("reopen");
    let miner = net.neurons[1];
    mine(&mut c, miner, 0, &[net.transfer(0, 0, miner, 9, 224)]);
    mine_n(&mut c, miner, 3);
    let tip = c.tip();
    let bal = c.balance(&miner, &net.native);
    drop(c);
    let c2 = net.reopen(0);
    assert_eq!(c2.tip(), tip);
    assert_eq!(c2.balance(&miner, &net.native), bal);
    assert_eq!(c2.nonce(&net.neurons[0]), 1);
    assert!(c2.status().conserved);
    // a different genesis refuses the store
    let other = Net::new(1);
    let dir = net.dirs.borrow()[0].clone();
    assert_eq!(Chain::open(&dir, &other.genesis.bytes, true).err(), Some(ChainError::Genesis));
}

#[test]
fn coinbase_amount_must_be_exact() {
    let net = Net::new(2);
    let mut c = net.open("coinbase");
    let miner = net.neurons[1];
    for delta in [1i64, -1] {
        let mut tpl = template(&mut c, miner, 0, &[]);
        let amount = (net.subsidy() as i64 + delta) as u64;
        let cb = Tx { body: Body::Coinbase { height: 1, to: miner, amount, extra: [0; 32] }, sig: vec![] }.encode();
        tpl.body = tx::join_body(&[cb.clone()]);
        tpl.header.tx_root = crate::chain::tree::tx_root(&[cb]);
        let block = block_of(&tpl);
        assert!(matches!(c.accept_block(&block, NOW), Err(ChainError::Block(BlockError::CoinbaseAmount { .. }))));
    }
    assert_eq!(c.tip().height, 0);
}

/// The wallet's signer and the kernel's validator agree: a transfer, a
/// cyberlink and a claim signed through `soft3::wallet` are admitted by
/// `validate_tx` and mined, and `wallet::verify` accepts what the kernel
/// signs.
#[test]
fn wallet_signed_transactions_pass_the_kernel_validator() {
    use crate::wallet::{self, Signer};
    let net = Net::new(2);
    let mut c = net.open("wallet");
    let chain = net.genesis.chain_id;
    let miner = net.neurons[1];
    let signer = Signer::from_key(net.keys[0].clone());
    assert_eq!(wallet::parse32(&signer.neuron_hex()).unwrap(), net.neurons[0]);
    let transfer = signer.transfer(&chain, 0, net.native, 9, miner, None).unwrap();
    let link = signer.cyberlink(&chain, 1, token("boot"), token("pussy"), 1, net.native, 0, None).unwrap();
    let dormant = Signer::from_key(key(net.dormant_key as u8));
    let claim = dormant.claim(&chain, 0, None).unwrap();
    for signed in [&transfer, &link, &claim] {
        let info = c.validate_tx(&signed.bytes, &[Tx::decode(&transfer.bytes).unwrap()]).unwrap();
        assert_eq!(info.txid, signed.txid, "wallet and kernel agree on the txid");
        assert!(wallet::verify(&chain, &signed.bytes).is_ok());
    }
    mine(&mut c, miner, 0, &[transfer.bytes.clone(), link.bytes.clone(), claim.bytes.clone()]);
    assert_eq!(c.nonce(&net.neurons[0]), 2);
    assert_eq!(c.balance(&miner, &net.native), FUNDS + net.subsidy() + 9 + 224 + 257 + 140);
    // and the kernel's own signing is what the wallet verifies
    let kernel_signed = net.transfer(0, 2, miner, 1, 224);
    assert!(wallet::verify(&chain, &kernel_signed).is_ok());
    assert!(c.status().conserved);
}
