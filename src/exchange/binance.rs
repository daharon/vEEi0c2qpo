use std::error::Error;

use crate::exchange::Exchange;
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

const STREAM_ENDPOINT: &str = "wss://stream.binance.com:9443/ws/";
const STREAM_SUFFIX: &str = "@depth10@100ms";

pub struct Binance;

#[async_trait]
impl Exchange for Binance {
    async fn start(trading_pair: &str) -> Result<(), Box<dyn Error>> {
        let stream_endpoint = format!("{}{}{}", STREAM_ENDPOINT, trading_pair, STREAM_SUFFIX);
        let url = Url::parse(&stream_endpoint)?;

        let (ws, _) = connect_async(url).await?;
        let (_, ws_rx) = ws.split();

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
                    println!("Binance message received:  {}", data_str);
                }
            };
        });
        Ok(ws_reader.await)
    }
}
