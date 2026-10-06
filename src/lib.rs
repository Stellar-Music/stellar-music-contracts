#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short,
    token, Address, Env, String, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    TrackNotFound = 4,
    TrackInactive = 5,
    PassAlreadyPurchased = 6,
    InvalidPrice = 7,
    InvalidBasisPoints = 8,
    SplitAlreadyLocked = 9,
}

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

/// Level 2 architectural foundation: Split recipient structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitRecipient {
    pub recipient: Address,
    pub share_basis_points: u32, // 10000 = 100.00%
}

/// Level 2 architectural foundation: Multi-party revenue split agreement
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitAgreement {
    pub track_id: u64,
    pub recipients: Vec<SplitRecipient>,
    pub is_locked: bool,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Track(u64),
    Pass(Address, u64),
    TrackCount,
    TotalPasses,
    SplitAgreement(u64),
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
        env.storage().instance().set(&DataKey::TrackCount, &0u64);
        env.storage().instance().set(&DataKey::TotalPasses, &0u64);
        Ok(())
    }

    /// Register a new track by an artist
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

        env.storage().persistent().set(&DataKey::Track(count), &track);
        env.storage().instance().set(&DataKey::TrackCount, &count);

        env.events().publish(
            (symbol_short!("track"), symbol_short!("reg")),
            (count, artist, pass_price),
        );

        Ok(count)
    }

    /// Update track active status or pass price (Artist authorization required)
    pub fn update_track(
        env: Env,
        artist: Address,
        track_id: u64,
        pass_price: i128,
        is_active: bool,
    ) -> Result<(), Error> {
        artist.require_auth();

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

        track.pass_price = pass_price;
        track.is_active = is_active;
        env.storage().persistent().set(&track_key, &track);

        env.events().publish(
            (symbol_short!("track"), symbol_short!("upd")),
            (track_id, pass_price, is_active),
        );

        Ok(())
    }

    /// Purchase a Music Pass for a track using any SEP-41 / SAC Stellar token (e.g. Native XLM)
    pub fn purchase_pass(
        env: Env,
        listener: Address,
        track_id: u64,
        token: Address,
    ) -> Result<PassInfo, Error> {
        listener.require_auth();

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

        // Execute payment transfer from listener directly to track artist
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
        env.storage().instance().set(&DataKey::TotalPasses, &total_passes);

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

    /// Query Track Information by ID
    pub fn get_track(env: Env, track_id: u64) -> Option<TrackInfo> {
        env.storage().persistent().get(&DataKey::Track(track_id))
    }

    /// Query Pass Information for a listener and track
    pub fn get_pass(env: Env, listener: Address, track_id: u64) -> Option<PassInfo> {
        env.storage().persistent().get(&DataKey::Pass(listener, track_id))
    }

    /// Get total number of registered tracks
    pub fn get_track_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::TrackCount).unwrap_or(0)
    }

    /// Get total number of music passes issued platform-wide
    pub fn get_total_passes(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::TotalPasses).unwrap_or(0)
    }

    // =========================================================================
    // Level 2 Architecture Foundation: Signed Revenue Split Agreements
    // =========================================================================

    /// Register a multi-party revenue split for a track (Level 2 extensible hook)
    pub fn register_split_agreement(
        env: Env,
        artist: Address,
        track_id: u64,
        recipients: Vec<SplitRecipient>,
    ) -> Result<(), Error> {
        artist.require_auth();

        let track: TrackInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Track(track_id))
            .ok_or(Error::TrackNotFound)?;

        if track.artist != artist {
            return Err(Error::NotAuthorized);
        }

        let split_key = DataKey::SplitAgreement(track_id);
        if let Some(existing) = env.storage().persistent().get::<_, SplitAgreement>(&split_key) {
            if existing.is_locked {
                return Err(Error::SplitAlreadyLocked);
            }
        }

        // Verify total basis points equals exactly 10,000 (100.00%)
        let mut total_basis_points: u32 = 0;
        for recipient in recipients.iter() {
            total_basis_points += recipient.share_basis_points;
        }

        if total_basis_points != 10000 {
            return Err(Error::InvalidBasisPoints);
        }

        let agreement = SplitAgreement {
            track_id,
            recipients: recipients.clone(),
            is_locked: false,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&split_key, &agreement);

        env.events().publish(
            (symbol_short!("split"), symbol_short!("reg")),
            (track_id, recipients.len()),
        );

        Ok(())
    }

    /// Query split agreement for a track
    pub fn get_split_agreement(env: Env, track_id: u64) -> Option<SplitAgreement> {
        env.storage().persistent().get(&DataKey::SplitAgreement(track_id))
    }
}

#[cfg(test)]
mod test;
