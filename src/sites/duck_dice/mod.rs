use std::time::Duration;

use async_trait::async_trait;
use rand::Rng;
use reqwest::header::{
    AUTHORIZATION, SERVER, STRICT_TRANSPORT_SECURITY, USER_AGENT, X_CONTENT_TYPE_OPTIONS,
    X_FRAME_OPTIONS,
};
use reqwest::redirect::Policy;
use reqwest::{
    header::{HeaderMap, CACHE_CONTROL, CONTENT_TYPE},
    Url,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{SiteConfig, TomlStrategies};
use crate::currency::Currency;
use crate::sites::fake_test::{duckdice_fake_bet, reset_server_seed};
use crate::sites::{BetError, BetResult, Site, Sites};
use crate::strategies::Strategy;

const API_KEY: &str = "";

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
    pub choice: String,
    pub result: bool,
    pub number: u32,
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
    pub choice: String,
    pub result: bool,
    pub number: u32,
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
            choice: value.choice.clone(),
            result: value.result,
            number: value.number,
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
            last_deposit: value.last_deposit.map(|last_deposit| last_deposit.into()),
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
    offline_balance: f32,
    initial_balance: f32,
    site_balance: f32,
    profit: f32,
    previous_hash: String,
    client_seed: String,
    default_headers: HeaderMap,
    currency: Currency,
    faucet: bool,
    initialized_hash: bool,
    wins: u32,
    losses: u32,
    seed_profit: f32,
    api_key: String,
    use_site_balance: bool,
    balance_modifier: f32,
    use_fake_betting: bool,
    tle_hash: Option<String>,
}

impl Default for DuckDiceIo {
    fn default() -> Self {
        let currency = Currency::ETH;

        Self {
            client: reqwest::Client::new(),
            rolls: 0,
            strategy: Box::new(
                // crate::strategies::blaks_runner::BlaksRunner5_0::default()
                // crate::strategies::my_strategy::MyStrat::default()
                crate::strategies::none::NoStrat::default()
                    .with_balance(4.)
                    .with_min_bet(currency.get_min_bet(Sites::DuckDiceIo))
                    .with_initial_bet(currency.get_min_bet(Sites::DuckDiceIo)),
            ),
            history: Vec::new(),
            history_size: 10,
            current_bet: currency.get_min_bet(Sites::DuckDiceIo),
            chance: 2.,
            balance: 3.,
            offline_balance: 10.,
            initial_balance: 0.,
            site_balance: 0.,
            profit: 0.,
            previous_hash: String::new(),
            client_seed: String::new(),
            default_headers: HeaderMap::new(),
            currency,
            faucet: true,
            initialized_hash: false,
            wins: 0,
            losses: 0,
            seed_profit: 0.,
            api_key: String::new(),
            use_site_balance: true,
            balance_modifier: 1.,
            use_fake_betting: false,
            tle_hash: None,
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
        self.default_headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".parse().unwrap());

        self.client = reqwest::Client::builder()
            .default_headers(self.default_headers.clone())
            .user_agent("DuckDiceBot/1.0.0")
            .redirect(Policy::limited(200))
            .build()?;

        if self.use_site_balance {
            let user_info_url = Url::parse(&format!(
                "https://duckdice.io/api/bot/user-info?api_key={API_KEY}",
            ))
            .expect("Failed to parse user_info URL");
            let res: serde_json::Value =
                self.client.get(user_info_url).send().await?.json().await?;
            let res: UserInfoJson = serde_json::from_value(res).unwrap();
            let res: UserInfo = res.into();

            for balance in &res.balances {
                if balance.currency == self.currency.to_string().as_str() {
                    if let Some(main) = &if self.faucet {
                        balance.faucet.clone()
                    } else {
                        balance.main.clone()
                    } {
                        let val = main.parse::<f32>().unwrap_or(0.);
                        self.strategy.set_balance(val * self.balance_modifier);
                        self.site_balance = val;
                        self.balance = val * self.balance_modifier;
                        self.initial_balance = val * self.balance_modifier;
                    }
                }
            }
        } else {
            self.strategy
                .set_balance(self.offline_balance * self.balance_modifier);
            self.site_balance = self.offline_balance;
            self.balance = self.offline_balance * self.balance_modifier;
            self.initial_balance = self.offline_balance * self.balance_modifier;
        }

        Ok(())
    }

    async fn do_bet(&mut self, prediction: f32, confidence: f32) -> Result<BetResult, BetError> {
        if self.balance >= self.initial_balance * 10. {
            if self.use_site_balance {
                println!("[WIN] Resetting {:0>.8}", self.site_balance);
                self.balance = self.site_balance * self.balance_modifier;
                self.initial_balance = self.site_balance * self.balance_modifier;
            } else {
                println!("[WIN] Resetting {:0>.8}", self.offline_balance);
                self.balance = self.offline_balance * self.balance_modifier;
                self.initial_balance = self.offline_balance * self.balance_modifier;
            }
            self.strategy.reset();

            if self.profit > 0. && self.use_site_balance {
                let user_info_url = Url::parse(&format!(
                    "https://duckdice.io/api/bot/user-info?api_key={API_KEY}",
                ))
                .expect("Failed to parse user_info URL");
                let res: serde_json::Value =
                    self.client.get(user_info_url).send().await?.json().await?;
                let res: UserInfoJson = serde_json::from_value(res).unwrap();
                let res: UserInfo = res.into();

                for balance in &res.balances {
                    if balance.currency == self.currency.to_string().as_str() {
                        if let Some(main) = &if self.faucet {
                            balance.faucet.clone()
                        } else {
                            balance.main.clone()
                        } {
                            let val = main.parse::<f32>().unwrap_or(0.);
                            self.strategy.set_balance(val * self.balance_modifier);
                            self.site_balance = val;
                            self.balance = val * self.balance_modifier;
                            self.initial_balance = val * self.balance_modifier;
                            self.strategy.reset();
                            self.profit = 0.;
                        }
                    }
                }
            } else if self.profit > 0. && !self.use_site_balance {
                self.strategy
                    .set_balance(self.offline_balance * self.balance_modifier);
                self.balance = self.offline_balance * self.balance_modifier;
                self.initial_balance = self.offline_balance * self.balance_modifier;
                self.strategy.reset();
                self.profit = 0.;
            }
        }

        if self.balance - self.current_bet <= 0. && self.use_site_balance {
            println!("[FAIL] Resetting {:0>.8}", self.site_balance);
            self.balance = self.site_balance * self.balance_modifier;
            self.initial_balance = self.site_balance * self.balance_modifier;
            self.wins = 0;
            self.losses = 0;
            self.seed_profit = 0.;
            self.strategy.reset();
            let randomize_url = Url::parse_with_params(
                "https://duckdice.io/api/randomize",
                &[("api_key", API_KEY)],
            )
            .expect("Failed to parse randomize URL");

            self.client_seed = rand::rng()
                .sample_iter(rand::distr::Alphabetic)
                .take(30)
                .map(char::from)
                .collect();
            let res_randomize = self
                .client
                .post(randomize_url)
                .json(&json!({
                    "clientSeed": self.client_seed.clone(),
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
            let user_info_url = Url::parse(&format!(
                "https://duckdice.io/api/bot/user-info?api_key={API_KEY}",
            ))
            .expect("Failed to parse user_info URL");
            let res: serde_json::Value =
                self.client.get(user_info_url).send().await?.json().await?;
            let res: UserInfoJson = serde_json::from_value(res).unwrap();
            let res: UserInfo = res.into();

            for balance in &res.balances {
                if balance.currency == self.currency.to_string().as_str() {
                    if let Some(main) = &if self.faucet {
                        balance.faucet.clone()
                    } else {
                        balance.main.clone()
                    } {
                        let _val = main.parse::<f32>().unwrap_or(0.);
                        self.strategy
                            .set_balance(self.site_balance * self.balance_modifier);
                        self.balance = self.site_balance * self.balance_modifier;
                        self.initial_balance = self.site_balance * self.balance_modifier;
                        self.strategy.reset();
                    }
                }
            }
        } else if self.balance - self.current_bet <= 0. && !self.use_site_balance {
            println!("[FAIL] Resetting {:0>.8}", self.offline_balance);
            reset_server_seed();
            self.balance = self.offline_balance * self.balance_modifier;
            self.initial_balance = self.offline_balance * self.balance_modifier;
            self.wins = 0;
            self.losses = 0;
            self.seed_profit = 0.;
            self.strategy.reset();
        }

        self.rolls += 1;
        let next_bet_data = self.strategy.get_next_bet(prediction, confidence);
        self.current_bet = next_bet_data.0;
        self.chance = next_bet_data.2;
        let high = next_bet_data.3;

        self.chance = self.chance.max(2.);

        if self.history.len() < self.history_size {
            self.current_bet = self.currency.get_min_bet(Sites::DuckDiceIo);
            self.chance = 50.;
        }

        self.current_bet = self
            .current_bet
            .max(self.currency.get_min_bet(Sites::DuckDiceIo));

        let bet_url =
            Url::parse_with_params("https://duckdice.io/api/play", &[("api_key", API_KEY)])
                .expect("Failed to parse do_bet URL");

        if self.use_fake_betting {
            let bet_result = duckdice_fake_bet(
                high,
                "BeO2jZRd4nidPz4U40e2G7hT22s9GA",
                self.current_bet,
                next_bet_data.1,
            );

            self.history.push(bet_result.clone().into());
            if self.history.len() > self.history_size {
                self.history = self.history[1..].to_vec();
            }

            if self.current_bet > self.site_balance {
                self.current_bet = self.currency.get_min_bet(Sites::DuckDiceIo);
            }

            if self.current_bet > self.site_balance {
                panic!("Insufficient Balance");
            }

            return Ok(bet_result.into());
        }

        if self.current_bet > self.balance && self.use_site_balance {
            println!("[FAIL] Resetting {:0>.8}", self.site_balance);
            self.balance = self.site_balance * self.balance_modifier;
            self.initial_balance = self.site_balance * self.balance_modifier;
            self.wins = 0;
            self.losses = 0;
            self.seed_profit = 0.;
            self.strategy.reset();
            let randomize_url = Url::parse_with_params(
                "https://duckdice.io/api/randomize",
                &[("api_key", API_KEY)],
            )
            .expect("Failed to parse randomize URL");

            self.client_seed = rand::rng()
                .sample_iter(rand::distr::Alphabetic)
                .take(30)
                .map(char::from)
                .collect();
            let res_randomize = self
                .client
                .post(randomize_url)
                .json(&json!({
                    "clientSeed": self.client_seed.clone(),
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
            let user_info_url = Url::parse(&format!(
                "https://duckdice.io/api/bot/user-info?api_key={API_KEY}",
            ))
            .expect("Failed to parse user_info URL");
            let res: serde_json::Value =
                self.client.get(user_info_url).send().await?.json().await?;
            let res: UserInfoJson = serde_json::from_value(res).unwrap();
            let res: UserInfo = res.into();

            for balance in &res.balances {
                if balance.currency == self.currency.to_string().as_str() {
                    if let Some(main) = &if self.faucet {
                        balance.faucet.clone()
                    } else {
                        balance.main.clone()
                    } {
                        let _val = main.parse::<f32>().unwrap_or(0.);
                        self.strategy
                            .set_balance(self.site_balance * self.balance_modifier);
                        self.balance = self.site_balance * self.balance_modifier;
                        self.initial_balance = self.site_balance * self.balance_modifier;
                        self.strategy.reset();
                    }
                }
            }

            let next_bet_data = self.strategy.get_next_bet(prediction, confidence);
            self.current_bet = next_bet_data.0;
            self.chance = next_bet_data.2;
            let _high = next_bet_data.3;
        }

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
                user_wagering_bonus_hash: None, /*Some("97a8d827da".to_string()),*/
                faucet: if self.faucet { Some(true) } else { None },
                tle_hash: if self.faucet {
                    None
                } else {
                    self.tle_hash.clone()
                },
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
                        .get(format!("https://duckdice.io/api/bet/{}", res.bet.hash))
                        .send()
                        .await?;
                    let res_bet_data: serde_json::Value = res_bet_data.json().await?;

                    res.bet.previous_hash = self.previous_hash.clone();
                    self.previous_hash = res_bet_data["seed"]["serverSeedHash"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    self.client_seed = res_bet_data["seed"]["clientSeed"]
                        .as_str()
                        .unwrap()
                        .to_string();
                    res.bet.hash = self.previous_hash.clone();
                    self.initialized_hash = true;
                } else {
                    res.bet.previous_hash = self.previous_hash.clone();
                    res.bet.hash = self.previous_hash.clone();
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

                let mut bet_result: BetResult = res.into();
                bet_result.client_seed = self.client_seed.clone();

                Ok(bet_result)
            }
            Err(_) => Err(BetError::EmptyReply),
        }
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.offline_balance += bet_result.win_amount;
        self.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        self.seed_profit += bet_result.win_amount;
        self.strategy.on_win(bet_result);
        self.wins += 1;
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.offline_balance += bet_result.win_amount;
        self.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        self.seed_profit += bet_result.win_amount;
        self.losses += 1;
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

impl SiteConfig for DuckDiceIo {
    fn with_api_key(mut self, api_key: String) -> Self
    where
        Self: Sized,
    {
        self.api_key = api_key;

        self
    }

    fn with_currency(mut self, currency: Currency) -> Self
    where
        Self: Sized,
    {
        self.currency = currency;

        self
    }

    fn with_strategy(mut self, strategy: TomlStrategies) -> Self
    where
        Self: Sized,
    {
        self.strategy = match strategy {
            TomlStrategies::AiFight => Box::new(crate::strategies::ai_fight::AIFight::default()),
            TomlStrategies::MyStrategy => {
                Box::new(crate::strategies::my_strategy::MyStrat::default())
            }
            TomlStrategies::BlaksRunner => {
                Box::new(crate::strategies::blaks_runner::BlaksRunner5_0::default())
            }
            TomlStrategies::None => Box::new(crate::strategies::none::NoStrat::default()),
        };

        self
    }
}
