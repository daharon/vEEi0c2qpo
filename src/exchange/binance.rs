use std::error::Error;

use crate::common::{order_book_entries_to_rpc_levels, OrderBookEntry};
use crate::exchange::Exchange;
use crate::OrderBook;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

static EXCHANGE_NAME: &str = "binance";
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

impl Into<OrderBook> for BinanceOrderBookMessage {
    fn into(self) -> OrderBook {
        let mut bids = order_book_entries_to_rpc_levels(EXCHANGE_NAME, self.bids);
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        let mut asks = order_book_entries_to_rpc_levels(EXCHANGE_NAME, self.asks);
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        OrderBook {
            exchange: EXCHANGE_NAME,
            bids,
            asks,
        }
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
