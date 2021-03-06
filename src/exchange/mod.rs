use std::error::Error;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::common::OrderBook;

pub mod binance;
pub mod bitstamp;

#[async_trait]
pub trait Exchange {
    type OrderBookMessage: for<'a> serde::Deserialize<'a> + Into<OrderBook> + Send;

    /// Exchange-specific logic to connect to the exchange's websocket
    /// and subscribe to the appropriate order book stream.
    async fn connect(
        trading_pair: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>>;

    /// Connect and read from the exchange's websocket stream.
    async fn start(
        trading_symbol: &str,
        sink: mpsc::Sender<OrderBook>,
    ) -> Result<(), Box<dyn Error>> {
        let (mut tx, mut rx) = Self::connect(trading_symbol).await?.split();

        // Read from the stream.
        while let Some(message) = rx.next().await {
            match message {
                Err(_) => {
                    // Drop problematic messages and continue.
                    error!("Error reading message from websocket.");
                }
                Ok(m) => match m {
                    Message::Text(text) => {
                        debug!("Text message received:  {}", text);
                        match serde_json::from_str::<Self::OrderBookMessage>(&text) {
                            Err(err) => error!("Error deserializing message:  {}", err),
                            Ok(order_book_msg) => {
                                if let Err(err) = sink.send(order_book_msg.into()).await {
                                    error!(
                                        "Error sending order book message on the channel:  {}",
                                        err
                                    );
                                }
                            }
                        }
                    }
                    Message::Ping(_) => {
                        debug!("Received PING.  Sending PONG.");
                        tx.send(Message::Pong(vec![0; 0])).await.unwrap();
                    }
                    Message::Pong(_) => debug!("Received PONG."),
                    Message::Binary(_) => debug!("Skipping binary message handling."),
                    Message::Close(_) => debug!("Server closed the websocket connection."),
                },
            }
        }
        Ok(())
    }
}
