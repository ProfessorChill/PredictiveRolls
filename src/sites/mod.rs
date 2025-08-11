use async_trait::async_trait;

pub mod crypto_games;
pub mod duck_dice;
pub mod fake_test;
pub mod free_bitco_in;
pub mod windice;

#[derive(Debug)]
pub enum BetError {
    EmptyReply,
    Failed,
    LoginFailed,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for BetError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

#[derive(Clone, Debug)]
pub struct BetResult {
    pub hash_previous_roll: String,
    pub hash_next_roll: String,
    pub client_seed: String,
    pub nonce: u32,
    pub symbol: String,
    pub result: bool,
    pub is_high: bool,
    pub number: u32,
    pub threshold: u32,
    pub chance: f32,
    pub payout: f32,
    pub bet_amount: f32,
    pub win_amount: f32,
}

impl From<free_bitco_in::BetSiteResult> for BetResult {
    fn from(value: free_bitco_in::BetSiteResult) -> Self {
        Self {
            hash_previous_roll: value.server_seed_hash_previous_roll,
            hash_next_roll: value.server_seed_hash_next_roll,
            client_seed: value.client_seed_previous_roll.clone(),
            nonce: value.nonce_next_roll.clone().parse::<u32>().unwrap_or(0),
            symbol: "BTC".to_string(),
            result: value.result,
            is_high: value.rolled_number > 5000 && value.result,
            number: value.rolled_number,
            // We can't get this number from freebitco.in without external data so we won't include
            // that data.
            threshold: 0,
            // Likewise with chance.
            chance: 0.,
            // And for this as well.
            payout: 0.,
            // You guessed it.
            bet_amount: 0.,
            win_amount: value.amount_won,
        }
    }
}

impl From<duck_dice::BetMakeResponse> for BetResult {
    fn from(value: duck_dice::BetMakeResponse) -> Self {
        Self {
            hash_previous_roll: value.bet.previous_hash.clone(),
            hash_next_roll: value.bet.hash.clone(),
            client_seed: String::new(),
            nonce: value.bet.nonce as u32,
            symbol: value.bet.symbol,
            result: value.bet.result,
            is_high: value.bet.choice.chars().next().unwrap_or(' ') == '>',
            number: value.bet.number,
            threshold: 0,
            chance: value.bet.chance,
            payout: value.bet.payout,
            bet_amount: value.bet.bet_amount,
            win_amount: value.bet.profit,
        }
    }
}

impl From<crypto_games::BetSiteResult> for BetResult {
    fn from(value: crypto_games::BetSiteResult) -> Self {
        Self {
            hash_previous_roll: value.server_seed.clone(),
            hash_next_roll: value.next_server_seed_hash.clone(),
            client_seed: "BeO2jZRd4nidPz4U40e2G7hT22s9GA".to_string(),
            nonce: 0,
            symbol: "SOL".to_string(),
            result: value.profit > 0.,
            is_high: value.roll as u32 > 5000 && value.profit > 0.,
            number: value.roll as u32,
            threshold: 0,
            chance: 0.,
            payout: value.payout as f32,
            bet_amount: 0.,
            win_amount: value.profit as f32,
        }
    }
}

#[async_trait]
pub trait Site {
    async fn login(&mut self) -> Result<(), BetError>;
    async fn do_bet(&mut self, prediction: f32, confidence: f32) -> Result<BetResult, BetError>;
    fn on_win(&mut self, bet_result: &BetResult);
    fn on_lose(&mut self, bet_result: &BetResult);
    fn get_history(&self) -> Vec<BetResult>;
    fn get_history_size(&self) -> usize;
    fn get_rolls(&self) -> u64;
    fn get_current_bet(&self) -> f32;
    fn get_current_multiplier(&self) -> f32;
    fn get_profit(&self) -> f32;
    fn get_balance(&self) -> f32;
}

pub trait SiteCurrency {
    fn get_min_bet(&self) -> f32;
}

pub enum Sites {
    DuckDiceIo,
    CryptoGames,
    FreeBitcoIn,
}
