use std::net::SocketAddr;

use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;

use crate::common::{config::Config, OrderBook};
use crate::exchange::Exchange;
use crate::merger::OrderBookMerger;
use crate::proto::orderbook_aggregator_server::OrderbookAggregatorServer;
use crate::rpc::server::OrderbookAggregatorService;

mod proto {
    tonic::include_proto!("orderbook");
}
mod common;
mod exchange;
mod merger;
mod rpc;

#[tokio::main]
async fn main() {
    let config = Config::new();

    // Start the exchange readers and receive a stream of order-books.
    let order_books_rx = start_exchange_readers(&config.symbol.to_lowercase()).await;

    // Start the order-book merger coroutine.
    let (merged_tx, _) = broadcast::channel(100);
    let mtx = merged_tx.clone();
    tokio::spawn(async move {
        OrderBookMerger::default().start(mtx, order_books_rx).await;
    });

    // Start the gRPC service.
    println!("Staring gRPC server...");
    let orderbook_aggregator_service = OrderbookAggregatorService::new(merged_tx);
    let service = OrderbookAggregatorServer::new(orderbook_aggregator_service);
    let addr = SocketAddr::new(config.host, config.port);
    Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .expect("Failed to start the gRPC server.");
}

/// Start the exchange websocket readers.
/// Returns a live stream of order-books.
async fn start_exchange_readers(trading_symbol: &str) -> mpsc::Receiver<OrderBook> {
    let (tx, rx) = mpsc::channel(100);

    let binance_sender = tx.clone();
    let symbol = trading_symbol.to_string();
    let _binance_stream = tokio::spawn(async move {
        exchange::binance::Binance::start(&symbol, binance_sender)
            .await
            .expect("Binance stream failed to start.");
    });
    let bitstamp_sender = tx.clone();
    let symbol = trading_symbol.to_string();
    let _bitstamp_stream = tokio::spawn(async move {
        exchange::bitstamp::Bitstamp::start(&symbol, bitstamp_sender)
            .await
            .expect("Bitstamp stream failed to start.");
    });
    return rx;
}
