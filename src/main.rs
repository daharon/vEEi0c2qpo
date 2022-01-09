use crate::common::OrderBook;
use crate::exchange::Exchange;
use crate::merger::OrderBookMerger;
use crate::rpc::server::OrderbookAggregatorService;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("orderbook");
}
mod common;
mod exchange;
mod merger;
mod rpc;

const TRADING_PAIR: &str = "ethbtc";

#[tokio::main]
async fn main() {
    // TODO: Remove this.
    let test = proto::Level {
        exchange: "test-exchange".to_string(),
        price: 0.12345,
        amount: 1.0,
    };
    println!("{:?}", test);
    // Start the exchange readers and receive a stream of order-books.
    let order_books_rx = start_exchange_readers().await;

    // Start the order-book merger coroutine.
    let (merged_tx, mut merged_rx) = broadcast::channel(100);
    let mtx = merged_tx.clone();
    let merger = tokio::spawn(async move {
        OrderBookMerger::default().start(mtx, order_books_rx).await;
    });

    // Start the gRPC service.
    println!("Staring gRPC server...");
    let orderbook_aggregator_service = OrderbookAggregatorService::new(merged_tx);
    let service = proto::orderbook_aggregator_server::OrderbookAggregatorServer::new(
        orderbook_aggregator_service,
    );
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    Server::builder().add_service(service).serve(addr).await;
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
