use std::error::Error;

use crate::common::OrderBookEntry;
use crate::exchange::Exchange;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

const STREAM_ENDPOINT: &str = "wss://stream.binance.com:9443/ws/";
const STREAM_SUFFIX: &str = "@depth10@100ms";

pub struct Binance;

#[derive(Debug, Deserialize, PartialEq)]
pub struct BinanceOrderBookMessage {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: i64,
    #[serde(deserialize_with = "crate::common::deserialize_order_book_entries")]
    pub bids: Vec<OrderBookEntry>,
    #[serde(deserialize_with = "crate::common::deserialize_order_book_entries")]
    pub asks: Vec<OrderBookEntry>,
}

impl Into<String> for BinanceOrderBookMessage {
    fn into(self) -> String {
        format!("{:?}", self).to_string()
    }
}

#[async_trait]
impl Exchange for Binance {
    type OrderBookMessage = BinanceOrderBookMessage;

    async fn connect(
        trading_pair: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        let stream_endpoint = format!("{}{}{}", STREAM_ENDPOINT, trading_pair, STREAM_SUFFIX);
        let url = Url::parse(&stream_endpoint)?;

        let (ws, _) = connect_async(url).await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::OrderBookEntry;
    use crate::exchange::binance::BinanceOrderBookMessage;

    #[test]
    fn deserialize_order_book_message() {
        let expected = BinanceOrderBookMessage {
            last_update_id: 4736432536,
            bids: vec![
                OrderBookEntry {
                    price: 0.07642400,
                    quantity: 5.87980000,
                },
                OrderBookEntry {
                    price: 0.07642100,
                    quantity: 2.51320000,
                },
            ],
            asks: vec![
                OrderBookEntry {
                    price: 0.07642500,
                    quantity: 8.80000000,
                },
                OrderBookEntry {
                    price: 0.07642700,
                    quantity: 0.12400000,
                },
            ],
        };
        let msg = include_str!("../../tests/binance_order_book_message.json");
        let actual = serde_json::from_str::<BinanceOrderBookMessage>(msg);
        println!("{:?}", actual);

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }
}
