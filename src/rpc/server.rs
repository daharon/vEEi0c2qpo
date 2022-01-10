use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use proto::orderbook_aggregator_server::OrderbookAggregator;

use crate::proto;

pub struct OrderbookAggregatorService {
    /// Subscribe to this broadcast channel for the merged order-book stream.
    broadcast_tx: broadcast::Sender<proto::Summary>,
}

impl OrderbookAggregatorService {
    pub fn new(channel: broadcast::Sender<proto::Summary>) -> Self {
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
            while let Ok(summary) = merged_order_books.recv().await {
                tx.send(Ok(summary)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
