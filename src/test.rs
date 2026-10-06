#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, String,
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

    // Setup Token (SAC simulation)
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = TokenClient::new(&env, &token_id.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let artist = Address::generate(&env);
    let listener = Address::generate(&env);

    // Mint tokens to listener
    let initial_balance: i128 = 100_000_0000;
    token_admin_client.mint(&listener, &initial_balance);
    assert_eq!(token_client.balance(&listener), initial_balance);
    assert_eq!(token_client.balance(&artist), 0);

    // Register track with 5 XLM pass price
    let pass_price: i128 = 5_000_0000;
    let track_id = client.register_track(
        &artist,
        &pass_price,
        &String::from_str(&env, "ipfs://stellar-soundscape"),
    );

    // Verify initial pass state
    assert_eq!(client.has_valid_pass(&listener, &track_id), false);

    // Listener purchases pass
    let pass_info = client.purchase_pass(&listener, &track_id, &token_id.address());
    assert_eq!(pass_info.track_id, track_id);
    assert_eq!(pass_info.listener, listener);
    assert_eq!(pass_info.amount_paid, pass_price);

    // Check financial settlement: funds transferred directly to artist
    assert_eq!(token_client.balance(&listener), initial_balance - pass_price);
    assert_eq!(token_client.balance(&artist), pass_price);

    // Check pass access state
    assert_eq!(client.has_valid_pass(&listener, &track_id), true);
    assert_eq!(client.get_total_passes(), 1);

    let retrieved_pass = client.get_pass(&listener, &track_id).unwrap();
    assert_eq!(retrieved_pass.amount_paid, pass_price);
}

#[test]
fn test_duplicate_purchase_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = StellarAssetClient::new(&env, &token_id.address());

    let artist = Address::generate(&env);
    let listener = Address::generate(&env);

    token_admin_client.mint(&listener, &100_000_0000);

    let track_id = client.register_track(
        &artist,
        &2_000_0000,
        &String::from_str(&env, "ipfs://single"),
    );

    client.purchase_pass(&listener, &track_id, &token_id.address());

    // Second purchase attempt should fail with PassAlreadyPurchased error
    let res = client.try_purchase_pass(&listener, &track_id, &token_id.address());
    assert!(res.is_err());
}

#[test]
fn test_cannot_purchase_inactive_track() {
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

    let track_id = client.register_track(
        &artist,
        &2_000_0000,
        &String::from_str(&env, "ipfs://single"),
    );

    // Deactivate track
    client.update_track(&artist, &track_id, &2_000_0000, &false);

    let res = client.try_purchase_pass(&listener, &track_id, &token_id.address());
    assert!(res.is_err());
}

#[test]
fn test_level2_split_agreement_foundation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StellarMusicContract, ());
    let client = StellarMusicContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let artist = Address::generate(&env);
    let producer = Address::generate(&env);

    let track_id = client.register_track(
        &artist,
        &5_000_0000,
        &String::from_str(&env, "ipfs://collab"),
    );

    // Valid split: 70% artist (7000 bps) + 30% producer (3000 bps) = 10000 bps
    let recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            share_basis_points: 7000,
        },
        SplitRecipient {
            recipient: producer.clone(),
            share_basis_points: 3000,
        },
    ];

    client.register_split_agreement(&artist, &track_id, &recipients);

    let agreement = client.get_split_agreement(&track_id).unwrap();
    assert_eq!(agreement.track_id, track_id);
    assert_eq!(agreement.recipients.len(), 2);
    assert_eq!(agreement.is_locked, false);

    // Invalid split total (e.g. 8000 bps != 10000) should fail
    let invalid_recipients = vec![
        &env,
        SplitRecipient {
            recipient: artist.clone(),
            share_basis_points: 5000,
        },
    ];
    let res = client.try_register_split_agreement(&artist, &track_id, &invalid_recipients);
    assert!(res.is_err());
}
