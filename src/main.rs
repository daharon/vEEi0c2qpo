mod exchange;

#[tokio::main]
async fn main() {
    let binance_stream = tokio::spawn(async {
        exchange::binance::Binance::start("ethbtc").await
            .expect("Binance stream failed to start.");
    });
    binance_stream.await;
}
