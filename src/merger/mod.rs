use crate::OrderBook;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

/// The number of order book entries to keep for processing.
/// Set it to 10 since that is the output of the gRPC stream.
const NUM_ORDER_BOOK_ENTRIES: usize = 10;

#[derive(Default)]
pub struct OrderBookMerger {
    pub order_books: HashMap<&'static str, OrderBook>,
}

impl OrderBookMerger {
    /// Read from the order-book stream and merge them as they arrive.
    /// Send the merged order books out on the broadcast channel.
    pub async fn start(
        &mut self,
        tx: broadcast::Sender<String>,
        mut rx: mpsc::Receiver<OrderBook>,
    ) {
        while let Some(mut order_book) = rx.recv().await {
            let exchange_name = order_book.exchange.clone();
            // Truncate the bids and asks.
            order_book.bids.truncate(NUM_ORDER_BOOK_ENTRIES);
            order_book.asks.truncate(NUM_ORDER_BOOK_ENTRIES);

            self.order_books.insert(exchange_name, order_book);
            println!("{:?}", self.order_books.get(exchange_name));

            tx.send("Merger!".to_string());
        }
    }
}
