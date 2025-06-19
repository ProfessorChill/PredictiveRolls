use lazy_static::lazy_static;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

use crate::sites::free_bitco_in::BetSiteResult;

lazy_static! {
    pub static ref SERVER_STORAGE: Mutex<FakeServerStorage> =
        Mutex::new(FakeServerStorage::default());
}

#[derive(Debug, Default)]
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
}

/// Returns: (rolled_number, server_seed, nonce)
pub fn gen_fake_bet(
    server_storage: &mut FakeServerStorage,
    client_seed: &str,
) -> (u32, String, u64) {
    let sys_random = SystemRandom::new();

    let mut server_seed = [0u8; 64];
    sys_random.fill(&mut server_seed).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(server_seed);
    let result = hasher.finalize();
    let server_seed = hex::encode(result);

    let mut combined_seed = Vec::new();
    combined_seed.extend_from_slice(client_seed.as_bytes());
    combined_seed.extend_from_slice(server_seed.as_bytes());
    combined_seed.extend_from_slice(&server_storage.current_nonce.to_be_bytes());

    let key = hmac::Key::new(hmac::HMAC_SHA256, server_seed.as_bytes());
    let tag = hmac::sign(&key, &combined_seed);

    let random_bytes = &tag.as_ref()[..4];
    let random_u32 = u32::from_le_bytes(random_bytes.try_into().unwrap());

    let number = random_u32 % 10_000;

    (number, server_seed, server_storage.current_nonce)
}

pub fn free_bitcoin_fake_bet(
    high: bool,
    client_seed: &str,
    stake: f32,
    multiplier: f32,
) -> BetSiteResult {
    let server_storage: &mut FakeServerStorage = &mut SERVER_STORAGE.lock().unwrap();

    for _ in 0..4 {
        let (rolled_number, server_seed, _nonce) = gen_fake_bet(server_storage, client_seed);
        server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
        server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
        server_storage.server_seed_hash_next_roll = server_seed.clone();
        server_storage.previous_nonce = server_storage.current_nonce;
        server_storage.current_nonce = server_storage.next_nonce;
        server_storage.next_nonce += 1;
        server_storage.previous_roll = server_storage.current_roll;
        server_storage.current_roll = server_storage.next_roll;
        server_storage.next_roll = rolled_number;
    }

    let (rolled_number, server_seed, _nonce) = gen_fake_bet(server_storage, client_seed);
    server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
    server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
    server_storage.server_seed_hash_next_roll = server_seed.clone();
    server_storage.previous_nonce = server_storage.current_nonce;
    server_storage.current_nonce = server_storage.next_nonce;
    server_storage.next_nonce += 1;
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
