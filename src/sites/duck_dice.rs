use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{
    AUTHORIZATION, SERVER, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use reqwest::redirect::Policy;
use reqwest::{
    header::{HeaderMap, CACHE_CONTROL, CONTENT_TYPE},
    Url,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::sites::{BetError, BetResult, Site};
use crate::strategies::Strategy;

const API_KEY: &str = "73e34c7c-a058-11ef-b604-3280a0c8331e";

#[derive(Debug)]
pub enum Currency {
    XRP,
    DECOY,
    USDT,
    BTC,
    LTC,
    TRX,
    DOGE,
    ETH,
    XLM,
    BCH,
    BNB,
    SHIB,
    USDC,
    ADA,
    DASH,
    SOL,
    ATOM,
    ETC,
    XMR,
    EOS,
    BTTC,
    POL,
    DOT,
    ZEC,
    RVN,
    LINK,
    DAI,
    TUSD,
    AVAX,
    NEAR,
    ZEN,
    AAVE,
    NOT,
    ENA,
    UNI,
    TON,
    TRUMP,
    FDUSD,
    WBTC,
    CAD,
}

impl Currency {
    pub fn get_min_bet(&self) -> f32 {
        match self {
            Self::XRP => 0.0005,
            Self::DECOY => 0.01,
            Self::USDT => 0.002,
            Self::BTC => 0.00000001,
            Self::LTC => 0.00002,
            Self::TRX => 0.006,
            Self::DOGE => 0.01,
            Self::ETH => 0.0000005,
            Self::XLM => 0.005,
            Self::BCH => 0.000005,
            Self::BNB => 0.000002,
            Self::SHIB => 100.,
            Self::USDC => 0.002,
            Self::ADA => 0.002,
            Self::DASH => 0.0001,
            Self::SOL => 0.000008,
            Self::ATOM => 0.0004,
            Self::ETC => 0.00012,
            Self::XMR => 0.000006,
            Self::EOS => 0.002,
            Self::BTTC => 2000.,
            Self::POL => 0.01,
            Self::DOT => 0.0005,
            Self::ZEC => 0.00004,
            Self::RVN => 0.01,
            Self::LINK => 0.0001,
            Self::DAI => 0.002,
            Self::TUSD => 0.002,
            Self::AVAX => 0.0001,
            Self::NEAR => 0.0008,
            Self::ZEN => 0.0002,
            Self::AAVE => 0.000006,
            Self::NOT => 0.8,
            Self::ENA => 0.006,
            Self::UNI => 0.0003,
            Self::TON => 0.0006,
            Self::TRUMP => 0.0002,
            Self::FDUSD => 0.002,
            Self::WBTC => 0.00000004,
            Self::CAD => 0.002,
        }
    }
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        match self {
            Self::XRP => "XRP".to_string(),
            Self::DECOY => "DECOY".to_string(),
            Self::USDT => "USDT".to_string(),
            Self::BTC => "BTC".to_string(),
            Self::LTC => "LTC".to_string(),
            Self::TRX => "TRX".to_string(),
            Self::DOGE => "DOGE".to_string(),
            Self::ETH => "ETH".to_string(),
            Self::XLM => "XLM".to_string(),
            Self::BCH => "BCH".to_string(),
            Self::BNB => "BNB".to_string(),
            Self::SHIB => "SHIB".to_string(),
            Self::USDC => "USDC".to_string(),
            Self::ADA => "ADA".to_string(),
            Self::DASH => "DASH".to_string(),
            Self::SOL => "SOL".to_string(),
            Self::ATOM => "ATOM".to_string(),
            Self::ETC => "ETC".to_string(),
            Self::XMR => "XMR".to_string(),
            Self::EOS => "EOS".to_string(),
            Self::BTTC => "BTTC".to_string(),
            Self::POL => "POL".to_string(),
            Self::DOT => "DOT".to_string(),
            Self::ZEC => "ZEC".to_string(),
            Self::RVN => "RVN".to_string(),
            Self::LINK => "LINK".to_string(),
            Self::DAI => "DAI".to_string(),
            Self::TUSD => "TUSD".to_string(),
            Self::AVAX => "AVAX".to_string(),
            Self::NEAR => "NEAR".to_string(),
            Self::ZEN => "ZEN".to_string(),
            Self::AAVE => "AAVE".to_string(),
            Self::NOT => "NOT".to_string(),
            Self::ENA => "ENA".to_string(),
            Self::UNI => "UNI".to_string(),
            Self::TON => "TON".to_string(),
            Self::TRUMP => "TRUMP".to_string(),
            Self::FDUSD => "FDUSD".to_string(),
            Self::WBTC => "WBTC".to_string(),
            Self::CAD => "CAD".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Jackpot {
    pub amount: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetMake {
    pub symbol: String,
    pub chance: f32,
    pub is_high: bool,
    pub amount: f32,
    pub user_wagering_bonus_hash: Option<String>,
    pub faucet: Option<bool>,
    pub tle_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetJson {
    pub hash: String,
    pub symbol: String,
    pub result: bool,
    pub is_high: bool,
    pub number: u32,
    pub threshold: u32,
    pub chance: f32,
    pub payout: f32,
    pub bet_amount: String,
    pub win_amount: String,
    pub profit: String,
    pub mined: Option<String>,
    pub nonce: u64,
    pub created: u64,
    pub game_mode: String,
}

#[derive(Clone, Debug)]
pub struct Bet {
    pub previous_hash: String,
    pub hash: String,
    pub symbol: String,
    pub result: bool,
    pub is_high: bool,
    pub number: u32,
    pub threshold: u32,
    pub chance: f32,
    pub payout: f32,
    pub bet_amount: f32,
    pub win_amount: f32,
    pub profit: f32,
    pub mined: f32,
    pub nonce: u64,
    pub created: u64,
    pub game_mode: String,
}

impl From<BetJson> for Bet {
    fn from(value: BetJson) -> Self {
        Bet {
            previous_hash: value.hash.clone(),
            hash: value.hash.clone(),
            symbol: value.symbol.clone(),
            result: value.result,
            is_high: value.is_high,
            number: value.number,
            threshold: value.threshold,
            chance: value.chance,
            payout: value.payout,
            bet_amount: value.bet_amount.parse::<f32>().unwrap_or(0.),
            win_amount: value.win_amount.parse::<f32>().unwrap_or(0.),
            profit: value.profit.parse::<f32>().unwrap_or(0.),
            mined: value
                .mined
                .unwrap_or("0.0".to_string())
                .parse::<f32>()
                .unwrap_or(0.),
            nonce: value.nonce,
            created: value.created,
            game_mode: value.game_mode.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsoluteLevel {
    pub level: u32,
    pub xp: u64,
    pub xp_next: u64,
    pub xp_prev: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserJson {
    pub hash: String,
    pub level: u32,
    pub username: String,
    pub bets: u64,
    pub nonce: u64,
    pub wins: u64,
    pub luck: f32,
    pub balance: String,
    pub profit: String,
    pub volume: String,
    pub absolute_level: AbsoluteLevel,
}

#[derive(Clone, Debug)]
pub struct User {
    pub hash: String,
    pub level: u32,
    pub username: String,
    pub bets: u64,
    pub nonce: u64,
    pub wins: u64,
    pub luck: f32,
    pub balance: f32,
    pub profit: f32,
    pub volume: f32,
    pub absolute_level: AbsoluteLevel,
}

impl From<UserJson> for User {
    fn from(value: UserJson) -> Self {
        User {
            hash: value.hash.clone(),
            level: value.level,
            username: value.username.clone(),
            bets: value.bets,
            nonce: value.nonce,
            wins: value.wins,
            luck: value.luck,
            balance: value.balance.parse::<f32>().unwrap_or(0.),
            profit: value.profit.parse::<f32>().unwrap_or(0.),
            volume: value.volume.parse::<f32>().unwrap_or(0.),
            absolute_level: value.absolute_level.clone(),
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetMakeResponseJson {
    pub bet: BetJson,
    pub is_jackpot: bool,
    pub jackpot_status: Option<bool>,
    pub jackpot: Option<Jackpot>,
    pub user: UserJson,
}

#[derive(Clone, Debug)]
pub struct BetMakeResponse {
    pub bet: Bet,
    pub is_jackpot: bool,
    pub jackpot_status: Option<bool>,
    pub jackpot: Option<Jackpot>,
    pub user: User,
}

impl From<BetMakeResponseJson> for BetMakeResponse {
    fn from(value: BetMakeResponseJson) -> Self {
        BetMakeResponse {
            bet: value.bet.into(),
            is_jackpot: value.is_jackpot,
            jackpot_status: value.jackpot_status,
            jackpot: value.jackpot.clone(),
            user: value.user.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastDepositJson {
    pub created_at: u64,
    pub currency: String,
    pub amount: String,
}

#[derive(Debug)]
pub struct LastDeposit {
    pub created_at: u64,
    pub currency: String,
    pub amount: f32,
}

impl From<LastDepositJson> for LastDeposit {
    fn from(value: LastDepositJson) -> Self {
        Self {
            created_at: value.created_at,
            currency: value.currency.clone(),
            amount: value.amount.parse::<f32>().unwrap_or(0.),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WageredJson {
    pub currency: String,
    pub amount: String,
}

#[derive(Debug)]
pub struct Wagered {
    pub currency: String,
    pub amount: f32,
}

impl From<&WageredJson> for Wagered {
    fn from(value: &WageredJson) -> Self {
        Self {
            currency: value.currency.clone(),
            amount: value.amount.parse::<f32>().unwrap_or(0.),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Balance {
    pub currency: String,
    pub main: Option<String>,
    pub faucet: Option<String>,
    pub affiliate: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WageringBonusJson {
    pub name: String,
    #[serde(rename(deserialize = "type"))]
    pub data_type: String,
    pub hash: String,
    pub status: String,
    pub symbol: String,
    pub margin: String,
}

#[derive(Debug)]
pub struct WageringBonus {
    pub name: String,
    pub data_type: String,
    pub hash: String,
    pub status: String,
    pub symbol: String,
    pub margin: f32,
}

impl From<&WageringBonusJson> for WageringBonus {
    fn from(value: &WageringBonusJson) -> Self {
        Self {
            name: value.name.clone(),
            data_type: value.data_type.clone(),
            hash: value.hash.clone(),
            status: value.status.clone(),
            symbol: value.symbol.clone(),
            margin: value.margin.parse::<f32>().unwrap_or(0.),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TLE {
    pub hash: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoJson {
    pub hash: String,
    pub username: String,
    pub created_at: u64,
    pub level: u32,
    pub campaign: Option<String>,
    pub affiliate: Option<String>,
    pub last_deposit: Option<LastDepositJson>,
    pub wagered: Vec<WageredJson>,
    pub balances: Vec<Balance>,
    pub wagering_bonuses: Vec<WageringBonusJson>,
    pub tle: Vec<TLE>,
}

#[derive(Debug)]
pub struct UserInfo {
    pub hash: String,
    pub username: String,
    pub created_at: u64,
    pub level: u32,
    pub campaign: Option<String>,
    pub affiliate: Option<String>,
    pub last_deposit: Option<LastDeposit>,
    pub wagered: Vec<Wagered>,
    pub balances: Vec<Balance>,
    pub wagering_bonuses: Vec<WageringBonus>,
    pub tle: Vec<TLE>,
}

impl From<UserInfoJson> for UserInfo {
    fn from(value: UserInfoJson) -> Self {
        Self {
            hash: value.hash.clone(),
            username: value.username.clone(),
            created_at: value.created_at,
            level: value.level,
            campaign: value.campaign.clone(),
            affiliate: value.affiliate.clone(),
            last_deposit: if let Some(last_deposit) = value.last_deposit {
                Some(last_deposit.into())
            } else {
                None
            },
            wagered: value
                .wagered
                .iter()
                .map(|wager| wager.into())
                .collect::<Vec<Wagered>>(),
            balances: value.balances.clone(),
            wagering_bonuses: value
                .wagering_bonuses
                .iter()
                .map(|wagering_bonus| wagering_bonus.into())
                .collect::<Vec<WageringBonus>>(),
            tle: value.tle.clone(),
        }
    }
}

pub struct DuckDiceIo {
    pub client: reqwest::Client,
    pub rolls: u64,
    pub strategy: Box<dyn Strategy>,
    history: Vec<BetResult>,
    history_size: usize,
    current_bet: f32,
    chance: f32,
    balance: f32,
    profit: f32,
    previous_hash: String,
    default_headers: HeaderMap,
    currency: Currency,
    faucet: bool,
    initialized_hash: bool,
}

impl Default for DuckDiceIo {
    fn default() -> Self {
        let currency = Currency::BTC;

        Self {
            client: reqwest::Client::new(),
            rolls: 0,
            strategy: Box::new(
                // crate::strategies::blaks_runner::BlaksRunner5_0::default()
                crate::strategies::my_strategy::MyStrat::default()
                    // crate::strategies::none::NoStrat::default()
                    .with_balance(0.07747934)
                    .with_min_bet(currency.get_min_bet())
                    .with_initial_bet(currency.get_min_bet()),
            ),
            history: Vec::new(),
            history_size: 10,
            current_bet: currency.get_min_bet(),
            chance: 2.,
            balance: 0.07747934,
            profit: 0.,
            previous_hash: String::new(),
            default_headers: HeaderMap::new(),
            currency,
            faucet: false,
            initialized_hash: false,
        }
    }
}

#[async_trait]
impl Site for DuckDiceIo {
    async fn login(&mut self) -> Result<(), BetError> {
        self.default_headers
            .insert(CONTENT_TYPE, "application/json".parse().unwrap());
        self.default_headers
            .insert(CACHE_CONTROL, "no-cache, private".parse().unwrap());
        self.default_headers.insert(
            STRICT_TRANSPORT_SECURITY,
            "max-age=15552000; includeSubDomains".parse().unwrap(),
        );
        self.default_headers
            .insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
        self.default_headers
            .insert(X_FRAME_OPTIONS, "sameorigin".parse().unwrap());
        self.default_headers
            .insert(SERVER, "cloudflare".parse().unwrap());

        self.client = reqwest::Client::builder()
            .default_headers(self.default_headers.clone())
            .user_agent("DuckDiceBot/1.0.0")
            .redirect(Policy::limited(200))
            .build()?;

        let user_info_url = Url::parse(&format!(
            "https://duckdice.io/api/bot/user-info?api_key={API_KEY}",
        ))
        .expect("Failed to parse user_info URL");

        let res: serde_json::Value = self.client.get(user_info_url).send().await?.json().await?;
        let res: UserInfoJson = serde_json::from_value(res).unwrap();
        let res: UserInfo = res.into();

        println!("{}", res.hash);

        for balance in &res.balances {
            if balance.currency == self.currency.to_string().as_str() {
                if let Some(main) = &if self.faucet {
                    balance.faucet.clone()
                } else {
                    balance.main.clone()
                } {
                    let val = main.parse::<f32>().unwrap_or(0.);
                    self.strategy.set_balance(val);
                    self.balance = val;
                }
            }
        }

        Ok(())
    }

    async fn do_bet(&mut self, prediction: f32, confidence: f32) -> Result<BetResult, BetError> {
        self.rolls += 1;
        let next_bet_data = self.strategy.get_next_bet(prediction, confidence);
        self.current_bet = next_bet_data.0;
        self.chance = next_bet_data.2;
        let high = next_bet_data.3;

        if self.history.len() < self.history_size {
            self.current_bet = self.currency.get_min_bet();
            self.chance = 50.;
        }

        self.current_bet = self.current_bet.max(self.currency.get_min_bet());

        let bet_url =
            Url::parse_with_params("https://duckdice.io/api/play", &[("api_key", API_KEY)])
                .expect("Failed to parse do_bet URL");

        let res = self
            .client
            .post(bet_url)
            .json(&BetMake {
                symbol: self.currency.to_string(),
                chance: format!("{:.2}", self.chance).parse::<f32>().unwrap_or(0.),
                is_high: high,
                amount: format!("{:.8}", self.current_bet)
                    .parse::<f32>()
                    .unwrap_or(0.),
                user_wagering_bonus_hash: None,
                faucet: if self.faucet { Some(true) } else { None },
                tle_hash: None,
            })
            .send()
            .await;

        match res {
            Ok(res) => {
                if res.status() == 403 {
                    self.rolls -= 1;
                    self.default_headers.remove("cf-ray");
                    self.default_headers
                        .insert("cf-ray", res.headers()["cf-ray"].clone());

                    self.client = reqwest::Client::builder()
                        .default_headers(self.default_headers.clone())
                        .user_agent("DuckDiceBot/1.0.0")
                        .redirect(Policy::limited(200))
                        .build()?;
                    return Err(BetError::EmptyReply);
                }
                let res: serde_json::Value = res.json().await?;

                let res: BetMakeResponseJson = serde_json::from_value(res).unwrap();
                let mut res: BetMakeResponse = res.into();

                if !self.initialized_hash {
                    let res_bet_data = self
                        .client
                        .get(&format!("https://duckdice.io/api/bet/{}", res.bet.hash))
                        .send()
                        .await?;
                    let res_bet_data: serde_json::Value = res_bet_data.json().await?;

                    res.bet.previous_hash = self.previous_hash.clone();
                    self.previous_hash = res_bet_data["seed"]["serverSeedHash"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    res.bet.hash = self.previous_hash.clone();
                    self.initialized_hash = true;
                }

                if res.bet.nonce >= 1000 {
                    let randomize_url = Url::parse_with_params(
                        "https://duckdice.io/api/randomize",
                        &[("api_key", API_KEY)],
                    )
                    .expect("Failed to parse randomize URL");

                    let res_randomize = self
                        .client
                        .post(randomize_url)
                        .json(&json!({
                            "clientSeed": "BeO2jZRd4nidPz4U40e2G7hT22s9GA",
                        }))
                        .send()
                        .await?;

                    if let Some(retry_after) = res_randomize.headers().get("retry-after") {
                        let _ = tokio::time::sleep(Duration::from_secs(
                            retry_after.to_str().unwrap().parse::<u64>().unwrap(),
                        ))
                        .await;
                    }
                    self.initialized_hash = false;
                }

                self.default_headers.remove(AUTHORIZATION);
                self.client = reqwest::Client::builder()
                    .default_headers(self.default_headers.clone())
                    .user_agent("DuckDiceBot/1.0.0")
                    .redirect(Policy::limited(200))
                    .build()?;

                self.history.push(res.clone().into());
                if self.history.len() > self.history_size {
                    self.history = self.history[1..].to_vec();
                }

                Ok(res.into())
            }
            Err(_) => Err(BetError::EmptyReply),
        }
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        self.strategy.on_win(bet_result);
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        let mut bet_result = bet_result.clone();
        bet_result.win_amount = -bet_result.win_amount;
        self.strategy.on_lose(&bet_result);
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
        1. / (self.chance / 100.)
    }

    fn get_history_size(&self) -> usize {
        self.history_size
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }

    fn get_balance(&self) -> f32 {
        self.balance
    }
}
