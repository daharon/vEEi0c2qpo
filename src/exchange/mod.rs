use async_trait::async_trait;
use std::error::Error;

pub mod binance;
pub mod bitstamp;

#[async_trait]
pub trait Exchange {
    async fn start(trading_pair: &str) -> Result<(), Box<dyn Error>>;
}
