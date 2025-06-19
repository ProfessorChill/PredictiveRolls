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

        if self.gain * 0.25 > self.loss / 100. {
            self.current_bet = self.min_bet + (self.gain * 0.25);
        } else {
            self.current_bet = self.loss / 100.;
        }
        self.current_bet = self.current_bet.max(self.min_bet);
        self.multiplier = base_multiplier;

        (self.current_bet, self.multiplier, self.chance, self.high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.profit += bet_result.win_amount;
        self.bank += bet_result.win_amount;
        self.loss -= bet_result.win_amount;
        self.gain += bet_result.win_amount;
        self.win_streak += 1;

        // self.current_bet /= self.multiplier - 1.;
        self.loss = self.loss.max(0.);
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.profit -= bet_result.win_amount;
        self.bank -= bet_result.win_amount;
        self.gain -= bet_result.win_amount * 2.;
        self.loss += self.current_bet;
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
}
