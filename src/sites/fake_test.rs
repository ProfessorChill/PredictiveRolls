use lazy_static::lazy_static;
use rand::Rng;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256, Sha512};
use std::sync::Mutex;

use crate::sites::duck_dice::{AbsoluteLevel, Bet, BetMakeResponse, User};
use crate::sites::free_bitco_in::BetSiteResult;

lazy_static! {
    pub static ref SERVER_STORAGE: Mutex<FakeServerStorage> =
        Mutex::new(FakeServerStorage::default());
}

#[derive(Debug)]
pub struct FakeServerStorage {
    pub server_seed_hash_previous_roll: String,
    pub server_seed_hash_next_roll: String,
    pub server_seed_previous_roll: u32,
    pub previous_nonce: u64,
    pub current_nonce: u64,
    pub next_nonce: u64,
    pub current_seed_hash: String,
    pub previous_roll: u32,
    pub current_roll: u32,
    pub next_roll: u32,
    pub initialized: bool,
    pub client_seed: String,
    pub server_seed: String,
}

impl Default for FakeServerStorage {
    fn default() -> Self {
        Self {
            server_seed_hash_previous_roll: String::new(),
            server_seed_hash_next_roll: String::new(),
            server_seed_previous_roll: 0,
            previous_nonce: 0,
            current_nonce: 0,
            next_nonce: 0,
            current_seed_hash: String::new(),
            previous_roll: 0,
            current_roll: 0,
            next_roll: 0,
            initialized: false,
            client_seed: String::new(),
            server_seed: String::new(),
        }
    }
}

/// Returns: (rolled_number, server_seed, nonce)
pub fn gen_fake_bet(
    server_storage: &mut FakeServerStorage,
    client_seed: &str,
) -> (u32, String, u64) {
    if server_storage.current_nonce == 0 {
        server_storage.server_seed = rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
    }
    let mut hasher = Sha256::new();
    hasher.update(&server_storage.server_seed);
    let result = hasher.finalize();
    let server_seed_hash = hex::encode(result);

    // DuckdiceIO Dice Generation.
    let mut combined_seed = Vec::new();
    combined_seed.extend_from_slice(server_storage.server_seed.as_bytes());
    combined_seed.extend_from_slice(client_seed.as_bytes());
    combined_seed.extend_from_slice(server_storage.current_nonce.to_string().as_bytes());

    let mut hasher = Sha512::new();
    hasher.update(combined_seed);
    let result = hasher.finalize();
    let hash = hex::encode(result);
    let mut lucky = 100000000;
    let mut index = 0;
    while lucky > 1000000 {
        lucky = u32::from_str_radix(&hash[index..=index + 4], 16).unwrap();
        index += 5;
    }
    let number = lucky % 10000;

    (number, server_seed_hash, server_storage.current_nonce)
}

pub fn free_bitcoin_fake_bet(
    high: bool,
    client_seed: &str,
    stake: f32,
    multiplier: f32,
) -> BetSiteResult {
    let server_storage: &mut FakeServerStorage = &mut SERVER_STORAGE.lock().unwrap();

    let (rolled_number, server_seed, _nonce) = gen_fake_bet(server_storage, client_seed);
    server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
    server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
    server_storage.server_seed_hash_next_roll = server_seed.clone();
    server_storage.previous_nonce = server_storage.current_nonce;
    server_storage.next_nonce = server_storage.current_nonce + 1;
    server_storage.current_nonce = server_storage.next_nonce;
    server_storage.previous_roll = server_storage.current_roll;
    server_storage.current_roll = server_storage.next_roll;
    server_storage.next_roll = rolled_number;

    let target = (10_000. * ((97.50 / multiplier) / 100.)) as u32;
    let result = (high && server_storage.current_roll > (10_000 - target))
        || (!high && server_storage.current_roll < target);

    BetSiteResult {
        success_code: "1".to_string(),
        result,
        rolled_number: server_storage.current_roll,
        user_balance: 0.,
        amount_won: if result {
            stake * (multiplier - 1.)
        } else {
            stake
        },
        server_seed_hash_next_roll: server_storage.server_seed_hash_next_roll.clone(),
        client_seed_previous_roll: client_seed.to_string(),
        nonce_next_roll: server_storage.current_nonce.to_string(),
        server_seed_previous_roll: server_storage.server_seed_previous_roll.to_string(),
        server_seed_hash_previous_roll: server_storage.server_seed_hash_previous_roll.clone(),
        previous_nonce: server_storage.previous_nonce.to_string(),
        jackpot_result: 0,
        jackpot_amount_won: 0.,
        bonus_account_balance_after_bet: 0.,
        bonus_acount_wager_remaining: 0.,
        max_amount_bonus_eligable: 0.,
        max_bet: 20.,
        account_balance_after_bet: 0.,
        account_balance_before_bet: 0.,
        bonus_account_balance_before_bet: 0.,
    }
}

pub fn duckdice_fake_bet(
    high: bool,
    client_seed: &str,
    stake: f32,
    multiplier: f32,
) -> BetMakeResponse {
    let server_storage: &mut FakeServerStorage = &mut SERVER_STORAGE.lock().unwrap();

    let (rolled_number, server_seed, _nonce) = gen_fake_bet(server_storage, client_seed);
    server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
    server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
    server_storage.server_seed_hash_next_roll = server_seed.clone();
    server_storage.previous_nonce = server_storage.current_nonce;
    server_storage.next_nonce = server_storage.current_nonce + 1;
    server_storage.current_nonce = server_storage.next_nonce;
    server_storage.previous_roll = server_storage.current_roll;
    server_storage.current_roll = server_storage.next_roll;
    server_storage.next_roll = rolled_number;

    let target = (10_000. * ((97.50 / multiplier) / 100.)) as u32;
    let result = (high && server_storage.current_roll > (10_000 - target))
        || (!high && server_storage.current_roll < target);

    BetMakeResponse {
        bet: Bet {
            previous_hash: server_storage.server_seed_hash_previous_roll.clone(),
            hash: server_storage.current_seed_hash.clone(),
            symbol: "UNKNOWN".to_string(),
            choice: if server_storage.current_roll > 5000 {
                "> 5000".to_string()
            } else {
                "< 5000".to_string()
            },
            result,
            number: server_storage.current_roll,
            chance: 100. * ((100. / multiplier) / 100.),
            payout: if result {
                stake * (multiplier - 1.)
            } else {
                -stake
            },
            bet_amount: stake,
            win_amount: if result {
                stake * (multiplier - 1.)
            } else {
                -stake
            },
            profit: if result {
                stake * (multiplier - 1.)
            } else {
                -stake
            },
            mined: 0.,
            nonce: server_storage.current_nonce,
            created: 0,
            game_mode: String::new(),
        },
        is_jackpot: false,
        jackpot_status: None,
        jackpot: None,
        user: User {
            hash: "".to_string(),
            level: 0,
            username: "".to_string(),
            bets: 0,
            nonce: 0,
            wins: 0,
            luck: 0.,
            balance: 0.,
            profit: 0.,
            volume: 0.,
            absolute_level: AbsoluteLevel {
                level: 0,
                xp: 0,
                xp_next: 0,
                xp_prev: 0,
            },
        },
    }
}

pub fn reset_server_seed() {
    let server_storage: &mut FakeServerStorage = &mut SERVER_STORAGE.lock().unwrap();

    server_storage.current_nonce = 0;
}
