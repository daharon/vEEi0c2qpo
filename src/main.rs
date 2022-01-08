use crate::exchange::Exchange;

mod exchange;

const TRADING_PAIR: &str = "ethbtc";

#[tokio::main]
async fn main() {
    //    let (order_book_tx, order_book_rx) = tokio::sync::mpsc::channel(100);

    let binance_stream = tokio::spawn(async {
        exchange::binance::Binance::start(TRADING_PAIR)
            .await
            .expect("Binance stream failed to start.");
    });
    let bitstamp_stream = tokio::spawn(async {
        exchange::bitstamp::Bitstamp::start(TRADING_PAIR)
            .await
            .expect("Bitstamp stream failed to start.");
    });

    binance_stream.await;
    bitstamp_stream.await;
}
