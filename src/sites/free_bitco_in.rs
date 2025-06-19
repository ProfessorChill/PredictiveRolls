use async_trait::async_trait;
use rand::{thread_rng, Rng};
use reqwest::{cookie::Jar, Url};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    sites::{fake_test::free_bitcoin_fake_bet, BetError, BetResult, Site},
    strategies::Strategy,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub csrf_token: String,
    pub op: String,
    pub btc_address: String,
    pub password: String,
    pub tfa_code: String,
}

#[derive(Debug, Clone)]
pub struct BetSiteResult {
    pub success_code: String,
    pub result: bool,
    pub rolled_number: u32,
    pub user_balance: f32,
    pub amount_won: f32,
    pub server_seed_hash_next_roll: String,
    pub client_seed_previous_roll: String,
    pub nonce_next_roll: String,
    pub server_seed_previous_roll: String,
    pub server_seed_hash_previous_roll: String,
    pub previous_nonce: String,
    pub jackpot_result: u8,
    pub jackpot_amount_won: f32,
    pub bonus_account_balance_after_bet: f32,
    pub bonus_acount_wager_remaining: f32,
    pub max_amount_bonus_eligable: f32,
    pub max_bet: f32,
    pub account_balance_before_bet: f32,
    pub account_balance_after_bet: f32,
    pub bonus_account_balance_before_bet: f32,
}

impl From<&str> for BetSiteResult {
    fn from(value: &str) -> Self {
        let bet_split = value
            .split(':')
            .map(|val| val.to_string())
            .collect::<Vec<String>>();

        if bet_split.len() < 22 {
            panic!("{value:?}");
        }

        Self {
            success_code: bet_split[0].clone(),
            result: bet_split[1].as_str() == "w",
            rolled_number: bet_split[2].parse::<u32>().unwrap(),
            user_balance: bet_split[3].parse::<f32>().unwrap(),
            amount_won: bet_split[4].parse::<f32>().unwrap(),
            server_seed_hash_next_roll: bet_split[6].clone(),
            client_seed_previous_roll: bet_split[7].clone(),
            nonce_next_roll: bet_split[8].clone(),
            server_seed_previous_roll: bet_split[9].clone(),
            server_seed_hash_previous_roll: bet_split[10].clone(),
            previous_nonce: bet_split[12].clone(),
            jackpot_result: bet_split[13].parse::<u8>().unwrap(),
            jackpot_amount_won: bet_split[15].parse::<f32>().unwrap(),
            bonus_account_balance_after_bet: bet_split[16].parse::<f32>().unwrap(),
            bonus_acount_wager_remaining: bet_split[17].parse::<f32>().unwrap(),
            max_amount_bonus_eligable: bet_split[18].parse::<f32>().unwrap(),
            max_bet: bet_split[19].parse::<f32>().unwrap(),
            account_balance_before_bet: bet_split[20].parse::<f32>().unwrap(),
            account_balance_after_bet: bet_split[21].parse::<f32>().unwrap(),
            bonus_account_balance_before_bet: bet_split[22].parse::<f32>().unwrap(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserStats {
    pub balance: f32,
    pub dice_profit: f32,
    pub jackpot_spent: f32,
    pub jackpot_winnings: f32,
    pub lottery_spent: f32,
    pub reward_points: u32,
    pub rolls_played: u64,
    pub status: String,
    pub total_winnings: f32,
    pub wagered: f32,
}

impl From<serde_json::Value> for UserStats {
    fn from(value: serde_json::Value) -> Self {
        Self {
            balance: (value["balance"].as_f64().unwrap() * 1e-8f64) as f32,
            dice_profit: (value["dice_profit"].as_f64().unwrap() * 1e-8f64) as f32,
            jackpot_spent: (value["jackpot_spent"].as_f64().unwrap() * 1e-8f64) as f32,
            jackpot_winnings: (value["jackpot_winnings"].as_f64().unwrap() * 1e-8f64) as f32,
            lottery_spent: (value["lottery_spent"].as_f64().unwrap() * 1e-8f64) as f32,
            reward_points: value["reward_points"].as_u64().unwrap() as u32,
            rolls_played: value["rolls_played"].as_u64().unwrap(),
            status: value["status"].as_str().unwrap().to_string(),
            total_winnings: (value["total_winnings"].as_f64().unwrap() * 1e-8f64) as f32,
            wagered: (value["wagered"].as_f64().unwrap() * 1e-8f64) as f32,
        }
    }
}

impl Default for UserStats {
    fn default() -> Self {
        Self {
            balance: 0.00000400,
            dice_profit: 0.,
            jackpot_spent: 0.,
            jackpot_winnings: 0.,
            lottery_spent: 0.,
            reward_points: 0,
            rolls_played: 0,
            status: String::new(),
            total_winnings: 0.,
            wagered: 0.,
        }
    }
}

pub struct FreeBitcoIn {
    pub rolls: u64,
    pub client_seed: String,
    pub current_bet: f32,
    pub multiplier: f32,
    pub user_stats: UserStats,
    pub profit: f32,
    pub prediction: u32,
    pub strategy: Box<dyn Strategy>,
    client: reqwest::Client,
    cookie_jar: Arc<Jar>,
    csrf_token: String,
    history: Vec<BetResult>,
    history_size: usize,
    use_site_balance: bool,
    use_fake_betting: bool,
    wins: u64,
    loses: u64,
}

impl Default for FreeBitcoIn {
    fn default() -> Self {
        Self {
            rolls: 0,
            client_seed: "lYypIPVEgzvCflWF".to_string(),
            current_bet: 2e-8,
            multiplier: 2.,
            user_stats: UserStats::default(),
            profit: 0.,
            prediction: 0,
            strategy: Box::new(
                // crate::strategies::blaks_runner::BlaksRunner5_0::default()
                // crate::strategies::none::NoStrat::default()
                crate::strategies::my_strategy::MyStrat::default()
                    .with_balance(274e-8)
                    .with_min_bet(2e-8)
                    .with_initial_bet(2e-8),
            ),
            client: reqwest::Client::new(),
            cookie_jar: Arc::new(Jar::default()),
            csrf_token: String::new(),
            history: Vec::new(),
            history_size: 10,
            use_site_balance: true,
            use_fake_betting: false,
            wins: 0,
            loses: 0,
        }
    }
}

#[async_trait]
impl Site for FreeBitcoIn {
    async fn login(&mut self) -> Result<(), BetError> {
        self.client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(Arc::clone(&self.cookie_jar))
            .build()?;

        if !self.use_site_balance {
            self.user_stats.balance = self.strategy.get_balance();
        }

        if self.use_fake_betting {
            return Ok(());
        }

        let url = "https://freebitco.in/"
            .parse::<reqwest::Url>()
            .expect("Failed to parse freebitco.in address");
        let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
            .chars()
            .collect::<Vec<char>>();
        let mut csrf_token = String::new();
        {
            let mut rng = thread_rng();
            for _ in 0..12 {
                let rand_num = rng.gen_range(0..charset.len());
                csrf_token.push(charset[rand_num]);
            }
        }
        self.csrf_token = csrf_token.clone();

        self.cookie_jar
            .add_cookie_str(&format!("csrf_token={csrf_token}; Path=/; Secure"), &url);

        let _ = self.client.get(url.clone()).send().await?;
        let login_post = LoginRequest {
            csrf_token: "".to_string(),
            op: "login_new".to_string(),
            btc_address: "".to_string(),
            password: "".to_string(),
            tfa_code: "".to_string(),
        };

        let login_response = self
            .client
            .post(url.clone())
            .form(&[
                ("csrf_token", login_post.csrf_token),
                ("op", login_post.op),
                ("btc_address", login_post.btc_address),
                ("password", login_post.password),
                ("tfa_code", login_post.tfa_code),
            ])
            .send()
            .await?
            .text()
            .await?;

        let login_res_split: Vec<&str> = login_response.split(':').collect();

        if login_res_split.len() < 4 {
            eprintln!("{login_response}");
            return Err(BetError::LoginFailed);
        }

        self.cookie_jar.add_cookie_str(
            &format!("btc_address={}; Path=/; Secure", login_res_split[1]),
            &url,
        );
        self.cookie_jar.add_cookie_str(
            &format!("fbtc_session={}; Path=/; Secure", login_res_split[4]),
            &url,
        );
        self.cookie_jar.add_cookie_str(
            &format!("fbtc_userid={}; Path=/; Secure", login_res_split[3]),
            &url,
        );
        self.cookie_jar
            .add_cookie_str("have_account=1; Path=/; Secure", &url);
        self.cookie_jar.add_cookie_str(
            &format!("password={}; Path=/; Secure", login_res_split[2]),
            &url,
        );

        let user_stats_res: serde_json::Value = self
            .client
            .get("https://freebitco.in/cgi-bin/api.pl?op=get_user_stats")
            .send()
            .await?
            .json()
            .await?;
        self.user_stats = UserStats::from(user_stats_res);
        if self.use_site_balance {
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
        let mut chance = (55.) * (1. - ((prediction - 5000.).abs() / 5000.));
        chance = chance.max(0.01).min(50.);

        let mut multiplier = 1. / (chance / 100.);
        multiplier = multiplier.clamp(1.01, 4750.);
        self.multiplier = multiplier;

        if self.history.len() < self.history_size {
            self.current_bet = 1e-8;
            self.multiplier = 2.;
        }

        if self.use_fake_betting {
            let bet_result =
                free_bitcoin_fake_bet(high, &self.client_seed, self.current_bet, self.multiplier);

            self.history.push(bet_result.clone().into());
            if self.history.len() > self.history_size {
                self.history = self.history[1..].to_vec();
            }

            if self.current_bet > self.user_stats.balance {
                self.loses += 1;
                self.strategy.set_balance(0.0001);
                self.strategy.reset();
                let next_bet_data = self.strategy.get_next_bet(prediction, confidence);
                self.current_bet = next_bet_data.0;
                self.multiplier = next_bet_data.1;

                panic!("W: {} || L: {}", self.wins, self.loses);
            }

            Ok(bet_result.into())
        } else {
            let bet_url = Url::parse_with_params(
                "https://freebitco.in/cgi-bin/bet.pl",
                &[
                    ("m", if high { "hi" } else { "lo" }),
                    ("client_seed", &self.client_seed),
                    ("jackpot", "0"),
                    ("stake", &format!("{:.8}", self.current_bet)),
                    ("multiplier", &format!("{:.2}", self.multiplier)),
                    ("csrf_token", &self.csrf_token.clone()),
                    ("rand", {
                        let mut rng = thread_rng();

                        &format!("{}", rng.gen::<f64>())
                    }),
                ],
            )
            .expect("Failed to create freebitco.in bet URL");

            let bet_response = self.client.get(bet_url).send().await?.text().await?;
            let bet_result = BetSiteResult::from(bet_response.as_str());

            self.history.push(bet_result.clone().into());
            if self.history.len() > self.history_size {
                self.history = self.history[1..].to_vec();
            }

            if self.current_bet > self.user_stats.balance {
                panic!("Not enough money!");
            }

            Ok(bet_result.into())
        }
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.user_stats.balance += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        self.strategy.on_win(bet_result);
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.user_stats.balance -= bet_result.win_amount;
        self.profit -= bet_result.win_amount;
        // let mut bet_result = bet_result.clone();
        // bet_result.win_amount = -bet_result.win_amount;
        self.strategy.on_lose(bet_result);
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
