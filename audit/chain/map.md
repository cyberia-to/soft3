---
title: chain — the requirement map
tags: cyber, soft3, audit, chain, requirements, map
crystal-type: audit
crystal-domain: cyber
date: 2026-10-22
alias: chain map, chain requirement map, chain requirements map
---

# chain — the requirement map

One row per requirement of [specs/chain.md](../../specs/chain.md): the code that implements it, the tests that check its cases, the receipt that observed it, and its state. The form is [cyberia/dev § requirements](../../../cyberia/dev.md): `open` — no code, no test or no case; `simulated` — tests green on synthetic input; `measured` — observed on the real path with a receipt at a commit; `proven` — a blessed conformance vector or a proof. `stale` is never written here: `scripts/requirements.py` computes it from git when the code changed after the receipt landed.

The map owns the links to code, test and proof; the clause owns its cases. Paths are relative to the repository; tests name `file::function`; a receipt is a page in this directory.

| requirement | code | test | proof | state |
|---|---|---|---|---|
| CHAIN-PARAMS | crate/src/chain/params.rs | params.rs::subsidy_is_one_percent_per_year_floored, rule_set_table_starts_at_genesis, epoch_clock_matches_chain_economy_section_10 | conformance-2026-10-16.md | simulated |
| CHAIN-HEADER | crate/src/chain/header.rs | header.rs::conformance_section_5_header_bytes_and_id; net/codec.rs::header_vector | conformance-2026-10-16.md | proven |
| CHAIN-POW | crate/src/chain/pow.rs | pow.rs::test_header_fails_its_own_target, conformance_section_5_mined_header; mine/target.rs::conformance_mined_header_verifies | hemera-0.4-2026-10-16.md | proven |
| CHAIN-BITS | crate/src/chain/header.rs | header.rs::conformance_section_2_compact_target; net/codec.rs::compact_target_vectors | conformance-2026-10-16.md | proven |
| CHAIN-TIME | crate/src/chain/daa.rs; crate/src/chain/engine.rs | daa.rs::mtp_lower_middle; tests/forks.rs::timestamps_mtp_rule_future_drift_and_bits; net/kernel/tests.rs::future_headers_are_held_then_retried_and_bad_blocks_are_invalid | conformance-2026-10-16.md | simulated |
| CHAIN-ASERT | crate/src/chain/daa.rs | daa.rs::conformance_section_4_rows, floor_div_rounds_toward_negative_infinity, factor_endpoints, t_spacing_holds_bits_and_half_spacing_roughly_doubles_difficulty, huge_positive_shift_saturates_at_limit; mine/asert_sim.rs::asert_behaviour_and_hundredfold_step | mine-2026-10-15.md | proven |
| CHAIN-BODY | crate/src/chain/tx.rs; crate/src/chain/apply.rs | tx.rs::body_split_round_trip; net/codec.rs::body_round_trip_and_budgets | conformance-2026-10-16.md | simulated |
| CHAIN-TXROOT | crate/src/chain/tree.rs | tree.rs::conformance_section_6_leaf_and_node, odd_nodes_promote_not_duplicate | conformance-2026-10-16.md | proven |
| CHAIN-TX | crate/src/chain/tx.rs | tx.rs::sizes_and_rejections_every_kind; net/codec.rs::encoding_rejections, transfer_statement_vector, sign_and_verify_binds_chain | conformance-2026-10-16.md | proven |
| CHAIN-COINBASE | crate/src/chain/apply.rs | tx.rs::conformance_section_6_coinbase_bytes; tests/rules.rs::coinbase_amount_must_be_exact, coinbase_maturity_100_rejected_101_accepted | slice-2026-10-16.md | measured |
| CHAIN-TRANSFER | crate/src/chain/apply.rs | tests/rules.rs::transfer_moves_value_and_the_nonce_advances, fee_floor_signature_and_balance_rules | conformance-2026-10-16.md | simulated |
| CHAIN-CYBERLINK | crate/src/chain/apply.rs | tests/rules.rs::cyberlink_rules_and_effects; tx.rs::valence_two_is_rejected; tests/economy.rs::staked_cyberlink_rules_kinds_6_and_7 | economy-2026-10-17.md | simulated |
| CHAIN-NONCE | crate/src/chain/apply.rs; crate/src/chain/net/mempool.rs | tests/rules.rs::transfer_moves_value_and_the_nonce_advances; net/mempool.rs::admission_order_and_nonce_window, a_relayed_nonce_beyond_our_window_is_a_race_not_an_offence; tests/simulate.rs::nonce_ordering_property_under_shuffled_submission | slice-2026-10-16.md | measured |
| CHAIN-FEE | crate/src/chain/apply.rs; crate/src/chain/net/mempool.rs | tests/rules.rs::fee_floor_signature_and_balance_rules; net/mempool.rs::fee_rate_is_compared_exactly | conformance-2026-10-16.md | simulated |
| CHAIN-SIG | crate/src/chain/sigs.rs | sigs.rs::signed_transfer_verifies_and_every_mutation_fails, conformance_section_6_transfer_statement; conformance.rs::signed_vectors_verify_and_are_deterministic | conformance-2026-10-16.md | proven |
| CHAIN-CLAIM | crate/src/chain/apply.rs; crate/src/chain/sigs.rs | tests/rules.rs::claim_moves_every_dormant_balance_to_the_key; sigs.rs::claim_subject_is_the_key_and_account_must_match, abandon_key_identity_and_legacy_owner | conformance-2026-10-16.md | simulated |
| CHAIN-IDS | crate/src/chain/sigs.rs | sigs.rs::conformance_section_1_and_7_anchors; net/codec.rs::hemera_anchors | hemera-0.4-2026-10-16.md | proven |
| CHAIN-STATE-DIMS | crate/src/chain/bbg_state.rs | bbg_state.rs::journal_reverts_every_dimension_exactly | economy-2026-10-17.md | simulated |
| CHAIN-STATE-ROOT | crate/src/chain/bbg_state.rs; crate/src/chain/state.rs | tests/rules.rs::reopen_replays_bodies_and_verifies_roots; tests/forks.rs::validation_order_pow_before_body_and_state_root_marks_invalid | slice-2026-10-16.md | measured |
| CHAIN-CONSERVATION | crate/src/chain/apply.rs; crate/src/chain/bbg_state.rs | tests/rules.rs::extends_tip_pays_subsidy_and_keeps_conservation; tests/simulate.rs::two_hundred_blocks_of_random_txs_keep_conservation_through_a_reorg; bbg_state.rs::debit_never_goes_negative_and_sums_track | economy-2026-10-17.md | simulated |
| CHAIN-SUBSIDY | crate/src/chain/params.rs; crate/src/chain/economy | params.rs::subsidy_is_one_percent_per_year_floored; tests/economy.rs::coinbase_split_follows_the_active_stake_of_the_last_boundary | economy-2026-10-17.md | simulated |
| CHAIN-RETENTION | crate/src/chain/store.rs | store.rs::headers_bodies_undo_and_main_chain_survive_reopen; tests/rules.rs::reopen_replays_bodies_and_verifies_roots | conformance-2026-10-16.md | simulated |
| CHAIN-GENESIS-FILE | crate/src/chain/genesis.rs | genesis.rs::subsidy_and_bits_are_checked, pussy_genesis_matches_the_blessed_vectors_when_cached | hemera-0.4-2026-10-16.md | proven |
| CHAIN-GENESIS-DERIVE | crate/src/genesis | genesis.rs::conformance_net_applies_through_the_pipeline_loader | hemera-0.4-2026-10-16.md | simulated |
| CHAIN-GENESIS-BLOCK | crate/src/chain/genesis.rs; crate/src/chain/params.rs | genesis.rs::links_apply_with_zero_amount_and_authors_get_records; params.rs::embedded_chains_have_the_launch_times | hemera-0.4-2026-10-16.md | simulated |
| CHAIN-TWO-CHAINS | crate/src/chain/params.rs | params.rs::embedded_chains_have_the_launch_times | slice-2026-10-16.md | simulated |
| CHAIN-FORK-CHOICE | crate/src/chain/select.rs; crate/src/chain/engine.rs | tests/forks.rs::heavier_fork_reorganizes_and_equals_a_fresh_replay, equal_work_keeps_the_first_seen_tip, three_stores_converge_deterministically; net/kernel/tests.rs::header_first_then_bodies_then_a_reorg_with_events | slice-2026-10-16.md | simulated |
| CHAIN-REORG-LIMIT | crate/src/chain/select.rs | select.rs::reorg_limit_is_119; tests/forks.rs::reorg_limit_rejects_a_fork_from_tip_minus_120_and_takes_tip_minus_119; net/kernel/tests.rs::a_refused_deep_fork_does_not_capture_body_sync | slice-2026-10-16.md | simulated |
| CHAIN-CHECKPOINTS | crate/src/chain/select.rs; crate/src/chain/net/kernel.rs | select.rs::checkpoints_verify_hold_and_veto; tests/forks.rs::checkpoints_veto_a_heavier_fork_and_reject_bad_signers; net/kernel/tests.rs::a_fresh_node_holding_a_checkpoint_downloads_through_it, a_checkpoint_that_contradicts_the_main_chain_moves_the_node_to_the_checkpointed_chain; tests/chain_checkpoints_on_connect.rs | slice-2026-10-16.md | simulated |
| CHAIN-ORPHANS | crate/src/chain/net/sync.rs | — | — | open |
| CHAIN-VALIDATION-ORDER | crate/src/chain/engine.rs | tests/forks.rs::validation_order_pow_before_body_and_state_root_marks_invalid | slice-2026-10-16.md | measured |
| CHAIN-RULESETS | crate/src/chain/params.rs | params.rs::rule_set_table_starts_at_genesis | conformance-2026-10-16.md | simulated |
| CHAIN-SHADOW | crate/src/chain/economy/run.rs | — | — | open |
| CHAIN-POOL-WIDE | crate/src/chain/pool | pool/tests.rs::wide_digest_extends_hemera_and_is_canonical, wide_digest_is_two_squeezes_of_the_xof | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-RECORD | crate/src/chain/pool | pool/tests.rs::record_owner_rho_commitment_nullifier | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-DERIVE | crate/src/chain/pool | pool/tests.rs::record_owner_rho_commitment_nullifier | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-TREE | crate/src/chain/pool | pool/tests.rs::tree_node_is_the_tagged_wide_digest, tree_roots_and_paths | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-TURNSTILE | — | — | — | open |
| CHAIN-POOL-SPEND | crate/src/chain/pool | pool/tests.rs::spend_encoding_is_canonical_and_strict | pool-formats-2026-10-16.md | simulated |
| CHAIN-POOL-TX | crate/src/chain/tx.rs | tx.rs::sizes_and_rejections_every_kind | conformance-2026-10-16.md | simulated |
| CHAIN-POOL-BLOCK-PROOF | — | — | — | open |
| CHAIN-POOL-TAGS | crate/src/chain/pool | pool/tests.rs::wide_digest_extends_hemera_and_is_canonical | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-GENESIS | crate/src/chain/genesis.rs | genesis.rs::subsidy_and_bits_are_checked | pool-formats-2026-10-16.md | proven |
| CHAIN-POOL-RULESETS | crate/src/chain/params.rs | params.rs::rule_set_table_starts_at_genesis | pool-formats-2026-10-16.md | simulated |
| CHAIN-ACCEPTANCE | crate/src/chain | — | slice-2026-10-16.md | open |

Notes on the states: `measured` rows cite the vertical slice (three nodes on one machine, every block mined and crossed the wire) or the economy run (two nodes agreeing on every epoch); they are not the 24 h two-machine run of §12, which is `CHAIN-ACCEPTANCE` and stays `open` until its receipt exists. `proven` rows are the vectors blessed into `conformance/chain.snap` and the pool formats re-blessed under hemera 0.4. `CHAIN-ORPHANS`, `CHAIN-SHADOW`, `CHAIN-POOL-TURNSTILE` and `CHAIN-POOL-BLOCK-PROOF` have no test named after their cases: rule set 2 is not implemented, and the orphan pool's eviction rules are untested.
