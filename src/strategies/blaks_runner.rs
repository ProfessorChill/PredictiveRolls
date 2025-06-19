use crate::sites::BetResult;
use crate::strategies::Strategy;

#[derive(Debug)]
pub struct BlaksRunner5_0 {
    initialized: bool,
    pub base_chance: f32,
    pub chance_inc: f32,
    pub inc_divisor: f32,
    pub site_max_profit: f32,
    pub toggle_high_low: bool,
    pub bet_high: bool,
    pub rest_time: f32,
    pub max_win_mult: u64,
    pub house_percent: f32,
    pub max_bet: f32,
    pub min_bet: f32,
    pub base_bet: f32,
    pub chance_mult: f32,
    pub chance_max: f32,
    pub total_profit: f32,
    pub win_mult: f32,
    pub inc_roll: u64,
    pub start_balance: f32,
    pub bankroll: f32,
    pub old_base_chance: f32,
    pub current_step: u64,
    pub loss_count: u64,
    pub step_count: u64,
    pub spent: f32,
    pub high_low_loss_count: u64,
    pub high_low_average: [f32; 8],
    pub average_count: usize,
    pub average_index: u64,
    pub average_max: usize,
    pub roll_count: u64,
    pub roll_seed_count: u64,
    pub profit: f32,
    pub chance: f32,
    pub temp_win_mult: f32,
    pub next_bet: f32,
    pub min_chance: f32,
    pub max_chance: f32,
    pub win_target: f32,
}

impl BlaksRunner5_0 {
    fn calc_chance(&mut self, win: bool, last_rolled: u32) {
        if self.old_base_chance == 0. {
            self.old_base_chance = self.base_chance;
        }

        if win {
            let rolled_number = last_rolled / 100;
            let target = if rolled_number >= 50 {
                100 - last_rolled
            } else {
                last_rolled
            };

            self.high_low_average[self.average_count] = target as f32;
            self.average_count += 1;
            if self.average_count >= self.average_max {
                self.average_count = 0;
            }

            let mut temp_average = 0.;
            for i in self.high_low_average {
                temp_average += i;
            }
            temp_average /= self.average_max as f32;

            self.chance = temp_average;

            if self.chance > (self.old_base_chance * self.chance_max) {
                self.chance = self.old_base_chance * self.chance_max;
            }
        } else {
            let win_amount = (100. - (100. * (self.house_percent / 100.))) / self.chance;

            if self.loss_count as f32 > win_amount {
                self.chance += self.chance_inc;
            }
        }
    }

    fn auto_tune(&mut self) {
        let mut win_amount = (100. - (100. * (self.house_percent / 100.))) / self.chance;
        win_amount *= 1e-8;

        let temp_calc = 1.
            + ((self.chance / 100.)
                * ((100. - self.house_percent) / ((100. - self.house_percent) / 2.)));
        let needed = (win_amount * self.temp_win_mult) + (self.next_bet * temp_calc) + self.spent;

        let mut next_mult = needed / win_amount;
        if next_mult < 1. {
            next_mult = 1.;
        }
        self.next_bet = self.base_bet * next_mult;

        if (win_amount * self.next_bet) - self.next_bet > self.site_max_profit
            && self.site_max_profit != 0.
        {
            self.next_bet = self.site_max_profit / win_amount;
        }

        if self.next_bet > self.max_bet && self.max_bet != 0. {
            self.next_bet = self.max_bet;
        }
    }

    fn reset(&mut self) {
        let inc_divisor = 10000000.;
        self.base_chance = 1.;
        self.chance_inc = 0.00010;
        self.inc_divisor = inc_divisor;
        self.site_max_profit = 0.;
        self.toggle_high_low = false;
        self.bet_high = false;
        self.rest_time = 0.;
        self.max_win_mult = 512;
        self.house_percent = 5.;
        self.max_bet = 0.;
        self.chance_mult = 1.6666;
        self.chance_max = 1.5;
        self.total_profit = 0.;
        self.win_mult = 1.;
        self.inc_roll = 0;
        self.old_base_chance = 0.;
        self.current_step = 0;
        self.loss_count = 0;
        self.step_count = 0;
        self.spent = 0.;
        self.high_low_loss_count = 0;
        self.high_low_average = [0.; 8];
        self.average_count = 0;
        self.average_index = 0;
        self.average_max = 8;
        self.roll_count = 0;
        self.roll_seed_count = 0;
        self.chance = 1.;
        self.next_bet = 1e-8;
        self.temp_win_mult = 1.;
    }
}

impl Default for BlaksRunner5_0 {
    fn default() -> Self {
        let inc_divisor = 10000000.;

        Self {
            initialized: false,
            base_chance: 1.,
            chance_inc: 0.00010,
            inc_divisor,
            site_max_profit: 0.,
            toggle_high_low: false,
            bet_high: false,
            rest_time: 0.,
            max_win_mult: 512,
            house_percent: 5.,
            max_bet: 0.,
            min_bet: 1e-8,
            base_bet: 1e-8,
            chance_mult: 1.6666,
            chance_max: 1.5,
            total_profit: 0.,
            win_mult: 1.,
            inc_roll: 0,
            start_balance: 1e-8,
            bankroll: 1e-8,
            old_base_chance: 0.,
            current_step: 0,
            loss_count: 0,
            step_count: 0,
            spent: 0.,
            high_low_loss_count: 0,
            high_low_average: [0.; 8],
            average_count: 0,
            average_index: 0,
            average_max: 8,
            roll_count: 0,
            roll_seed_count: 0,
            profit: 0.,
            chance: 1.,
            next_bet: 1e-8,
            temp_win_mult: 1.,
            min_chance: 0.02,
            max_chance: 5.,
            win_target: 0.0001,
        }
    }
}

impl Strategy for BlaksRunner5_0 {
    fn with_initial_bet(mut self, initial_bet: f32) -> Self {
        self.base_bet = initial_bet;
        self.next_bet = initial_bet;

        self
    }

    fn with_balance(mut self, balance: f32) -> Self {
        self.start_balance = balance;
        self.bankroll = balance;
        self.win_target = balance;
        self.profit = 0.;

        self
    }

    fn with_min_bet(mut self, min_bet: f32) -> Self {
        self.min_bet = min_bet;

        self
    }

    fn set_balance(&mut self, balance: f32) {
        self.bankroll = balance;
        self.start_balance = balance;
        self.win_target = balance;
        self.profit = 0.;
    }

    fn get_next_bet(&mut self, prediction: f32, _confidence: f32) -> (f32, f32, f32, bool) {
        self.bet_high = prediction > 5000.;
        if prediction > 0. && !self.initialized {
            self.reset();
            self.initialized = true;
        }

        self.chance = (50. + self.house_percent) * (1. - ((prediction - 5000.).abs() / 5000.));
        self.chance = self.chance.max(self.min_chance).min(self.max_chance);
        self.auto_tune();

        let mut multiplier = 1. / (self.chance / 100.);
        multiplier = multiplier.clamp(1.01, 4750.);

        self.next_bet = self.next_bet.max(1e-8);

        (self.next_bet, multiplier, self.chance, self.bet_high)
    }

    fn on_win(&mut self, bet_result: &BetResult) {
        self.total_profit += bet_result.win_amount;
        self.chance = self.base_chance;
        self.loss_count = 0;
        self.step_count = 0;
        self.spent -= bet_result.win_amount;
        self.spent = self.spent.max(0.);
        self.high_low_loss_count = 0;
        self.start_balance += bet_result.win_amount;
        self.bankroll += bet_result.win_amount;
        self.profit += bet_result.win_amount;

        let temp_calc = self.bankroll * 1e8;
        let mut temp_mult = temp_calc / self.inc_divisor;
        if temp_mult < 1. {
            temp_mult = 1.;
        }
        self.win_mult = temp_mult;
        if self.win_mult > self.max_win_mult as f32 && self.max_win_mult != 0 {
            self.win_mult = self.max_win_mult as f32;
        }
        self.temp_win_mult = self.win_mult;
        // self.next_bet = self.base_bet;
        self.calc_chance(true, bet_result.number);
        self.auto_tune();
    }

    fn on_lose(&mut self, bet_result: &BetResult) {
        self.loss_count += 1;
        self.high_low_loss_count += 1;
        self.spent += bet_result.win_amount;
        self.bankroll -= bet_result.win_amount;
        self.profit -= bet_result.win_amount;

        let win_temp = (100. - (100. * (self.house_percent / 100.))) / self.chance;
        if self.high_low_loss_count as f32 >= win_temp {
            if self.toggle_high_low {
                self.bet_high = !self.bet_high;
            }

            if self.loss_count as f32 >= win_temp * 25. && self.temp_win_mult > 1. {
                self.temp_win_mult = 1.;
            }

            self.high_low_loss_count = 0;
        }

        self.calc_chance(false, bet_result.number);
        self.auto_tune();
    }

    fn get_balance(&self) -> f32 {
        self.bankroll
    }

    fn get_profit(&self) -> f32 {
        self.profit
    }

    fn get_win_target(&self) -> f32 {
        self.win_target
    }
}
