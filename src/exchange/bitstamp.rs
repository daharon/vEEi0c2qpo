use crate::exchange::Exchange;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::error::Error;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

const STREAM_ENDPOINT: &str = "wss://ws.bitstamp.net/";

pub struct Bitstamp;

#[async_trait]
impl Exchange for Bitstamp {
    async fn start(trading_pair: &str) -> Result<(), Box<dyn Error>> {
        let url = Url::parse(STREAM_ENDPOINT)?;
        let (ws, _) = connect_async(url).await?;
        let (mut ws_tx, ws_rx) = ws.split();

        // Subscribe to the appropriate stream.
        let sub_req = Message::Text(subscription_request(trading_pair));
        ws_tx.send(sub_req).await?;

        // Read from the stream.
        let ws_reader = ws_rx.for_each(|msg_result| async {
            match msg_result {
                Err(_) =>
                // Drop problematic messages and continue.
                {
                    eprintln!("Error reading message from websocket.")
                }
                Ok(msg) => {
                    let data = msg.into_data();
                    let data_str = String::from_utf8_lossy(&data);
                    println!("Bitstamp message received:  {}", data_str);
                }
            };
        });
        Ok(ws_reader.await)
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
