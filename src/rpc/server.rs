use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use proto::orderbook_aggregator_server::OrderbookAggregator;

use crate::proto;

pub struct OrderbookAggregatorService {
    /// Subscribe to this broadcast channel for the merged order-book stream.
    broadcast_tx: broadcast::Sender<String>,
}

impl OrderbookAggregatorService {
    pub fn new(channel: broadcast::Sender<String>) -> Self {
        Self {
            broadcast_tx: channel,
        }
    }
}

#[async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = ReceiverStream<Result<proto::Summary, Status>>;

    async fn book_summary(
        &self,
        request: Request<proto::Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        eprintln!("Client connected from {:?}", request.remote_addr());

        let (tx, rx) = mpsc::channel(10);
        let mut merged_order_books = self.broadcast_tx.subscribe();

        tokio::spawn(async move {
            let mut counter = 0.0;
            while let Ok(msg) = merged_order_books.recv().await {
                // TODO:  Populate with real data.
                let summary = proto::Summary {
                    spread: 0.0,
                    bids: vec![proto::Level {
                        exchange: msg.to_string(),
                        price: counter,
                        amount: counter + 1.0,
                    }],
                    asks: vec![proto::Level {
                        exchange: msg.to_string(),
                        price: counter,
                        amount: counter + 1.0,
                    }],
                };
                tx.send(Ok(summary)).await;
                counter += 0.0001;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
