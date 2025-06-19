use crate::sites::BetResult;
use crate::strategies::Strategy;

#[derive(Debug)]
pub struct AIFight {
    high: bool,
    win_streak: usize,
    loss_streak: usize,
    min_bet: f32,
    rolls: usize,
    multiplier: f32,
    bank: f32,
    current_bet: f32,
    max_chance: f32,
    min_chance: f32,
    chance: f32,
    house_percent: f32,
    profit: f32,
    initialized: bool,
}

impl Default for AIFight {
    fn default() -> Self {
        Self {
            high: false,
            win_streak: 0,
            loss_streak: 0,
            min_bet: 1e-8,
            rolls: 0,
            multiplier: 2.,
            bank: 1e-8,
            current_bet: 1e-8,
            max_chance: 50.,
            min_chance: 0.02,
            chance: 50.,
            house_percent: 5.,
            profit: 0.,
            initialized: false,
        }
    }
}

impl Strategy for AIFight {
    fn with_initial_bet(mut self, initial_bet: f32) -> Self {
        self.current_bet = initial_bet;

        self
    }

    fn with_balance(mut self, balance: f32) -> Self {
        self.bank = balance;

        self
    }

    fn with_min_bet(mut self, min_bet: f32) -> Self {
        self.min_bet = min_bet;

        self
    }

    fn set_balance(&mut self, balance: f32) {
        self.bank = balance;
    }

    fn get_next_bet(&mut self, prediction: f32, _confidence: f32) -> (f32, f32, f32, bool) {
        self.high = prediction > 5000.;
        if !self.initialized && prediction == 0. {
            self.initialized = true;
            return (1e-8, 2., 50., self.high);
        }

        self.rolls += 1;
        let bet_multiplier = 1.
            + (if self.win_streak > self.loss_streak {
                self.win_streak as f32 - self.loss_streak as f32
            } else {
                self.loss_streak as f32 - self.win_streak as f32
            }) * 0.05;

        self.chance = (50. + self.house_percent) * (1. - ((prediction - 5000.).abs() / 5000.));
        self.chance = self.chance.max(self.min_chance).min(self.max_chance);

        self.multiplier = 1. / (self.chance / 100.);
        self.multiplier = self.multiplier.clamp(1.01, 4750.);
        let bet_win_adjustment = self.min_bet * (self.win_streak as f32 * 0.5);
        self.current_bet = bet_win_adjustment + (self.bank * 1e-4 * bet_multiplier);
        self.current_bet = self.current_bet.max(self.min_bet).min(self.bank);

        (self.current_bet, self.multiplier, self.chance, self.high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.bank += bet_result.win_amount;
        self.profit += bet_result.win_amount;
        self.win_streak += 1;
        self.loss_streak = self.loss_streak.saturating_sub(1);
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.bank -= bet_result.win_amount;
        self.profit -= bet_result.win_amount;
        self.loss_streak += 1;
        self.win_streak = self.win_streak.saturating_sub(1);
    }

    fn get_balance(&self) -> f32 {
        self.bank
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }
}
