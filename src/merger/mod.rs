use crate::{proto, OrderBook};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

/// The number of order book entries to keep for processing.
/// Set it to 10 since that is the output of the gRPC stream.
const NUM_ORDER_BOOK_ENTRIES: usize = 10;

#[derive(Default)]
pub struct OrderBookMerger {
    /// The up-to-date state of the exchanges' order-books.
    /// `<exchange-name> => <order-book>`
    pub order_books: HashMap<&'static str, OrderBook>,
}

impl OrderBookMerger {
    /// Read from the order-book stream and merge them as they arrive.
    /// Send the merged order books out on the broadcast channel.
    pub async fn start(
        &mut self,
        tx: broadcast::Sender<proto::Summary>,
        mut rx: mpsc::Receiver<OrderBook>,
    ) {
        while let Some(mut order_book) = rx.recv().await {
            let exchange_name = order_book.exchange.clone();
            // Truncate the bids and asks.
            order_book.bids.truncate(NUM_ORDER_BOOK_ENTRIES);
            order_book.asks.truncate(NUM_ORDER_BOOK_ENTRIES);

            // Update the order-book state.
            self.order_books.insert(exchange_name, order_book);
            eprintln!("{:?}", self.order_books.get(exchange_name));

            // Merge the order books and send to the broadcast channel.
            let binance_bids = match self.order_books.get("binance") {
                None => continue,
                Some(order_book) => order_book.bids.clone(),
            };
            let bitstamp_asks = match self.order_books.get("bitstamp") {
                None => continue,
                Some(order_book) => order_book.asks.clone(),
            };
            let spread = bitstamp_asks.first().unwrap().price - binance_bids.first().unwrap().price;

            let merged_books = proto::Summary {
                spread,
                bids: binance_bids,
                asks: bitstamp_asks,
            };

            tx.send(merged_books);
        }
    }
}
