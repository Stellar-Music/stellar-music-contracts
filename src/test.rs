#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
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
