use crate::sites::BetResult;
use crate::strategies::Strategy;

#[derive(Debug)]
pub struct MyStrat {
    high: bool,
    multiplier: f32,
    max_chance: f32,
    min_chance: f32,
    min_bet: f32,
    initial_bet: f32,
    chance: f32,
    house_percent: f32,
    current_bet: f32,
    bank: f32,
    profit: f32,
    loss: f32,
    gain: f32,
    win_streak: u32,
    loss_streak: u32,
    avg_losses_per_win: Vec<u32>,
}

impl Default for MyStrat {
    fn default() -> Self {
        Self {
            high: false,
            multiplier: 2.,
            max_chance: 70.,
            min_chance: 0.04,
            min_bet: 1e-8,
            initial_bet: 1e-8,
            chance: 50.,
            house_percent: 1.,
            current_bet: 1e-8,
            bank: 1e-8,
            profit: 0.,
            loss: 0.,
            gain: 0.,
            win_streak: 0,
            loss_streak: 0,
            avg_losses_per_win: Vec::new(),
        }
    }
}

impl Strategy for MyStrat {
    fn get_next_bet(&mut self, prediction: f32, _confidence: f32) -> (f32, f32, f32, bool) {
        self.high = prediction > 5000.;

        let chance = (50. + self.house_percent) * (1. - ((prediction - 5000.).abs() / 5000.));
        self.chance = chance.max(self.min_chance).min(self.max_chance);

        let mut base_multiplier = 1. / (chance / 100.);
        base_multiplier = base_multiplier.clamp(1.02, 4750.0);

        /*
        if self.bank / 1000. > self.min_bet {
            self.min_bet = self.bank / 1000.;
        }
        */
        let avg_losses = self.avg_losses_per_win.iter().sum::<u32>() as f32
            / self.avg_losses_per_win.len() as f32;
        if !avg_losses.is_nan() && self.avg_losses_per_win.len() >= 10 {
            self.current_bet =
                (self.min_bet * avg_losses) + (self.loss.max(self.gain) / (base_multiplier - 1.));
        } else {
            self.current_bet = self.loss.max(self.gain) / (base_multiplier - 1.);
        }
        self.current_bet = self.current_bet.max(self.min_bet);

        self.multiplier = base_multiplier;

        (self.current_bet, self.multiplier, self.chance, self.high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.loss = 0.;
        self.gain += bet_result.win_amount * 0.25;
        self.profit += bet_result.win_amount;
        self.bank += bet_result.win_amount;
        if self.loss_streak > 1 {
            self.avg_losses_per_win.push(self.loss_streak);
            if self.avg_losses_per_win.len() > 10 {
                self.avg_losses_per_win = self.avg_losses_per_win[1..].to_vec();
            }
        }
        self.win_streak += 1;
        self.loss_streak = 0;

        self.loss = self.loss.max(0.);
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.profit -= bet_result.win_amount;
        self.bank -= bet_result.win_amount;
        self.gain -= bet_result.win_amount;
        self.loss += self.current_bet;
        self.loss_streak += 1;
        self.win_streak = 0;

        self.gain = self.gain.max(0.);
    }

    fn with_balance(mut self, balance: f32) -> Self
    where
        Self: Sized,
    {
        self.bank = balance;

        self
    }

    fn with_min_bet(mut self, min_bet: f32) -> Self
    where
        Self: Sized,
    {
        self.min_bet = min_bet;

        self
    }

    fn with_initial_bet(mut self, initial_bet: f32) -> Self
    where
        Self: Sized,
    {
        self.initial_bet = initial_bet;

        self
    }

    fn get_balance(&self) -> f32 {
        self.bank
    }

    fn set_balance(&mut self, balance: f32) {
        self.bank = balance;
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }

    fn reset(&mut self) {
        self.profit = 0.;
        self.gain = 0.;
        self.loss = 0.;
        self.loss_streak = 0;
        self.win_streak = 0;
    }
}
