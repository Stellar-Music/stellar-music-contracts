# Stellar Music Protocol — Comprehensive Integration Testing Matrix

## 1. Test Harness Distribution Overview

The **Stellar Music** codebase enforces complete automated test coverage across smart contracts, off-chain backend orchestration, and frontend user interface logic.

| Repository | Test Framework | Test Files | Total Tests | Pass Rate | Execution Time |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **`stellar-music-contracts`** | Rust `cargo test` / Soroban Testutils | 1 file (`test.rs`) | **34** | **100% (34/34)** | ~0.9s |
| **`stellar-music-backend`** | Vitest v3.2 / Supertest | 19 suites | **89** | **100% (89/89)** | ~5.7s |
| **`stellar-music-frontend`** | Vitest v4.1 / Testing Library | 18 suites | **130** | **100% (130/130)**| ~5.7s |
| **GRAND TOTAL** | *Multi-Tier Automated Harness* | **38 suites** | **253 tests** | **100% (253/253)**| **< 15s Total** |

---

## 2. Smart Contracts Test Suite Breakdown (`stellar-music-contracts/src/test.rs`)

1. `test_contract_initialization`: Genesis admin setup and count initialization.
2. `test_contract_version`: Semantic version tuple `(1, 1, 0)` verification.
3. `test_admin_transfer_and_authorization`: Admin transfer and `require_auth` enforcement.
4. `test_platform_fee_configuration_and_caps`: Fee treasury and `MAX_PLATFORM_FEE_BPS` cap.
5. `test_emergency_pause_and_unpause`: Circuit breaker state transitions and event emission.
6. `test_register_track`: Active track creation with price and metadata URI.
7. `test_minimum_pass_price_enforcement`: Rejection of sub-minimum prices (<100,000 stroops).
8. `test_fuzz_purchase_pass_boundaries`: Fuzzing boundary values (free tracks, minimum price, max values).
9. `test_purchase_pass_and_payment_transfer`: SEP-41 token transfer and listener pass issuance.
10. `test_bulk_pass_check_and_listener_pass_count`: Batch pass queries and listener pass counters.
11. `test_update_metadata_uri_and_validation`: Artist URI updates and empty string rejection.
12. `test_track_archive_and_purchase_blocking`: Track archival and new pass sale denial.
13. `test_level2_create_split_agreement_success`: Multi-party split creation with 10,000 bps check.
14. `test_level2_reject_invalid_basis_points`: Rejection of under-allocated or over-allocated basis points.
15. `test_level2_reject_duplicate_recipient`: Duplicate contributor address detection and rejection.
16. `test_maximum_recipients_limit`: Enforcement of `MAX_RECIPIENTS` cap (25 parties).
17. `test_level2_multi_party_signing_flow_and_lock`: Cryptographic signature collection and auto-lock.
18. `test_level2_hash_mismatch_rejection`: Cryptographic hash mismatch signature rejection.
19. `test_level2_non_recipient_cannot_approve`: Denial of unauthorized non-recipient approval calls.
20. `test_level3_settlement_rejection_when_unlocked`: Settlement execution blocked if split is unlocked.
21. `test_level3_settlement_calculation_and_execution`: Integer stroop calculation and dry-run allocations.
22. `test_level3_settlement_invariant_and_rounding_dust`: Fractional dust remainder assignment to index 0.
23. `test_contract_stroop_loss_invariant`: 0-stroop precision loss invariant across diverse amounts.
24. `test_tiered_pass_purchase`: Standard vs collector pass pricing and independent ownership.
25. `test_pass_timestamp_and_ledger_sequence_invariant`: Ledger sequence and timestamp monotonicity.
26. `test_artist_treasury_withdraw`: Multi-party SAC token payouts and recipient balance verification.
27. `test_contract_event_emission_verification`: Event topic journal recording verification.
28. `test_unauthorized_signer_rejection`: Rejection of attacker calls to admin/artist functions.
29. `test_sac_token_transfer_cross_contract`: Multi-token cross-contract calling (XLM, USDC).
30. `test_contract_upgrade`: Soroban deployer bytecode upgradeability and auth check.
31. `test_maximum_supply_cap_invariant`: Total passes counter monotonicity and duplicate purchase rejection.
32. `test_signature_replay_defense`: Signature and agreement hash replay prevention.
33. `test_emergency_pause_settlement_blocking`: Circuit breaker blocking of settlements and passes.
34. `test_contract_error_codes_exhaustive`: Numeric discriminant alignment across all 24 error variants.
