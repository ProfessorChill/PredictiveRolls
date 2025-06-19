use crate::sites::BetResult;
use crate::strategies::Strategy;

#[derive(Debug)]
pub struct NoStrat {
    multiplier: f32,
    max_chance: f32,
    min_chance: f32,
    chance: f32,
    house_percent: f32,
    current_bet: f32,
    bank: f32,
    profit: f32,
    spent: f32,
    base_bet: f32,
    loss_streak: u32,
    win_streak: u32,
    high: bool,
    win_target: f32,
}

impl NoStrat {
    fn normal_strat(&mut self, prediction: f32, _confidence: f32) {
        self.chance = (50. + self.house_percent) * (1. - ((prediction - 5000.).abs() / 5000.));
        self.chance = self.chance.max(self.min_chance).min(self.max_chance);

        self.multiplier = 1. / (self.chance / 100.);
        self.multiplier = self.multiplier.clamp(1.01, 4750.);
    }
}

impl Default for NoStrat {
    fn default() -> Self {
        Self {
            high: false,
            multiplier: 2.,
            max_chance: 50.,
            min_chance: 0.02,
            chance: 50.,
            house_percent: 5.,
            current_bet: 2e-8,
            bank: 1e-4,
            profit: 0.,
            spent: 0.,
            base_bet: 2e-8,
            loss_streak: 0,
            win_streak: 0,
            win_target: 0.,
        }
    }
}

impl Strategy for NoStrat {
    fn get_next_bet(&mut self, prediction: f32, confidence: f32) -> (f32, f32, f32, bool) {
        self.high = prediction > 5000.;
        self.normal_strat(prediction, confidence);
        self.current_bet = self.current_bet.max(1e-8);

        (self.current_bet, self.multiplier, self.chance, self.high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.spent -= bet_result.win_amount;
        self.spent = self.spent.max(0.);
        self.profit += bet_result.win_amount;
        self.bank += bet_result.win_amount;
        self.win_streak += 1;
        self.current_bet = self.base_bet;
        self.loss_streak = 0;
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.spent += bet_result.win_amount;
        self.profit -= bet_result.win_amount;
        self.bank -= bet_result.win_amount;
        self.loss_streak += 1;
        self.win_streak = 0;
    }

    fn with_balance(mut self, balance: f32) -> Self
    where
        Self: Sized,
    {
        self.bank = balance;
        self.profit = 0.;
        self.win_target = balance;

        self
    }

    fn get_balance(&self) -> f32 {
        self.bank
    }

    fn set_balance(&mut self, balance: f32) {
        self.bank = balance;
        self.profit = 0.;
        self.win_target = balance;
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }

    fn get_win_target(&self) -> f32 {
        self.win_target
    }

    fn reset(&mut self) {
        self.chance = 50.;
        self.profit = 0.;
        self.spent = 0.;
        self.current_bet = 2e-8;
        self.base_bet = 2e-8;
        self.loss_streak = 0;
        self.win_streak = 0;
    }

    fn with_min_bet(mut self, min_bet: f32) -> Self
    where
        Self: Sized,
    {
        self.base_bet = min_bet;

        self
    }

    fn with_initial_bet(mut self, initial_bet: f32) -> Self
    where
        Self: Sized,
    {
        self.base_bet = initial_bet;

        self
    }
}
