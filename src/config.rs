use serde::Deserialize;

use crate::currency::Currency;

#[derive(Debug, Default, Deserialize)]
pub enum TomlStrategies {
    AiFight,
    BlaksRunner,
    MyStrategy,
    #[default]
    None,
}

#[derive(Debug, Deserialize)]
pub struct CryptoGamesConfig {
    pub enabled: bool,
    pub api_key: String,
    pub currency: Currency,
    pub strategy: TomlStrategies,
}

#[derive(Debug, Deserialize)]
pub struct FreeBitcoInConfig {
    pub enabled: bool,
    pub btc_address: String,
    pub password: String,
    pub strategy: TomlStrategies,
}

#[derive(Debug, Deserialize)]
pub struct DuckDiceConfig {
    pub enabled: bool,
    pub api_key: String,
    pub currency: Currency,
    pub strategy: TomlStrategies,
}

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub crypto_games: CryptoGamesConfig,
    pub freebitcoin: FreeBitcoInConfig,
    pub duck_dice: DuckDiceConfig,
}

pub trait SiteConfig {
    fn with_api_key(mut self, api_key: String) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn with_username(mut self) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn with_password(mut self) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn with_currency(mut self, currency: Currency) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn with_strategy(mut self, strategy: TomlStrategies) -> Self
    where
        Self: Sized,
    {
        self
    }
}
