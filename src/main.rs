use crate::exchange::Exchange;

mod common;
mod exchange;

const TRADING_PAIR: &str = "ethbtc";

#[tokio::main]
async fn main() {
    let (order_book_tx, mut order_book_rx) = tokio::sync::mpsc::channel(100);

    let binance_sender = order_book_tx.clone();
    let binance_stream = tokio::spawn(async move {
        exchange::binance::Binance::start(TRADING_PAIR, binance_sender)
            .await
            .expect("Binance stream failed to start.");
    });
    let bitstamp_sender = order_book_tx.clone();
    let bitstamp_stream = tokio::spawn(async move {
        exchange::bitstamp::Bitstamp::start(TRADING_PAIR, bitstamp_sender)
            .await
            .expect("Bitstamp stream failed to start.");
    });

    while let Some(msg) = order_book_rx.recv().await {
        println!("{}", msg);
    }

    binance_stream.await;
    bitstamp_stream.await;
}
