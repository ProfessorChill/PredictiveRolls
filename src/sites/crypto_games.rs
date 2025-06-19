use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    sites::{BetError, BetResult, Site},
    strategies::Strategy,
};

#[derive(Debug)]
pub enum Currency {
    BTC,
    ETH,
    USDT,
    SOL,
    XRP,
    BNB,
    USDC,
    DOGE,
    SHIB,
    LTC,
    BCH,
    PEPE,
    ETC,
    POL,
    GAS,
    PLAY,
}

impl Currency {
    pub fn get_min_bet(&self) -> f32 {
        match self {
            Self::BTC => 0.00000002,
            Self::ETH => 0.0000006,
            Self::USDT => 0.002,
            Self::XRP => 0.004,
            Self::BNB => 0.000004,
            Self::SOL => 0.00002,
            Self::USDC => 0.002,
            Self::DOGE => 0.04,
            Self::SHIB => 200.,
            Self::LTC => 0.00002,
            Self::BCH => 0.000004,
            Self::PEPE => 600.,
            Self::ETC => 0.00006,
            Self::POL => 0.01,
            Self::GAS => 0.0004,
            Self::PLAY => 20.,
        }
    }
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        match self {
            Self::BTC => "BTC".to_string(),
            Self::ETH => "ETH".to_string(),
            Self::USDT => "USDT".to_string(),
            Self::XRP => "XRP".to_string(),
            Self::BNB => "BNB".to_string(),
            Self::SOL => "SOL".to_string(),
            Self::USDC => "USDC".to_string(),
            Self::DOGE => "DOGE".to_string(),
            Self::SHIB => "SHIB".to_string(),
            Self::LTC => "LTC".to_string(),
            Self::BCH => "BCH".to_string(),
            Self::PEPE => "PEPE".to_string(),
            Self::ETC => "ETC".to_string(),
            Self::POL => "POL".to_string(),
            Self::GAS => "GAS".to_string(),
            Self::PLAY => "PLAY".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Coin {
    #[serde(rename(deserialize = "Coin"))]
    pub coin: String,
    #[serde(rename(deserialize = "MinBet"))]
    pub min_bet: f64,
    #[serde(rename(deserialize = "MaxWin"))]
    pub max_win: f64,
    #[serde(rename(deserialize = "MinPayout"))]
    pub min_payout: f64,
    #[serde(rename(deserialize = "MaxPayout"))]
    pub max_payout: f64,
    #[serde(rename(deserialize = "Edge"))]
    pub edge: f64,
}

#[derive(Debug, Serialize)]
pub struct Bet {
    #[serde(rename(serialize = "Bet"))]
    pub bet: f64,
    #[serde(rename(serialize = "Payout"))]
    pub payout: f64,
    #[serde(rename(serialize = "UnderOver"))]
    pub under_over: bool,
    #[serde(rename(serialize = "ClientSeed"))]
    pub client_seed: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BetSiteResult {
    #[serde(rename(deserialize = "BetId"))]
    pub bet_id: u64,
    #[serde(rename(deserialize = "Roll"))]
    pub roll: f64,
    #[serde(rename(deserialize = "Target"))]
    pub target: String,
    #[serde(rename(deserialize = "Profit"))]
    pub profit: f64,
    #[serde(rename(deserialize = "Payout"))]
    pub payout: f64,
    #[serde(rename(deserialize = "ServerSeed"))]
    pub server_seed: String,
    #[serde(rename(deserialize = "NextServerSeedHash"))]
    pub next_server_seed_hash: String,
    #[serde(rename(deserialize = "Balance"))]
    pub balance: f64,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    #[serde(rename(deserialize = "Balance"))]
    pub balance: f64,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub balance: f32,
}

impl Default for UserStats {
    fn default() -> Self {
        Self { balance: 0. }
    }
}

pub struct CryptoGames {
    pub rolls: u64,
    pub client_seed: String,
    pub current_bet: f32,
    pub multiplier: f32,
    pub user_stats: UserStats,
    pub profit: f32,
    pub prediction: u32,
    pub strategy: Box<dyn Strategy>,
    client: reqwest::Client,
    key: String,
    history: Vec<BetResult>,
    history_size: usize,
    use_site_balance: bool,
    currency: Currency,
}

impl Default for CryptoGames {
    fn default() -> Self {
        let currency = Currency::PLAY;

        Self {
            rolls: 0,
            client_seed: "BeO2jZRd4nidPz4U40e2G7hT22s9GA".to_string(),
            current_bet: currency.get_min_bet(),
            multiplier: 2.,
            user_stats: UserStats::default(),
            profit: 0.,
            prediction: 0,
            strategy: Box::new(
                // crate::strategies::blaks_runner::BlaksRunner5_0::default()
                crate::strategies::my_strategy::MyStrat::default()
                    // crate::strategies::none::NoStrat::default()
                    .with_balance(0.00037203)
                    .with_min_bet(currency.get_min_bet())
                    .with_initial_bet(currency.get_min_bet()),
            ),
            client: reqwest::Client::new(),
            key: "".to_string(),
            history: Vec::new(),
            history_size: 10,
            use_site_balance: true,
            currency,
        }
    }
}

#[async_trait]
impl Site for CryptoGames {
    async fn login(&mut self) -> Result<(), BetError> {
        let balance: Balance = self
            .client
            .get(format!(
                "https://api.crypto.games/v1/balance/{}/{}",
                self.currency.to_string(),
                self.key
            ))
            .send()
            .await?
            .json()
            .await?;
        if self.use_site_balance {
            self.user_stats.balance = balance.balance as f32;
            self.strategy.set_balance(self.user_stats.balance);
        }

        Ok(())
    }

    async fn do_bet(&mut self, prediction: f32, confidence: f32) -> Result<BetResult, BetError> {
        self.rolls += 1;
        let next_bet_data = self.strategy.get_next_bet(prediction, confidence);
        self.current_bet = next_bet_data.0;
        self.multiplier = next_bet_data.1;
        let high = next_bet_data.3;

        if self.history.len() < self.history_size {
            self.current_bet = self.currency.get_min_bet();
            self.multiplier = 2.;
        }

        self.multiplier = self.multiplier.clamp(1.02, 9900.);
        self.current_bet = self.current_bet.max(self.currency.get_min_bet());

        let res: serde_json::Value = self
            .client
            .post(format!(
                "https://api.crypto.games/v1/placebet/{}/{}",
                self.currency.to_string(),
                self.key
            ))
            .json(&Bet {
                bet: self.current_bet as f64,
                payout: self.multiplier as f64,
                under_over: high,
                client_seed: self.client_seed.clone(),
            })
            .send()
            .await?
            .json()
            .await?;

        let mut res: BetSiteResult = serde_json::from_value(res).unwrap();
        res.roll *= 100.;

        self.history.push(res.clone().into());
        if self.history.len() > self.history_size {
            self.history = self.history[1..].to_vec();
        }

        if self.current_bet > self.strategy.get_balance() {
            panic!("Not enough money!");
        }

        Ok(res.into())
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.user_stats.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;

        if self.history.len() >= self.history_size {
            self.strategy.on_win(bet_result);
        }
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        let mut bet_result = bet_result.clone();
        bet_result.win_amount = -bet_result.win_amount;

        self.user_stats.balance -= bet_result.win_amount;
        self.profit -= bet_result.win_amount;

        if self.history.len() >= self.history_size {
            self.strategy.on_lose(&bet_result);
        }
    }

    fn get_history(&self) -> Vec<BetResult> {
        self.history.clone()
    }

    fn get_rolls(&self) -> u64 {
        self.rolls
    }

    fn get_current_bet(&self) -> f32 {
        self.current_bet
    }

    fn get_current_multiplier(&self) -> f32 {
        self.multiplier
    }

    fn get_history_size(&self) -> usize {
        self.history_size
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }

    fn get_balance(&self) -> f32 {
        self.user_stats.balance
    }
}
