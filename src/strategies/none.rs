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
    base_min_bet: f32,
    base_bet: f32,
    loss_streak: u32,
    win_streak: u32,
    high: bool,
    win_target: f32,
    loss: f32,
    initial_bank: f32,
    set_base_bet: bool,
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
            chance: 0.02,
            house_percent: 1.,
            current_bet: 2e-8,
            bank: 1e-4,
            profit: 0.,
            spent: 0.,
            base_bet: 2e-8,
            base_min_bet: 2e-8,
            loss_streak: 0,
            win_streak: 0,
            win_target: 0.,
            loss: 0.,
            initial_bank: 0.,
            set_base_bet: false,
        }
    }
}

impl Strategy for NoStrat {
    fn get_next_bet(&mut self, prediction: f32, confidence: f32) -> (f32, f32, f32, bool) {
        self.high = prediction > 5000.;
        self.current_bet = self.current_bet.max(self.base_bet);

        self.normal_strat(prediction, confidence);
        if !self.set_base_bet {
            self.base_bet = (self.bank / 50.).max(self.base_bet);
            self.set_base_bet = true;
        }
        // self.current_bet = self.base_bet;

        (self.current_bet, self.multiplier, self.chance, self.high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.spent -= bet_result.win_amount;
        self.spent = self.spent.max(0.);
        self.profit += bet_result.win_amount;
        self.bank += bet_result.win_amount;
        self.win_streak += 1;
        self.loss_streak = 0;
        self.loss -= bet_result.win_amount;
        self.loss = self.loss.max(0.);

        /*
        self.chance = self.min_chance;
        self.multiplier = 1. / (self.chance / 100.);
        self.multiplier = self.multiplier.clamp(1.01, 4750.);
        */
        self.current_bet += bet_result.win_amount * 0.25;
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.spent += bet_result.win_amount;
        self.profit -= bet_result.win_amount;
        self.bank -= bet_result.win_amount;
        self.loss_streak += 1;
        self.win_streak = 0;
        self.loss += bet_result.win_amount;

        self.current_bet = self.base_bet;

        /*
        self.chance += self.min_chance;
        self.multiplier = 1. / (self.chance / 100.);
        self.multiplier = self.multiplier.clamp(1.01, 4750.);
        self.current_bet += self.current_bet / (self.multiplier - 1.15);
        */
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
        self.initial_bank = balance;
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
        self.current_bet = self.base_min_bet;
        self.bank = self.initial_bank;
        self.base_bet = (self.bank / 50.).max(self.base_min_bet);
        self.loss_streak = 0;
        self.win_streak = 0;
        self.set_base_bet = false;
    }

    fn with_min_bet(mut self, min_bet: f32) -> Self
    where
        Self: Sized,
    {
        self.base_bet = min_bet;
        self.base_min_bet = min_bet;

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
