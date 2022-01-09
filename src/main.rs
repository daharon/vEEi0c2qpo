use crate::common::OrderBook;
use crate::exchange::Exchange;
use crate::merger::OrderBookMerger;
use tokio::sync::{broadcast, mpsc};

mod common;
mod exchange;
mod merger;

const TRADING_PAIR: &str = "ethbtc";

#[tokio::main]
async fn main() {
    // Start the exchange readers and receive a stream of order-books.
    let order_books_rx = start_exchange_readers().await;

    // Start the order-book merger coroutine.
    let (merged_tx, mut merged_rx) = broadcast::channel(100);
    let mtx = merged_tx.clone();
    let merger = tokio::spawn(async move {
        OrderBookMerger::default().start(mtx, order_books_rx).await;
    });

    let mut subscription = merged_tx.subscribe();
    while let Ok(merged_order_book) = subscription.recv().await {
        println!("Received merge order-book:  {}", merged_order_book);
    }
    //merger.await;
}

/// Start the exchange websocket readers.
/// Returns a live stream of order-books.
async fn start_exchange_readers() -> mpsc::Receiver<OrderBook> {
    let (tx, rx) = mpsc::channel(100);

    let binance_sender = tx.clone();
    let _binance_stream = tokio::spawn(async move {
        exchange::binance::Binance::start(TRADING_PAIR, binance_sender)
            .await
            .expect("Binance stream failed to start.");
    });
    let bitstamp_sender = tx.clone();
    let _bitstamp_stream = tokio::spawn(async move {
        exchange::bitstamp::Bitstamp::start(TRADING_PAIR, bitstamp_sender)
            .await
            .expect("Bitstamp stream failed to start.");
    });
    return rx;
}
