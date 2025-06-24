use sikkara_core::{AppResult, Collector, CollectorStream};
use tokio_stream::StreamExt;

use crate::engine::{InternalEvent, Pool, PoolFeed};

/// Collector that listens for updates from a pool feed client
#[derive(Debug, Clone)]
pub struct PoolFeedCollector<P>
where
    P: PoolFeed + Send + Sync,
{
    pub pool: Pool,
    pub client: P,
    pub name: String,
}

impl<P> PoolFeedCollector<P>
where
    P: PoolFeed + Send + Sync,
{
    pub fn new(pool: Pool, client: P) -> Self {
        let name = format!("pool_feed_collector_{}", pool.symbol);
        Self { pool, client, name }
    }
}

#[async_trait::async_trait]
impl<P> Collector<InternalEvent> for PoolFeedCollector<P>
where
    P: PoolFeed + Send + Sync,
{
    fn name(&self) -> &str { &self.name }

    async fn subscribe_event_stream(&mut self) -> AppResult<CollectorStream<'_, InternalEvent>> {
        let stream = self
            .client
            .subscribe_pool_updates(self.pool.clone())
            .await?;
        let stream = stream.filter_map(|update| Some(InternalEvent::PoolPriceUpdate(update)));
        Ok(Box::pin(stream))
    }

    async fn unsubscribe_event_stream(&mut self) -> AppResult<()> { Ok(()) }
}
