pub mod ai_fight;
pub mod blaks_runner;
pub mod my_strategy;
pub mod none;

use crate::sites::BetResult;

pub trait Strategy: std::fmt::Debug + Send {
    fn with_initial_bet(self, _initial_bet: f32) -> Self
    where
        Self: Sized,
    {
        self
    }
    fn with_balance(self, _balance: f32) -> Self
    where
        Self: Sized,
    {
        self
    }
    fn with_min_bet(self, _min_bet: f32) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn set_balance(&mut self, balance: f32);

    /// Returns: (current_bet, multiplier, chance, high/low)
    fn get_next_bet(&mut self, prediction: f32, confidence: f32) -> (f32, f32, f32, bool);
    fn on_win(&mut self, bet_result: &BetResult);
    fn on_lose(&mut self, bet_result: &BetResult);
    fn get_balance(&self) -> f32;
    fn get_profit(&self) -> f32;
    fn get_win_target(&self) -> f32 {
        0.
    }
    fn reset(&mut self) {}
}
