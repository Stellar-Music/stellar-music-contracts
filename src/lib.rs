//! # 🎵 Stellar Music Soroban Smart Contracts
//!
//! Decentralized music streaming and automated royalty settlement protocol
//! built on Soroban (Stellar) with non-custodial passes, cryptographic 10,000 bps
//! split agreements, zero-leak integer stroop arithmetic, and multi-recipient settlement.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env,
    String, Symbol, Vec,
};

/// Protocol Error codes returned by smart contract invocations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Contract instance has already been initialized
    AlreadyInitialized = 1,
    /// Contract instance requires initialization prior to use
    NotInitialized = 2,
    /// Caller does not possess required authorization or cryptographic signature
    NotAuthorized = 3,
    /// Specified track ID does not exist in persistent storage
    TrackNotFound = 4,
    /// Track is marked inactive by the creator
    TrackInactive = 5,
    /// Listener already holds a valid music pass for this track
    PassAlreadyPurchased = 6,
    /// Price parameter is invalid or negative
    InvalidPrice = 7,
    /// Collaborator basis points sum does not equal exactly 10,000 (100.00%)
    InvalidBasisPoints = 8,
    /// Split agreement has already received all approvals and is permanently locked
    SplitAlreadyLocked = 9,
    /// Specified revenue split agreement was not found
    AgreementNotFound = 10,
    /// Caller is not an authorized collaborator in this split agreement
    NotARecipient = 11,
    /// Collaborator has already registered approval for this agreement version
    AlreadyApproved = 12,
    /// Submitted terms hash does not match canonical agreement hash
    HashMismatch = 13,
    /// Contributor list contains duplicate recipient addresses
    DuplicateRecipient = 14,
    /// Contributor recipient list cannot be empty
    NoRecipients = 15,
    /// Revenue settlement requires a LOCKED agreement
    AgreementNotLocked = 16,
    /// Settlement distribution gross amount must be strictly positive
    InvalidSettlementAmount = 17,
    /// Contract is currently paused under emergency circuit breaker
    ContractPaused = 18,
    /// Pass price is below the MIN_PASS_PRICE floor
    PriceTooLow = 19,
    /// Number of split recipients exceeds MAX_RECIPIENTS limit
    TooManyRecipients = 20,
    /// Track is archived and cannot receive new pass purchases
    TrackArchived = 21,
    /// Metadata URI string cannot be empty
    MetadataUriEmpty = 22,
    /// Platform fee basis points exceed MAX_PLATFORM_FEE_BPS ceiling
    InvalidFeeBps = 23,
    /// Deposit amount must be strictly greater than zero
    ZeroDeposit = 24,
}

/// Minimum pass price floor: 0.01 XLM (100,000 stroops)
pub const MIN_PASS_PRICE: i128 = 100_000;
/// Maximum collaborator limit to prevent unbounded execution loops
pub const MAX_RECIPIENTS: u32 = 25;
/// Maximum allowable protocol fee in basis points (1,000 bps = 10.00%)
pub const MAX_PLATFORM_FEE_BPS: u32 = 1000;
/// Storage TTL extension parameters
pub const STORAGE_TTL_THRESHOLD: u32 = 100_000;
pub const STORAGE_TTL_EXTEND: u32 = 500_000;
/// Semantic Contract Version: v1.1.0 (Drips Wave Protocol Edition)
pub const CONTRACT_VERSION_MAJOR: u32 = 1;
pub const CONTRACT_VERSION_MINOR: u32 = 1;
pub const CONTRACT_VERSION_PATCH: u32 = 0;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackInfo {
    pub id: u64,
    pub artist: Address,
    pub pass_price: i128,
    pub metadata_uri: String,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassInfo {
    pub track_id: u64,
    pub listener: Address,
    pub amount_paid: i128,
    pub purchased_at: u64,
    pub ledger_sequence: u32,
}

/// Level 2: Collaborator Revenue Split Recipient
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitRecipient {
    pub recipient: Address,
    pub role: Symbol,            // e.g. "artist", "producer", "songwrtr", "engineer"
    pub share_basis_points: u32, // 10000 = 100.00%
}

/// Level 2: Multi-Party Revenue Split Agreement State
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitAgreement {
    pub track_id: u64,
    pub version: u32,
    pub agreement_hash: BytesN<32>, // Deterministic SHA-256 of agreement terms
    pub recipients: Vec<SplitRecipient>,
    pub approvals: Vec<Address>, // Collected cryptographic approvals
    pub is_locked: bool,         // Immutable once all required parties sign
    pub created_at: u64,
    pub locked_at: u64,
}

/// Level 3: Executed Multi-Recipient Settlement Receipt
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementReceipt {
    pub settlement_id: u64,
    pub track_id: u64,
    pub agreement_version: u32,
    pub gross_amount: i128,
    pub recipient_count: u32,
    pub settled_at: u64,
    pub ledger_sequence: u32,
}

/// Level 3: Individual Recipient Settlement Breakdown
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipientPayout {
    pub recipient: Address,
    pub role: Symbol,
    pub basis_points: u32,
    pub amount: i128,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    Treasury,
    PlatformFeeBps,
    Track(u64),
    TrackArchived(u64),
    Pass(Address, u64),
    ListenerPassCount(Address),
    TrackPoolBalance(u64),
    TrackCount,
    TotalPasses,
    SplitVersion(u64),        // Latest version number for track
    SplitAgreement(u64, u32), // (track_id, version) -> SplitAgreement
    ActiveLockedSplit(u64),   // track_id -> active locked version
    SettlementCount,          // Monotonically increasing settlement counter
    Settlement(u64),          // settlement_id -> SettlementReceipt
    TrackSettledTotal(u64),   // track_id -> cumulative settled amount
}

#[contract]
pub struct StellarMusicContract;

#[contractimpl]
impl StellarMusicContract {
    /// Initialize the platform contract with an administrator address
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &0u32);
        env.storage().instance().set(&DataKey::TrackCount, &0u64);
        env.storage().instance().set(&DataKey::TotalPasses, &0u64);
        Ok(())
    }

    /// Return the semantic contract version
    pub fn get_contract_version(_env: Env) -> (u32, u32, u32) {
        (
            CONTRACT_VERSION_MAJOR,
            CONTRACT_VERSION_MINOR,
            CONTRACT_VERSION_PATCH,
        )
    }

    /// Transfer platform administrator privileges to a new address
    pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
        current_admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if current_admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("transfer")),
            (current_admin, new_admin),
        );
        Ok(())
    }

    /// Upgrade contract WASM bytecode to a new installed hash (admin only)
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        env.events().publish(
            (symbol_short!("contract"), symbol_short!("upgrd")),
            new_wasm_hash,
        );
        Ok(())
    }

    /// Configure platform fee treasury address and basis points (capped at MAX_PLATFORM_FEE_BPS)
    pub fn set_platform_fee(
        env: Env,
        admin: Address,
        treasury: Address,
        fee_bps: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        if fee_bps > MAX_PLATFORM_FEE_BPS {
            return Err(Error::InvalidFeeBps);
        }
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &fee_bps);
        env.events().publish(
            (symbol_short!("fee"), symbol_short!("config")),
            (treasury, fee_bps),
        );
        Ok(())
    }

    /// Query the configured platform fee in basis points
    pub fn get_platform_fee_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(0)
    }

    /// Query the configured platform treasury address
    pub fn get_treasury(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Treasury)
    }

    /// Check if the contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Admin emergency control to pause or unpause protocol operations
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().instance().set(&DataKey::Paused, &paused);
        env.events()
            .publish((symbol_short!("contract"), symbol_short!("pause")), paused);
        Ok(())
    }

    /// Extend instance and persistent storage TTL
    pub fn extend_ttl(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(STORAGE_TTL_THRESHOLD, STORAGE_TTL_EXTEND);
    }

    /// Register a new track by an artist (Level 1)
    pub fn register_track(
        env: Env,
        artist: Address,
        pass_price: i128,
        metadata_uri: String,
    ) -> Result<u64, Error> {
        artist.require_auth();

        if pass_price < 0 {
            return Err(Error::InvalidPrice);
        }
        if pass_price > 0 && pass_price < MIN_PASS_PRICE {
            return Err(Error::PriceTooLow);
        }

        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TrackCount)
            .unwrap_or(0);
        count += 1;

        let track = TrackInfo {
            id: count,
            artist: artist.clone(),
            pass_price,
            metadata_uri,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Track(count), &track);
        env.storage().instance().set(&DataKey::TrackCount, &count);

        env.events().publish(
            (symbol_short!("track"), symbol_short!("reg")),
            (count, artist, pass_price),
        );

        Ok(count)
    }

    /// Update track active status or pass price
    pub fn update_track(
        env: Env,
        artist: Address,
        track_id: u64,
        pass_price: i128,
        is_active: bool,
    ) -> Result<(), Error> {
        artist.require_auth();

        if Self::is_track_archived(env.clone(), track_id) {
            return Err(Error::TrackArchived);
        }

        let track_key = DataKey::Track(track_id);
        let mut track: TrackInfo = env
            .storage()
            .persistent()
            .get(&track_key)
            .ok_or(Error::TrackNotFound)?;

        if track.artist != artist {
            return Err(Error::NotAuthorized);
        }
        if pass_price < 0 {
            return Err(Error::InvalidPrice);
        }
        if pass_price > 0 && pass_price < MIN_PASS_PRICE {
            return Err(Error::PriceTooLow);
        }

        track.pass_price = pass_price;
        track.is_active = is_active;
        env.storage().persistent().set(&track_key, &track);

        env.events().publish(
            (symbol_short!("track"), symbol_short!("upd")),
            (track_id, pass_price, is_active),
        );

        Ok(())
    }

    /// Archive a track permanently, preventing new pass sales
    pub fn archive_track(env: Env, artist: Address, track_id: u64) -> Result<(), Error> {
        artist.require_auth();
        let track: TrackInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Track(track_id))
            .ok_or(Error::TrackNotFound)?;
        if track.artist != artist {
            return Err(Error::NotAuthorized);
        }
        env.storage()
            .persistent()
            .set(&DataKey::TrackArchived(track_id), &true);
        env.events().publish(
            (symbol_short!("track"), symbol_short!("archive")),
            (track_id, artist),
        );
        Ok(())
    }

    /// Check if a track has been permanently archived
    pub fn is_track_archived(env: Env, track_id: u64) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::TrackArchived(track_id))
            .unwrap_or(false)
    }

    /// Update track metadata URI with validation against empty strings
    pub fn update_metadata_uri(
        env: Env,
        artist: Address,
        track_id: u64,
        metadata_uri: String,
    ) -> Result<(), Error> {
        artist.require_auth();
        if metadata_uri.is_empty() {
            return Err(Error::MetadataUriEmpty);
        }
        let track_key = DataKey::Track(track_id);
        let mut track: TrackInfo = env
            .storage()
            .persistent()
            .get(&track_key)
            .ok_or(Error::TrackNotFound)?;
        if track.artist != artist {
            return Err(Error::NotAuthorized);
        }
        if Self::is_track_archived(env.clone(), track_id) {
            return Err(Error::TrackArchived);
        }
        track.metadata_uri = metadata_uri.clone();
        env.storage().persistent().set(&track_key, &track);
        env.events().publish(
            (symbol_short!("track"), symbol_short!("meta")),
            (track_id, metadata_uri),
        );
        Ok(())
    }

    /// Purchase a Music Pass for a track using any SEP-41 / SAC Stellar token (Level 1)
    pub fn purchase_pass(
        env: Env,
        listener: Address,
        track_id: u64,
        token: Address,
    ) -> Result<PassInfo, Error> {
        listener.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(Error::ContractPaused);
        }

        if Self::is_track_archived(env.clone(), track_id) {
            return Err(Error::TrackArchived);
        }

        let track_key = DataKey::Track(track_id);
        let track: TrackInfo = env
            .storage()
            .persistent()
            .get(&track_key)
            .ok_or(Error::TrackNotFound)?;

        if !track.is_active {
            return Err(Error::TrackInactive);
        }

        let pass_key = DataKey::Pass(listener.clone(), track_id);
        if env.storage().persistent().has(&pass_key) {
            return Err(Error::PassAlreadyPurchased);
        }

        if track.pass_price > 0 {
            let client = token::Client::new(&env, &token);
            client.transfer(&listener, &track.artist, &track.pass_price);
        }

        let pass_info = PassInfo {
            track_id,
            listener: listener.clone(),
            amount_paid: track.pass_price,
            purchased_at: env.ledger().timestamp(),
            ledger_sequence: env.ledger().sequence(),
        };

        env.storage().persistent().set(&pass_key, &pass_info);

        let mut total_passes: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPasses)
            .unwrap_or(0);
        total_passes += 1;
        env.storage()
            .instance()
            .set(&DataKey::TotalPasses, &total_passes);

        // Increment listener cumulative pass counter
        let listener_count_key = DataKey::ListenerPassCount(listener.clone());
        let l_count: u64 = env
            .storage()
            .persistent()
            .get(&listener_count_key)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&listener_count_key, &(l_count + 1));

        env.events().publish(
            (symbol_short!("pass"), symbol_short!("bought")),
            (track_id, listener, track.artist, track.pass_price),
        );

        Ok(pass_info)
    }

    /// Check if a listener holds a valid Music Pass for a given track
    pub fn has_valid_pass(env: Env, listener: Address, track_id: u64) -> bool {
        let pass_key = DataKey::Pass(listener, track_id);
        env.storage().persistent().has(&pass_key)
    }

    /// Batch verify whether a listener holds valid passes across multiple tracks
    pub fn has_passes_batch(env: Env, listener: Address, track_ids: Vec<u64>) -> Vec<bool> {
        let mut results = Vec::new(&env);
        for id in track_ids.iter() {
            results.push_back(Self::has_valid_pass(env.clone(), listener.clone(), id));
        }
        results
    }

    /// Query the total number of music passes owned by a specific listener
    pub fn get_listener_pass_count(env: Env, listener: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::ListenerPassCount(listener))
            .unwrap_or(0)
    }

    /// Query Track Information by ID
    pub fn get_track(env: Env, track_id: u64) -> Option<TrackInfo> {
        env.storage().persistent().get(&DataKey::Track(track_id))
    }

    /// Query lightweight track pass price without loading full metadata
    pub fn get_track_pass_price(env: Env, track_id: u64) -> Option<i128> {
        let track: TrackInfo = env.storage().persistent().get(&DataKey::Track(track_id))?;
        Some(track.pass_price)
    }

    /// Query Pass Information for a listener and track
    pub fn get_pass(env: Env, listener: Address, track_id: u64) -> Option<PassInfo> {
        env.storage()
            .persistent()
            .get(&DataKey::Pass(listener, track_id))
    }

    /// Get total number of registered tracks
    pub fn get_track_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TrackCount)
            .unwrap_or(0)
    }

    /// Get total number of music passes issued platform-wide
    pub fn get_total_passes(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalPasses)
            .unwrap_or(0)
    }

    /// Query cumulative deposited revenue pool balance for a track
    pub fn get_track_pool_balance(env: Env, track_id: u64) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TrackPoolBalance(track_id))
            .unwrap_or(0)
    }

    /// Deposit revenue into a track pool
    pub fn deposit_track_revenue(
        env: Env,
        caller: Address,
        track_id: u64,
        amount: i128,
        token: Option<Address>,
    ) -> Result<i128, Error> {
        caller.require_auth();
        if amount <= 0 {
            return Err(Error::ZeroDeposit);
        }
        let _track: TrackInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Track(track_id))
            .ok_or(Error::TrackNotFound)?;
        if let Some(token_addr) = token {
            let client = token::Client::new(&env, &token_addr);
            client.transfer(&caller, &env.current_contract_address(), &amount);
        }
        let pool_key = DataKey::TrackPoolBalance(track_id);
        let current_balance: i128 = env.storage().persistent().get(&pool_key).unwrap_or(0);
        let new_balance = current_balance + amount;
        env.storage().persistent().set(&pool_key, &new_balance);
        env.events().publish(
            (symbol_short!("pool"), symbol_short!("deposit")),
            (track_id, caller, amount),
        );
        Ok(new_balance)
    }

    // =========================================================================
    // LEVEL 2: REVENUE SPLIT AGREEMENT FINANCIAL LAYER
    // =========================================================================

    /// Create a new multi-party revenue split agreement version
    pub fn create_split_agreement(
        env: Env,
        artist: Address,
        track_id: u64,
        agreement_hash: BytesN<32>,
        recipients: Vec<SplitRecipient>,
    ) -> Result<u32, Error> {
        artist.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(Error::ContractPaused);
        }

        let track: TrackInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Track(track_id))
            .ok_or(Error::TrackNotFound)?;

        if track.artist != artist {
            return Err(Error::NotAuthorized);
        }

        if recipients.is_empty() {
            return Err(Error::NoRecipients);
        }

        if recipients.len() > MAX_RECIPIENTS {
            return Err(Error::TooManyRecipients);
        }

        // Validate exactly 10,000 basis points (100.00%) & check for duplicate addresses
        let mut total_basis_points: u32 = 0;
        for i in 0..recipients.len() {
            let r = recipients.get(i).unwrap();
            if r.share_basis_points == 0 {
                return Err(Error::InvalidBasisPoints);
            }
            total_basis_points += r.share_basis_points;

            // Duplicate recipient check
            for j in (i + 1)..recipients.len() {
                let other = recipients.get(j).unwrap();
                if r.recipient == other.recipient {
                    return Err(Error::DuplicateRecipient);
                }
            }
        }

        if total_basis_points != 10000 {
            return Err(Error::InvalidBasisPoints);
        }

        // Increment version for this track
        let version_key = DataKey::SplitVersion(track_id);
        let mut version: u32 = env.storage().persistent().get(&version_key).unwrap_or(0);
        version += 1;
        env.storage().persistent().set(&version_key, &version);

        let mut approvals: Vec<Address> = Vec::new(&env);
        // If the creating artist is one of the recipients, record their initial approval
        let mut artist_in_recipients = false;
        for r in recipients.iter() {
            if r.recipient == artist {
                artist_in_recipients = true;
                break;
            }
        }
        if artist_in_recipients {
            approvals.push_back(artist.clone());
        }

        // Auto-lock if single recipient and already approved
        let is_locked = approvals.len() == recipients.len();
        let now = env.ledger().timestamp();
        let locked_at = if is_locked { now } else { 0 };

        let agreement = SplitAgreement {
            track_id,
            version,
            agreement_hash,
            recipients: recipients.clone(),
            approvals,
            is_locked,
            created_at: now,
            locked_at,
        };

        env.storage()
            .persistent()
            .set(&DataKey::SplitAgreement(track_id, version), &agreement);

        if is_locked {
            env.storage()
                .persistent()
                .set(&DataKey::ActiveLockedSplit(track_id), &version);

            env.events().publish(
                (symbol_short!("split"), symbol_short!("locked")),
                (track_id, version),
            );
        }

        env.events().publish(
            (symbol_short!("split"), symbol_short!("created")),
            (track_id, version, recipients.len()),
        );

        Ok(version)
    }

    /// Approve an exact split agreement by a contributor with cryptographic auth
    pub fn approve_split_agreement(
        env: Env,
        contributor: Address,
        track_id: u64,
        version: u32,
        agreement_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        contributor.require_auth();

        let agreement_key = DataKey::SplitAgreement(track_id, version);
        let mut agreement: SplitAgreement = env
            .storage()
            .persistent()
            .get(&agreement_key)
            .ok_or(Error::AgreementNotFound)?;

        if agreement.is_locked {
            return Err(Error::SplitAlreadyLocked);
        }

        // Enforce agreement hash match so signers are approving the exact terms
        if agreement.agreement_hash != agreement_hash {
            return Err(Error::HashMismatch);
        }

        // Verify caller is a legitimate recipient in this agreement
        let mut is_recipient = false;
        for r in agreement.recipients.iter() {
            if r.recipient == contributor {
                is_recipient = true;
                break;
            }
        }
        if !is_recipient {
            return Err(Error::NotARecipient);
        }

        // Check if already approved
        for a in agreement.approvals.iter() {
            if a == contributor {
                return Err(Error::AlreadyApproved);
            }
        }

        agreement.approvals.push_back(contributor.clone());

        // Check if all required recipients have signed
        if agreement.approvals.len() == agreement.recipients.len() {
            agreement.is_locked = true;
            agreement.locked_at = env.ledger().timestamp();

            // Set as active locked split for Level 3 settlement
            env.storage()
                .persistent()
                .set(&DataKey::ActiveLockedSplit(track_id), &version);

            env.events().publish(
                (symbol_short!("split"), symbol_short!("locked")),
                (track_id, version),
            );
        }

        env.storage().persistent().set(&agreement_key, &agreement);

        env.events().publish(
            (symbol_short!("split"), symbol_short!("apprvd")),
            (track_id, version, contributor),
        );

        Ok(agreement.is_locked)
    }

    /// Query split agreement by track ID and version
    pub fn get_split_agreement(env: Env, track_id: u64, version: u32) -> Option<SplitAgreement> {
        env.storage()
            .persistent()
            .get(&DataKey::SplitAgreement(track_id, version))
    }

    /// Query active locked split agreement for Level 3 settlement
    pub fn get_active_locked_split(env: Env, track_id: u64) -> Option<SplitAgreement> {
        let active_version: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveLockedSplit(track_id))?;

        env.storage()
            .persistent()
            .get(&DataKey::SplitAgreement(track_id, active_version))
    }

    /// Get latest split agreement version for a track
    pub fn get_latest_split_version(env: Env, track_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::SplitVersion(track_id))
            .unwrap_or(0)
    }

    /// Check if a split agreement version is locked
    pub fn is_agreement_locked(env: Env, track_id: u64, version: u32) -> bool {
        if let Some(agreement) = Self::get_split_agreement(env, track_id, version) {
            agreement.is_locked
        } else {
            false
        }
    }

    // =========================================================================
    // LEVEL 3: AUTOMATED REVENUE SETTLEMENT ENGINE
    // =========================================================================

    /// Calculate recipient allocations deterministically according to the active locked split agreement
    /// Uses integer-precise math (basis points: 10,000 = 100.00%) with deterministic remainder policy
    pub fn calculate_split_allocations(
        env: Env,
        track_id: u64,
        gross_amount: i128,
    ) -> Result<Vec<RecipientPayout>, Error> {
        if gross_amount <= 0 {
            return Err(Error::InvalidSettlementAmount);
        }

        let agreement =
            Self::get_active_locked_split(env.clone(), track_id).ok_or(Error::AgreementNotFound)?;

        if !agreement.is_locked {
            return Err(Error::AgreementNotLocked);
        }

        let mut payouts: Vec<RecipientPayout> = Vec::new(&env);
        let mut total_allocated: i128 = 0;

        for r in agreement.recipients.iter() {
            let amount = (gross_amount * (r.share_basis_points as i128)) / 10000i128;
            total_allocated += amount;
            payouts.push_back(RecipientPayout {
                recipient: r.recipient,
                role: r.role,
                basis_points: r.share_basis_points,
                amount,
            });
        }

        // Deterministic remainder allocation: assign any fractional rounding dust to primary artist (index 0)
        let remainder = gross_amount - total_allocated;
        if remainder > 0 && !payouts.is_empty() {
            let first = payouts.get(0).unwrap();
            let adjusted = RecipientPayout {
                recipient: first.recipient,
                role: first.role,
                basis_points: first.basis_points,
                amount: first.amount + remainder,
            };
            payouts.set(0, adjusted);
        }

        Ok(payouts)
    }

    /// Execute automated multi-recipient settlement backed by the active locked split agreement
    pub fn execute_split_settlement(
        env: Env,
        caller: Address,
        track_id: u64,
        gross_amount: i128,
        token: Option<Address>,
    ) -> Result<SettlementReceipt, Error> {
        caller.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(Error::ContractPaused);
        }

        if gross_amount <= 0 {
            return Err(Error::InvalidSettlementAmount);
        }

        let agreement =
            Self::get_active_locked_split(env.clone(), track_id).ok_or(Error::AgreementNotFound)?;

        if !agreement.is_locked {
            return Err(Error::AgreementNotLocked);
        }

        let payouts = Self::calculate_split_allocations(env.clone(), track_id, gross_amount)?;

        // Execute token transfers if token address is supplied
        if let Some(token_address) = token {
            let client = token::Client::new(&env, &token_address);
            for payout in payouts.iter() {
                if payout.amount > 0 {
                    client.transfer(&caller, &payout.recipient, &payout.amount);
                    env.events().publish(
                        (symbol_short!("settle"), symbol_short!("paid")),
                        (track_id, payout.recipient, payout.amount),
                    );
                }
            }
        }

        // Increment settlement sequence counter
        let count_key = DataKey::SettlementCount;
        let mut settlement_id: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
        settlement_id += 1;
        env.storage().instance().set(&count_key, &settlement_id);

        let now = env.ledger().timestamp();
        let receipt = SettlementReceipt {
            settlement_id,
            track_id,
            agreement_version: agreement.version,
            gross_amount,
            recipient_count: payouts.len(),
            settled_at: now,
            ledger_sequence: env.ledger().sequence(),
        };

        // Record settlement receipt
        env.storage()
            .persistent()
            .set(&DataKey::Settlement(settlement_id), &receipt);

        // Update track cumulative settled total
        let total_key = DataKey::TrackSettledTotal(track_id);
        let current_total: i128 = env.storage().persistent().get(&total_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&total_key, &(current_total + gross_amount));

        // Publish contract settlement events
        env.events().publish(
            (symbol_short!("settle"), symbol_short!("done")),
            (settlement_id, track_id, agreement.version, gross_amount),
        );

        Ok(receipt)
    }

    /// Query settlement receipt by ID
    pub fn get_settlement(env: Env, settlement_id: u64) -> Option<SettlementReceipt> {
        env.storage()
            .persistent()
            .get(&DataKey::Settlement(settlement_id))
    }

    /// Query total settlements executed platform-wide
    pub fn get_settlement_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::SettlementCount)
            .unwrap_or(0)
    }

    /// Query cumulative settled amount for a track
    pub fn get_track_settled_total(env: Env, track_id: u64) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TrackSettledTotal(track_id))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
