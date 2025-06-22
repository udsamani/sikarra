use sikkara_core::{AppResult, Collector, CollectorStream};
use tokio_stream::StreamExt;

use crate::{event::InternalEvent, models::PoolSymbol, price_feed::PriceFeed};

#[derive(Debug, Clone)]
pub struct PriceFeedCollector<P>
where
    P: PriceFeed + Send + Sync,
{
    pub symbol: PoolSymbol,
    pub client: P,
    pub name: String,
}

impl<P> PriceFeedCollector<P>
where
    P: PriceFeed + Send + Sync,
{
    pub fn new(symbol: PoolSymbol, client: P) -> Self {
        Self { symbol, client, name: "price_feed_collector".to_string() }
    }
}

#[async_trait::async_trait]
impl<P> Collector<InternalEvent> for PriceFeedCollector<P>
where
    P: PriceFeed + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn subscribe_event_stream(&mut self) -> AppResult<CollectorStream<'_, InternalEvent>> {
        let stream = self
            .client
            .subscribe_price_feed(self.symbol.clone())
            .await?;
        let stream = stream.filter_map(|ticker| {
            if ticker.symbol == self.symbol {
                Some(InternalEvent::TickerUpdate(ticker))
            } else {
                None
            }
        });
        Ok(Box::pin(stream))
    }

    async fn unsubscribe_event_stream(&mut self) -> sikkara_core::AppResult<()> {
        todo!()
    }
}
