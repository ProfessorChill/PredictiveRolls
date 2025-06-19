use burn::data::dataset::Dataset;
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use rand::Rng;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BetResultCsvRecord {
    pub result: bool,
    pub rolled_number: u32,
    pub next_number: u32,
    pub user_balance: f64,
    pub amount_won: f64,
    pub server_seed_hash_next_roll: String,
    pub client_seed: String,
    pub nonce_next_roll: u64,
    pub nonce: u64,
    pub server_seed_previous_roll: String,
    pub server_seed_hash_previous_roll: String,
    pub previous_nonce: u64,
    #[serde(skip)]
    pub duplicate_rolls: Vec<u32>,
}

/// Returns: (rolled_number, server_seed, nonce)
pub fn gen_fake_bet(
    server_storage: &mut FakeServerStorage,
    client_seed: &str,
    nonce: u64,
) -> (u32, String, String, u64) {
    let sys_random = SystemRandom::new();

    let mut server_seed = [0u8; 64];
    sys_random.fill(&mut server_seed).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(server_seed);
    let result = hasher.finalize();
    let server_seed_hash = hex::encode(result);

    let client_seed: String = rand::thread_rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(rand::thread_rng().gen_range(0..64))
        .map(char::from)
        .collect();

    let mut combined_seed = Vec::new();
    combined_seed.extend_from_slice(&server_seed);
    combined_seed.extend_from_slice(client_seed.as_bytes());
    combined_seed.extend_from_slice(&nonce.to_be_bytes());

    let key = hmac::Key::new(hmac::HMAC_SHA256, &server_seed);
    let tag = hmac::sign(&key, &combined_seed);

    let random_bytes = &tag.as_ref()[..4];
    let random_u32 = u32::from_le_bytes(random_bytes.try_into().unwrap());

    let number = random_u32 % 10_000;

    (
        number,
        server_seed_hash,
        client_seed,
        server_storage.current_nonce,
    )
}

pub fn free_bitcoin_fake_bet(
    high: bool,
    client_seed: &str,
    _stake: f32,
    multiplier: f32,
    nonce: u64,
) -> BetResultCsvRecord {
    let server_storage: &mut FakeServerStorage = &mut SERVER_STORAGE.lock().unwrap();

    let (rolled_number, server_seed, s_client_seed, nonce) =
        gen_fake_bet(server_storage, client_seed, nonce);
    server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
    server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
    server_storage.server_seed_hash_next_roll = server_seed.clone();
    server_storage.previous_nonce = nonce;
    server_storage.current_nonce = nonce;
    server_storage.next_nonce = nonce + 1;
    server_storage.previous_roll = server_storage.current_roll;
    server_storage.current_roll = server_storage.next_roll;
    server_storage.next_roll = rolled_number;

    let target = (10_000. * ((99.95 / multiplier) / 100.)) as u32;
    let result = (high && server_storage.current_roll > (10_000 - target))
        || (!high && server_storage.current_roll < target);

    let mut record = BetResultCsvRecord {
        result,
        rolled_number,
        next_number: 0,
        user_balance: 0.,
        amount_won: 0.,
        server_seed_hash_next_roll: server_storage.server_seed_hash_next_roll.clone(),
        client_seed: s_client_seed.clone(),
        nonce_next_roll: nonce + 1,
        nonce: nonce,
        server_seed_previous_roll: server_storage.server_seed_previous_roll.to_string(),
        server_seed_hash_previous_roll: server_storage.server_seed_hash_previous_roll.clone(),
        previous_nonce: server_storage.previous_nonce,
        duplicate_rolls: Vec::new(),
    };

    let (rolled_number, server_seed, _client_seed, nonce) =
        gen_fake_bet(server_storage, client_seed, nonce);
    server_storage.server_seed_hash_previous_roll = server_storage.current_seed_hash.clone();
    server_storage.current_seed_hash = server_storage.server_seed_hash_next_roll.clone();
    server_storage.server_seed_hash_next_roll = server_seed.clone();
    server_storage.previous_nonce = nonce;
    server_storage.current_nonce = nonce;
    server_storage.next_nonce = nonce + 1;
    server_storage.previous_roll = server_storage.current_roll;
    server_storage.current_roll = server_storage.next_roll;
    server_storage.next_roll = rolled_number;

    record.next_number = rolled_number;

    record
}

pub struct BetResultsDataset {
    len: usize,
}

impl BetResultsDataset {
    pub fn train() -> Result<Self, std::io::Error> {
        Ok(Self { len: 1_000_000 })
    }

    pub fn test() -> Result<Self, std::io::Error> {
        Ok(Self { len: 1_000 })
    }
}

impl Dataset<BetResultCsvRecord> for BetResultsDataset {
    fn get(&self, index: usize) -> Option<BetResultCsvRecord> {
        Some(free_bitcoin_fake_bet(
            true,
            "lYypIPVEgzvCflWF",
            1e-8,
            2.,
            index as u64,
        ))
    }

    fn len(&self) -> usize {
        self.len
    }
}
