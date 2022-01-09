use crate::common::OrderBookEntry;
use crate::exchange::Exchange;
use crate::OrderBook;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use core::str::FromStr;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::error::Error;
use std::io::ErrorKind;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;

static EXCHANGE_NAME: &str = "bitstamp";
const STREAM_ENDPOINT: &str = "wss://ws.bitstamp.net/";

pub struct Bitstamp;

#[derive(Debug, Deserialize, PartialEq)]
pub struct BitstampOrderBookMessage {
    pub event: String,
    pub channel: String,
    pub data: BitstampOrderBookMessageData,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct BitstampOrderBookMessageData {
    #[serde(deserialize_with = "deserialize_order_book_ts")]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "crate::common::deserialize_order_book_entries")]
    pub bids: Vec<OrderBookEntry>,
    #[serde(deserialize_with = "crate::common::deserialize_order_book_entries")]
    pub asks: Vec<OrderBookEntry>,
}

fn deserialize_order_book_ts<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let unix_timestamp = i64::from_str(&s).map_err(serde::de::Error::custom)?;
    let ndt = NaiveDateTime::from_timestamp(unix_timestamp, 0);
    Ok(DateTime::from_utc(ndt, Utc))
}

impl Into<OrderBook> for BitstampOrderBookMessage {
    fn into(self) -> OrderBook {
        OrderBook {
            exchange: EXCHANGE_NAME,
            bids: self.data.bids,
            asks: self.data.asks,
        }
    }
}

#[async_trait]
impl Exchange for Bitstamp {
    type OrderBookMessage = BitstampOrderBookMessage;

    async fn connect(
        trading_pair: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        let url = Url::parse(STREAM_ENDPOINT)?;
        let (mut ws, _) = connect_async(url).await?;

        // Subscribe to the appropriate stream.
        let sub_req = Message::Text(subscription_request(trading_pair));
        ws.send(sub_req).await?;
        // Receive confirmation that the subscription was successful.
        if let Some(res) = ws.next().await {
            match res {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        let response = serde_json::from_str::<serde_json::Value>(&text)?;
                        // The expected response for a successful subscription.
                        if response["event"] != "bts:subscription_succeeded" {
                            return Err(Box::new(std::io::Error::new(
                                ErrorKind::NotConnected,
                                "Failed to subscribe to order-book stream.",
                            )));
                        }
                    }
                    _ => {
                        return Err(Box::new(std::io::Error::new(
                            ErrorKind::NotConnected,
                            "Failed to subscribe to order-book stream.",
                        )));
                    }
                },
                Err(err) => return Err(Box::new(err)),
            }
        }

        Ok(ws)
    }
}

fn subscription_request(trading_pair: &str) -> String {
    json!({
        "event": "bts:subscribe",
        "data": {
            "channel": format!("order_book_{}", trading_pair)
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{BitstampOrderBookMessage, BitstampOrderBookMessageData};
    use crate::common::OrderBookEntry;
    use chrono::{DateTime, TimeZone};

    #[test]
    fn deserialize_order_book_message() {
        let expected = BitstampOrderBookMessage {
            event: "data".to_string(),
            channel: "order_book_ethbtc".to_string(),
            data: BitstampOrderBookMessageData {
                timestamp: chrono::Utc.ymd(2022, 1, 8).and_hms(13, 14, 33),
                bids: vec![
                    OrderBookEntry {
                        price: 0.07638925,
                        quantity: 1.56365297,
                    },
                    OrderBookEntry {
                        price: 0.07638231,
                        quantity: 1.15000000,
                    },
                ],
                asks: vec![
                    OrderBookEntry {
                        price: 0.07644937,
                        quantity: 1.15000000,
                    },
                    OrderBookEntry {
                        price: 0.07645112,
                        quantity: 0.40000000,
                    },
                ],
            },
        };
        let msg = include_str!("../../tests/bitstamp_order_book_message.json");
        let actual = serde_json::from_str::<BitstampOrderBookMessage>(msg);
        println!("{:?}", actual);

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }
}
