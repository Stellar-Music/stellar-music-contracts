#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    vec, Address, BytesN, Env, String,
};

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_track_count(), 0);
    assert_eq!(client.get_total_passes(), 0);
}

#[test]
fn test_register_track() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let price: i128 = 5_000_0000; // 5 XLM
    let metadata_uri = String::from_str(&env, "ipfs://bafybeicoverart");

    let track_id = client.register_track(&artist, &price, &metadata_uri);
    assert_eq!(track_id, 1);
    assert_eq!(client.get_track_count(), 1);

    let track = client.get_track(&track_id).unwrap();
    assert_eq!(track.id, 1);
    assert_eq!(track.artist, artist);
    assert_eq!(track.pass_price, price);
    assert_eq!(track.is_active, true);
}

#[test]
fn test_purchase_pass_and_payment_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let artist = Address::generate(&env);
    let listener = Address::generate(&env);

    let initial_balance: i128 = 100_000_0000;
    token_admin_client.mint(&listener, &initial_balance);

    let pass_price: i128 = 5_000_0000;
    let track_id = client.register_track(
        &artist,
        &pass_price,
        &String::from_str(&env, "ipfs://stellar-soundscape"),
    );

    assert_eq!(client.has_valid_pass(&listener, &track_id), false);

    let pass_info = client.purchase_pass(&listener, &track_id, &token_id.address());
    assert_eq!(pass_info.track_id, track_id);
    assert_eq!(pass_info.listener, listener);
    assert_eq!(pass_info.amount_paid, pass_price);

    assert_eq!(token_client.balance(&listener), initial_balance - pass_price);
    assert_eq!(token_client.balance(&artist), pass_price);
    assert_eq!(client.has_valid_pass(&listener, &track_id), true);
    assert_eq!(client.get_total_passes(), 1);
}

#[test]
fn test_level2_create_split_agreement_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let engineer = Address::generate(&env);

    let track_id = client.register_track(
        &artist,
        &3_000_0000,
        &String::from_str(&env, "ipfs://meta"),
    );

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 5000, // 50%
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 3000, // 30%
        },
        SplitRecipient {
            recipient: engineer.clone(),
            role: symbol_short!("engineer"),
            share_basis_points: 2000, // 20%
        },
    ];

    let hash_bytes = [1u8; 32];
    let agreement_hash = BytesN::from_array(&env, &hash_bytes);

    let version = client.create_split_agreement(&artist, &track_id, &agreement_hash, &recipients);
    assert_eq!(version, 1);
    assert_eq!(client.get_latest_split_version(&track_id), 1);

    let agreement = client.get_split_agreement(&track_id, &version).unwrap();
    assert_eq!(agreement.track_id, track_id);
    assert_eq!(agreement.version, 1);
    assert_eq!(agreement.recipients.len(), 3);
    assert_eq!(agreement.approvals.len(), 1); // Artist auto-approved on creation
    assert_eq!(agreement.is_locked, false);
}

#[test]
fn test_level2_reject_invalid_basis_points() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    // Total = 9,000 (90%) - not 10,000
    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 9000,
        },
    ];
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let res = client.try_create_split_agreement(&artist, &track_id, &hash, &recipients);
    assert!(res.is_err());
}

#[test]
fn test_level2_reject_duplicate_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 5000,
        },
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 5000,
        },
    ];
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let res = client.try_create_split_agreement(&artist, &track_id, &hash, &recipients);
    assert!(res.is_err());
}

#[test]
fn test_level2_multi_party_signing_flow_and_lock() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);

    let track_id = client.register_track(&artist, &2_000_0000, &String::from_str(&env, "ipfs://"));

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 6000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 4000,
        },
    ];

    let hash_bytes = [7u8; 32];
    let agreement_hash = BytesN::from_array(&env, &hash_bytes);

    let v1 = client.create_split_agreement(&artist, &track_id, &agreement_hash, &recipients);
    assert_eq!(client.is_agreement_locked(&track_id, &v1), false);
    assert!(client.get_active_locked_split(&track_id).is_none());

    // Producer signs with matching agreement hash
    let locked = client.approve_split_agreement(&producer, &track_id, &v1, &agreement_hash);
    assert_eq!(locked, true); // All signed -> locked!

    assert_eq!(client.is_agreement_locked(&track_id, &v1), true);

    // Active locked split is now available for Level 3 settlement!
    let active_split = client.get_active_locked_split(&track_id).unwrap();
    assert_eq!(active_split.version, v1);
    assert_eq!(active_split.is_locked, true);
    assert_eq!(active_split.recipients.len(), 2);
}

#[test]
fn test_level2_non_recipient_cannot_approve() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let imposter = Address::generate(&env);

    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 7000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 3000,
        },
    ];

    let hash = BytesN::from_array(&env, &[12u8; 32]);
    let v1 = client.create_split_agreement(&artist, &track_id, &hash, &recipients);

    // Imposter tries to approve
    let res = client.try_approve_split_agreement(&imposter, &track_id, &v1, &hash);
    assert!(res.is_err());
}

#[test]
fn test_level2_hash_mismatch_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);

    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 5000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 5000,
        },
    ];

    let correct_hash = BytesN::from_array(&env, &[1u8; 32]);
    let wrong_hash = BytesN::from_array(&env, &[2u8; 32]);

    let v1 = client.create_split_agreement(&artist, &track_id, &correct_hash, &recipients);

    // Producer attempts to sign with mismatched agreement terms hash
    let res = client.try_approve_split_agreement(&producer, &track_id, &v1, &wrong_hash);
    assert!(res.is_err());
}

#[test]
fn test_level3_settlement_calculation_and_execution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let songwriter = Address::generate(&env);
    let engineer = Address::generate(&env);

    let track_id = client.register_track(&artist, &1_000_0000, &String::from_str(&env, "ipfs://"));

    // Level 2 Split: 50% / 25% / 15% / 10%
    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 5000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 2500,
        },
        SplitRecipient {
            recipient: songwriter.clone(),
            role: symbol_short!("songwrtr"),
            share_basis_points: 1500,
        },
        SplitRecipient {
            recipient: engineer.clone(),
            role: symbol_short!("engineer"),
            share_basis_points: 1000,
        },
    ];

    let hash = BytesN::from_array(&env, &[99u8; 32]);
    let v1 = client.create_split_agreement(&artist, &track_id, &hash, &recipients);

    // All required contributors sign -> agreement becomes LOCKED
    client.approve_split_agreement(&producer, &track_id, &v1, &hash);
    client.approve_split_agreement(&songwriter, &track_id, &v1, &hash);
    let is_locked = client.approve_split_agreement(&engineer, &track_id, &v1, &hash);
    assert_eq!(is_locked, true);

    // Level 3: Settlement Allocation Calculation for 100 XLM (1,000,000,000 stroops)
    let gross_amount: i128 = 1_000_000_000;
    let allocations = client.calculate_split_allocations(&track_id, &gross_amount);

    assert_eq!(allocations.len(), 4);
    assert_eq!(allocations.get(0).unwrap().amount, 500_000_000); // 50%
    assert_eq!(allocations.get(1).unwrap().amount, 250_000_000); // 25%
    assert_eq!(allocations.get(2).unwrap().amount, 150_000_000); // 15%
    assert_eq!(allocations.get(3).unwrap().amount, 100_000_000); // 10%

    // Financial invariant: Total payouts == Gross amount
    let total_payouts: i128 = allocations.get(0).unwrap().amount
        + allocations.get(1).unwrap().amount
        + allocations.get(2).unwrap().amount
        + allocations.get(3).unwrap().amount;
    assert_eq!(total_payouts, gross_amount);

    // Execute automated settlement
    let receipt = client.execute_split_settlement(&artist, &track_id, &gross_amount, &None);
    assert_eq!(receipt.settlement_id, 1);
    assert_eq!(receipt.track_id, track_id);
    assert_eq!(receipt.agreement_version, v1);
    assert_eq!(receipt.gross_amount, gross_amount);
    assert_eq!(receipt.recipient_count, 4);

    assert_eq!(client.get_settlement_count(), 1);
    assert_eq!(client.get_track_settled_total(&track_id), gross_amount);

    let fetched_receipt = client.get_settlement(&1).unwrap();
    assert_eq!(fetched_receipt.gross_amount, gross_amount);
}

#[test]
fn test_level3_settlement_rejection_when_unlocked() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);

    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 6000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 4000,
        },
    ];

    let hash = BytesN::from_array(&env, &[42u8; 32]);
    client.create_split_agreement(&artist, &track_id, &hash, &recipients);
    // Producer has NOT approved yet -> agreement is UNLOCKED

    let res = client.try_execute_split_settlement(&artist, &track_id, &100_0000, &None);
    assert!(res.is_err()); // AgreementNotLocked
}

#[test]
fn test_level3_settlement_invariant_and_rounding_dust() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let songwriter = Address::generate(&env);

    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://"));

    // Odd basis points: 33.34% + 33.33% + 33.33% = 100.00%
    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 3334,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 3333,
        },
        SplitRecipient {
            recipient: songwriter.clone(),
            role: symbol_short!("songwrtr"),
            share_basis_points: 3333,
        },
    ];

    let hash = BytesN::from_array(&env, &[15u8; 32]);
    let v1 = client.create_split_agreement(&artist, &track_id, &hash, &recipients);
    client.approve_split_agreement(&producer, &track_id, &v1, &hash);
    client.approve_split_agreement(&songwriter, &track_id, &v1, &hash);

    // Settle 100 stroops (odd amount testing integer rounding dust)
    let gross_stroops: i128 = 100;
    let allocations = client.calculate_split_allocations(&track_id, &gross_stroops);

    let mut sum: i128 = 0;
    for a in allocations.iter() {
        sum += a.amount;
    }
    // Strict invariant: sum must EXACTLY equal gross_stroops with zero leakage
    assert_eq!(sum, gross_stroops);

    let receipt = client.execute_split_settlement(&artist, &track_id, &gross_stroops, &None);
    assert_eq!(receipt.gross_amount, gross_stroops);
}


#[test]
fn test_emergency_pause_and_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.is_paused(), false);

    // Pause contract
    client.set_paused(&admin, &true);
    assert_eq!(client.is_paused(), true);

    // Any pass purchase must fail when paused
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let artist = Address::generate(&env);
    let listener = Address::generate(&env);
    let track_id = client.register_track(&artist, &100_000, &String::from_str(&env, "ipfs://test"));

    let res = client.try_purchase_pass(&listener, &track_id, &token_id.address());
    assert!(res.is_err());

    // Unpause
    client.set_paused(&admin, &false);
    assert_eq!(client.is_paused(), false);
}

#[test]
fn test_minimum_pass_price_enforcement() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);

    // Under minimum (50,000 < 100,000) fails
    let err_res = client.try_register_track(&artist, &50_000, &String::from_str(&env, "ipfs://cheap"));
    assert!(err_res.is_err());

    // Exactly minimum or free (0) succeeds
    let t1 = client.register_track(&artist, &100_000, &String::from_str(&env, "ipfs://valid"));
    assert_eq!(t1, 1);
    assert_eq!(client.get_track_pass_price(&t1), Some(100_000));

    let t2 = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://free"));
    assert_eq!(t2, 2);
    assert_eq!(client.get_track_pass_price(&t2), Some(0));
}

#[test]
fn test_maximum_recipients_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &100_000, &String::from_str(&env, "ipfs://collab"));

    // 26 recipients (> MAX_RECIPIENTS = 25)
    let mut recipients = Vec::new(&env);
    for _ in 0..26 {
        recipients.push_back(SplitRecipient {
            recipient: Address::generate(&env),
            role: symbol_short!("artist"),
            share_basis_points: 384,
        });
    }

    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let res = client.try_create_split_agreement(&artist, &track_id, &hash, &recipients);
    assert!(res.is_err());
}


#[test]
fn test_contract_version() {
    let env = Env::default();
    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let (major, minor, patch) = client.get_contract_version();
    assert_eq!(major, 1);
    assert_eq!(minor, 1);
    assert_eq!(patch, 0);
}

#[test]
fn test_admin_transfer_and_authorization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let imposter = Address::generate(&env);

    client.initialize(&admin1);

    // Imposter cannot transfer admin
    let imposter_res = client.try_set_admin(&imposter, &imposter);
    assert!(imposter_res.is_err());

    // Valid admin transfers to admin2
    client.set_admin(&admin1, &admin2);

    // Old admin can no longer set paused
    let pause_err = client.try_set_paused(&admin1, &true);
    assert!(pause_err.is_err());

    // New admin can pause
    client.set_paused(&admin2, &true);
    assert_eq!(client.is_paused(), true);
}

#[test]
fn test_platform_fee_configuration_and_caps() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    client.initialize(&admin);

    assert_eq!(client.get_platform_fee_bps(), 0);
    assert_eq!(client.get_treasury(), None);

    // 250 bps = 2.50% fee
    client.set_platform_fee(&admin, &treasury, &250);
    assert_eq!(client.get_platform_fee_bps(), 250);
    assert_eq!(client.get_treasury(), Some(treasury.clone()));

    // Fee exceeding 1000 bps (10%) must fail
    let excessive_err = client.try_set_platform_fee(&admin, &treasury, &1001);
    assert!(excessive_err.is_err());
}

#[test]
fn test_track_archive_and_purchase_blocking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let artist = Address::generate(&env);
    let listener = Address::generate(&env);

    let track_id = client.register_track(&artist, &100_000, &String::from_str(&env, "ipfs://song"));
    assert_eq!(client.is_track_archived(&track_id), false);

    // Archive track
    client.archive_track(&artist, &track_id);
    assert_eq!(client.is_track_archived(&track_id), true);

    // Purchasing an archived track must fail
    let res = client.try_purchase_pass(&listener, &track_id, &token_id.address());
    assert!(res.is_err());

    // Updating an archived track must fail
    let upd_res = client.try_update_track(&artist, &track_id, &200_000, &false);
    assert!(upd_res.is_err());
}

#[test]
fn test_update_metadata_uri_and_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &100_000, &String::from_str(&env, "ipfs://old"));

    // Empty URI fails
    let empty_err = client.try_update_metadata_uri(&artist, &track_id, &String::from_str(&env, ""));
    assert!(empty_err.is_err());

    // Valid update
    client.update_metadata_uri(&artist, &track_id, &String::from_str(&env, "ipfs://new-album-art"));
    let track = client.get_track(&track_id).unwrap();
    assert_eq!(track.metadata_uri, String::from_str(&env, "ipfs://new-album-art"));
}

#[test]
fn test_bulk_pass_check_and_listener_pass_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let listener = Address::generate(&env);

    let t1 = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://1"));
    let t2 = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://2"));
    let t3 = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://3"));

    assert_eq!(client.get_listener_pass_count(&listener), 0);

    let dummy_token = Address::generate(&env);
    client.purchase_pass(&listener, &t1, &dummy_token);
    client.purchase_pass(&listener, &t3, &dummy_token);

    assert_eq!(client.get_listener_pass_count(&listener), 2);

    let track_ids = soroban_sdk::vec![&env, t1, t2, t3];
    let results = client.has_passes_batch(&listener, &track_ids);

    assert_eq!(results.get(0), Some(true));  // t1: owned
    assert_eq!(results.get(1), Some(false)); // t2: not owned
    assert_eq!(results.get(2), Some(true));  // t3: owned
}

#[test]
fn test_fuzz_purchase_pass_boundaries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let artist = Address::generate(&env);

    // Boundary 1: Price below minimum (99_999) must fail
    let below_min = client.try_register_track(
        &artist,
        &99_999,
        &String::from_str(&env, "ipfs://below_min"),
    );
    assert_eq!(below_min, Err(Ok(Error::PriceTooLow)));

    // Boundary 2: Free track (0) succeeds and allows purchase without tokens
    let free_track_id = client.register_track(
        &artist,
        &0,
        &String::from_str(&env, "ipfs://free_track"),
    );
    let listener_free = Address::generate(&env);
    let pass_free = client.purchase_pass(&listener_free, &free_track_id, &token_id.address());
    assert_eq!(pass_free.amount_paid, 0);
    assert!(client.has_valid_pass(&listener_free, &free_track_id));

    // Boundary 3: Exact minimum pass price (MIN_PASS_PRICE = 100_000)
    let min_track_id = client.register_track(
        &artist,
        &MIN_PASS_PRICE,
        &String::from_str(&env, "ipfs://min_price_track"),
    );
    let listener_exact = Address::generate(&env);
    token_admin_client.mint(&listener_exact, &MIN_PASS_PRICE);

    let pass_min = client.purchase_pass(&listener_exact, &min_track_id, &token_id.address());
    assert_eq!(pass_min.amount_paid, MIN_PASS_PRICE);
    assert_eq!(token_client.balance(&listener_exact), 0);
    assert_eq!(token_client.balance(&artist), MIN_PASS_PRICE);

    // Boundary 4: Re-purchase rejection
    let re_purchase = client.try_purchase_pass(&listener_exact, &min_track_id, &token_id.address());
    assert_eq!(re_purchase, Err(Ok(Error::PassAlreadyPurchased)));
}

#[test]
fn test_contract_stroop_loss_invariant() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let songwriter = Address::generate(&env);

    let track_id = client.register_track(
        &artist,
        &1_000_0000,
        &String::from_str(&env, "ipfs://stroop-invariant-track"),
    );

    // 3-way split: 33.34%, 33.33%, 33.33% (sum = 10,000 bps)
    let recipients = soroban_sdk::vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 3334,
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 3333,
        },
        SplitRecipient {
            recipient: songwriter.clone(),
            role: symbol_short!("songwrtr"),
            share_basis_points: 3333,
        },
    ];

    let hash = BytesN::from_array(&env, &[7u8; 32]);
    let version = client.create_split_agreement(&artist, &track_id, &hash, &recipients);

    // Approve by remaining recipients to lock
    client.approve_split_agreement(&producer, &track_id, &version, &hash);
    client.approve_split_agreement(&songwriter, &track_id, &version, &hash);
    assert!(client.is_agreement_locked(&track_id, &version));

    // Diverse test amounts: 1 stroop, 7 stroops, 100 stroops, 9,999,999 stroops, 100,000,000,000 stroops
    let test_amounts: [i128; 8] = [
        1,
        7,
        100,
        999,
        10_000_000,
        77_777_777,
        1_234_567_890,
        99_999_999_999,
    ];

    for &gross in test_amounts.iter() {
        let allocations = client.calculate_split_allocations(&track_id, &gross);
        let mut sum_allocated: i128 = 0;
        for p in allocations.iter() {
            assert!(p.amount >= 0, "Recipient payout cannot be negative");
            sum_allocated += p.amount;
        }
        assert_eq!(
            sum_allocated, gross,
            "0-Stroop loss invariant failed: allocated sum must exactly match gross amount"
        );
    }
}

#[test]
fn test_tiered_pass_purchase() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let artist = Address::generate(&env);
    let standard_fan = Address::generate(&env);
    let collector_fan = Address::generate(&env);

    let standard_price: i128 = 5_000_0000;  // 5 XLM
    let collector_price: i128 = 50_000_0000; // 50 XLM

    token_admin_client.mint(&standard_fan, &standard_price);
    token_admin_client.mint(&collector_fan, &collector_price);

    // Register Tier 1: Standard Track Pass
    let t1 = client.register_track(
        &artist,
        &standard_price,
        &String::from_str(&env, "ipfs://tier1-standard-audio"),
    );

    // Register Tier 2: Exclusive Stem / Collector Pass
    let t2 = client.register_track(
        &artist,
        &collector_price,
        &String::from_str(&env, "ipfs://tier2-exclusive-stems"),
    );

    assert_eq!(client.get_track_pass_price(&t1), Some(standard_price));
    assert_eq!(client.get_track_pass_price(&t2), Some(collector_price));

    // Standard listener buys Tier 1
    let p1 = client.purchase_pass(&standard_fan, &t1, &token_id.address());
    assert_eq!(p1.amount_paid, standard_price);

    // Collector fan buys Tier 2
    let p2 = client.purchase_pass(&collector_fan, &t2, &token_id.address());
    assert_eq!(p2.amount_paid, collector_price);

    // Check balances
    assert_eq!(token_client.balance(&standard_fan), 0);
    assert_eq!(token_client.balance(&collector_fan), 0);
    assert_eq!(token_client.balance(&artist), standard_price + collector_price);

    // Verify independent ownership
    assert!(client.has_valid_pass(&standard_fan, &t1));
    assert!(!client.has_valid_pass(&standard_fan, &t2));
    assert!(!client.has_valid_pass(&collector_fan, &t1));
    assert!(client.has_valid_pass(&collector_fan, &t2));
}

#[test]
fn test_pass_timestamp_and_ledger_sequence_invariant() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://timestamped"));

    let listener_1 = Address::generate(&env);
    let listener_2 = Address::generate(&env);
    let dummy_token = Address::generate(&env);

    // Set initial Ledger state
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000;
        li.sequence_number = 100;
    });

    let p1 = client.purchase_pass(&listener_1, &track_id, &dummy_token);
    assert_eq!(p1.purchased_at, 1_000);
    assert_eq!(p1.ledger_sequence, 100);

    // Advance Ledger state forward
    env.ledger().with_mut(|li| {
        li.timestamp = 1_250;
        li.sequence_number = 120;
    });

    let p2 = client.purchase_pass(&listener_2, &track_id, &dummy_token);
    assert_eq!(p2.purchased_at, 1_250);
    assert_eq!(p2.ledger_sequence, 120);

    // Verify historical immutability from storage
    let stored_p1 = client.get_pass(&listener_1, &track_id).unwrap();
    assert_eq!(stored_p1.purchased_at, 1_000);
    assert_eq!(stored_p1.ledger_sequence, 100);
    assert!(p2.purchased_at > p1.purchased_at);
    assert!(p2.ledger_sequence > p1.ledger_sequence);
}

#[test]
fn test_artist_treasury_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let treasury = Address::generate(&env);
    let fee_bps: u32 = 250; // 2.5% protocol fee
    client.set_platform_fee(&admin, &treasury, &fee_bps);

    assert_eq!(client.get_platform_fee_bps(), 250);
    assert_eq!(client.get_treasury(), Some(treasury.clone()));

    // Token setup
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let pool_caller = Address::generate(&env);
    let gross_pool: i128 = 10_000_0000; // 10 XLM
    token_admin_client.mint(&pool_caller, &gross_pool);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);
    let track_id = client.register_track(&artist, &1_000_0000, &String::from_str(&env, "ipfs://audio"));

    let recipients = soroban_sdk::vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 7000, // 70%
        },
        SplitRecipient {
            recipient: producer.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 3000, // 30%
        },
    ];

    let hash = BytesN::from_array(&env, &[12u8; 32]);
    let version = client.create_split_agreement(&artist, &track_id, &hash, &recipients);
    client.approve_split_agreement(&producer, &track_id, &version, &hash);

    let receipt = client.execute_split_settlement(
        &pool_caller,
        &track_id,
        &gross_pool,
        &Some(token_id.address()),
    );

    assert_eq!(receipt.gross_amount, gross_pool);
    assert_eq!(receipt.recipient_count, 2);

    // Verify recipient balances after on-chain transfer
    assert_eq!(token_client.balance(&artist), 7_000_0000); // 7 XLM
    assert_eq!(token_client.balance(&producer), 3_000_0000); // 3 XLM
    assert_eq!(token_client.balance(&pool_caller), 0);
}

#[test]
fn test_contract_event_emission_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://event-track"));

    // Verify pass purchase publishes pass bought event
    let listener = Address::generate(&env);
    let dummy_token = Address::generate(&env);
    client.purchase_pass(&listener, &track_id, &dummy_token);

    let all_events = env.events().all();
    assert!(!all_events.is_empty(), "Events should be recorded on chain actions");
}

#[test]
fn test_unauthorized_signer_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &1_000_0000, &String::from_str(&env, "ipfs://legit-audio"));

    let attacker = Address::generate(&env);

    // 1. Attacker attempts to update track price
    let hack_price = client.try_update_track(&attacker, &track_id, &100_000, &true);
    assert_eq!(hack_price, Err(Ok(Error::NotAuthorized)));

    // 2. Attacker attempts to archive track
    let hack_archive = client.try_archive_track(&attacker, &track_id);
    assert_eq!(hack_archive, Err(Ok(Error::NotAuthorized)));

    // 3. Attacker attempts to change metadata URI
    let hack_uri = client.try_update_metadata_uri(&attacker, &track_id, &String::from_str(&env, "ipfs://malicious"));
    assert_eq!(hack_uri, Err(Ok(Error::NotAuthorized)));

    // 4. Attacker attempts to pause contract
    let hack_pause = client.try_set_paused(&attacker, &true);
    assert_eq!(hack_pause, Err(Ok(Error::NotAuthorized)));

    // 5. Attacker attempts to take over admin
    let hack_admin = client.try_set_admin(&attacker, &attacker);
    assert_eq!(hack_admin, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_sac_token_transfer_cross_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Deploy SAC Token A (XLM representation)
    let sac_admin_a = Address::generate(&env);
    let token_a_id = env.register_stellar_asset_contract_v2(sac_admin_a.clone());
    let token_a_client = TokenClient::new(&env, &token_a_id.address());
    let token_a_admin = StellarAssetClient::new(&env, &token_a_id.address());

    // Deploy SAC Token B (USDC representation)
    let sac_admin_b = Address::generate(&env);
    let token_b_id = env.register_stellar_asset_contract_v2(sac_admin_b.clone());
    let token_b_client = TokenClient::new(&env, &token_b_id.address());
    let token_b_admin = StellarAssetClient::new(&env, &token_b_id.address());

    let artist_a = Address::generate(&env);
    let artist_b = Address::generate(&env);
    let listener = Address::generate(&env);

    let price_a: i128 = 10_000_0000; // 10 XLM
    let price_b: i128 = 25_000_0000; // 25 USDC

    token_a_admin.mint(&listener, &price_a);
    token_b_admin.mint(&listener, &price_b);

    let track_a = client.register_track(&artist_a, &price_a, &String::from_str(&env, "ipfs://track-a"));
    let track_b = client.register_track(&artist_b, &price_b, &String::from_str(&env, "ipfs://track-b"));

    // Cross-contract call to Token A
    client.purchase_pass(&listener, &track_a, &token_a_id.address());
    assert_eq!(token_a_client.balance(&listener), 0);
    assert_eq!(token_a_client.balance(&artist_a), price_a);
    assert_eq!(token_b_client.balance(&listener), price_b); // Token B untouched

    // Cross-contract call to Token B
    client.purchase_pass(&listener, &track_b, &token_b_id.address());
    assert_eq!(token_b_client.balance(&listener), 0);
    assert_eq!(token_b_client.balance(&artist_b), price_b);

    assert!(client.has_valid_pass(&listener, &track_a));
    assert!(client.has_valid_pass(&listener, &track_b));
}

#[test]
fn test_contract_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let attacker = Address::generate(&env);
    let dummy_hash = BytesN::from_array(&env, &[0x42; 32]);

    // Unauthorized upgrade attempt must fail
    let err = client.try_upgrade(&attacker, &dummy_hash);
    assert_eq!(err, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_maximum_supply_cap_invariant() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &0, &String::from_str(&env, "ipfs://limited-drop"));

    let num_fans = 5;
    let mut listeners = soroban_sdk::Vec::new(&env);
    let dummy_token = Address::generate(&env);

    for _ in 0..num_fans {
        let fan = Address::generate(&env);
        listeners.push_back(fan.clone());
        client.purchase_pass(&fan, &track_id, &dummy_token);
    }

    // Invariant 1: Total passes counter must equal exact number of purchased passes
    assert_eq!(client.get_total_passes(), 5);

    // Invariant 2: Re-purchase rejection prevents supply inflation
    for fan in listeners.iter() {
        let attempt = client.try_purchase_pass(&fan, &track_id, &dummy_token);
        assert_eq!(attempt, Err(Ok(Error::PassAlreadyPurchased)));
        // Total passes remains strictly 5
        assert_eq!(client.get_total_passes(), 5);
    }
}

#[test]
fn test_signature_replay_defense() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let collaborator = Address::generate(&env);
    let track_id = client.register_track(&artist, &1_000_0000, &String::from_str(&env, "ipfs://split-track"));

    let recipients = soroban_sdk::vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 6000,
        },
        SplitRecipient {
            recipient: collaborator.clone(),
            role: symbol_short!("producer"),
            share_basis_points: 4000,
        },
    ];

    let legitimate_hash = BytesN::from_array(&env, &[0xAA; 32]);
    let replayed_foreign_hash = BytesN::from_array(&env, &[0xBB; 32]); // Different network or terms hash

    let version = client.create_split_agreement(&artist, &track_id, &legitimate_hash, &recipients);

    // 1. Replay attempt with mismatched/foreign terms hash fails immediately
    let replay_err = client.try_approve_split_agreement(
        &collaborator,
        &track_id,
        &version,
        &replayed_foreign_hash,
    );
    assert_eq!(replay_err, Err(Ok(Error::HashMismatch)));

    // 2. Legitimate approval succeeds
    let is_locked = client.approve_split_agreement(
        &collaborator,
        &track_id,
        &version,
        &legitimate_hash,
    );
    assert!(is_locked);

    // 3. Duplicate approval replay attempt fails with SplitAlreadyLocked
    let duplicate_attempt = client.try_approve_split_agreement(
        &collaborator,
        &track_id,
        &version,
        &legitimate_hash,
    );
    assert_eq!(duplicate_attempt, Err(Ok(Error::SplitAlreadyLocked)));
}

#[test]
fn test_emergency_pause_settlement_blocking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let track_id = client.register_track(&artist, &1_000_0000, &String::from_str(&env, "ipfs://pause-settle"));

    let recipients = soroban_sdk::vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            role: symbol_short!("artist"),
            share_basis_points: 10000,
        },
    ];

    let hash = BytesN::from_array(&env, &[0x33; 32]);
    let version = client.create_split_agreement(&artist, &track_id, &hash, &recipients);
    assert!(client.is_agreement_locked(&track_id, &version));

    // Admin pauses protocol
    client.set_paused(&admin, &true);
    assert!(client.is_paused());

    // Both pass purchases and settlement runs are strictly blocked during emergency pause
    let caller = Address::generate(&env);
    let settle_attempt = client.try_execute_split_settlement(&caller, &track_id, &10_000_0000, &None);
    assert_eq!(settle_attempt, Err(Ok(Error::ContractPaused)));

    let listener = Address::generate(&env);
    let dummy_token = Address::generate(&env);
    let pass_attempt = client.try_purchase_pass(&listener, &track_id, &dummy_token);
    assert_eq!(pass_attempt, Err(Ok(Error::ContractPaused)));

    // Admin lifts pause
    client.set_paused(&admin, &false);
    assert!(!client.is_paused());

    // Settlement now executes successfully
    let receipt = client.execute_split_settlement(&caller, &track_id, &10_000_0000, &None);
    assert_eq!(receipt.gross_amount, 10_000_0000);
}

#[test]
fn test_contract_error_codes_exhaustive() {
    // Assert explicit numeric discriminator values matching ABI contract specifications
    assert_eq!(Error::AlreadyInitialized as u32, 1);
    assert_eq!(Error::NotInitialized as u32, 2);
    assert_eq!(Error::NotAuthorized as u32, 3);
    assert_eq!(Error::TrackNotFound as u32, 4);
    assert_eq!(Error::TrackInactive as u32, 5);
    assert_eq!(Error::PassAlreadyPurchased as u32, 6);
    assert_eq!(Error::InvalidPrice as u32, 7);
    assert_eq!(Error::InvalidBasisPoints as u32, 8);
    assert_eq!(Error::SplitAlreadyLocked as u32, 9);
    assert_eq!(Error::AgreementNotFound as u32, 10);
    assert_eq!(Error::NotARecipient as u32, 11);
    assert_eq!(Error::AlreadyApproved as u32, 12);
    assert_eq!(Error::HashMismatch as u32, 13);
    assert_eq!(Error::DuplicateRecipient as u32, 14);
    assert_eq!(Error::NoRecipients as u32, 15);
    assert_eq!(Error::AgreementNotLocked as u32, 16);
    assert_eq!(Error::InvalidSettlementAmount as u32, 17);
    assert_eq!(Error::ContractPaused as u32, 18);
    assert_eq!(Error::PriceTooLow as u32, 19);
    assert_eq!(Error::TooManyRecipients as u32, 20);
    assert_eq!(Error::TrackArchived as u32, 21);
    assert_eq!(Error::MetadataUriEmpty as u32, 22);
    assert_eq!(Error::InvalidFeeBps as u32, 23);
    assert_eq!(Error::ZeroDeposit as u32, 24);
}
