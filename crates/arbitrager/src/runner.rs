use futures::future::join_all;
use sikkara_adapters::CoinbaseWsClient;
use sikkara_core::{AppResult, EngineRunner, ExponentialBackoff, Runner};
use sikkara_wsclient::WsConsumer;
use tokio::sync::{broadcast, mpsc};

use crate::{
    action::InternalAction,
    collectors::PriceFeedCollector,
    config::{ArbitrageConfig, CexConfig},
    engine::ArbitrageEngine,
    event::InternalEvent,
    strategy::SimpleArbitrageStrategy,
};

#[derive(Debug, Clone)]
pub struct ArbitrageRunner {}

#[async_trait::async_trait]
impl Runner<ArbitrageConfig> for ArbitrageRunner {
    fn name(&self) -> &str {
        "arbitrage_runner"
    }

    async fn run(
        self,
        parameters: ArbitrageConfig,
        shutdown: tokio_util::sync::CancellationToken,
    ) -> AppResult<()> {
        let (ws_message_sender, ws_message_receiver) = mpsc::channel(100);
        let (message_broadcaster, _) = broadcast::channel(100);

        let client = match &parameters.cex {
            CexConfig::Coinbase { ws_url } => {
                CoinbaseWsClient::new(ws_url.clone(), ws_message_sender, message_broadcaster)
            },
        };
        let mut consumer = WsConsumer {
            ws_url: client.ws_url().to_string(),
            callback: client.clone(),
            heartbeat_millis: 5000,
            backoff: ExponentialBackoff::default(),
            receiver: ws_message_receiver,
        };

        let mut runner_tasks = Vec::with_capacity(parameters.pools.len());
        let child_token = shutdown.child_token();
        runner_tasks.push(consumer.spawn(child_token));

        for pool in &parameters.pools {
            let mut runner = EngineRunner::<InternalEvent, InternalAction>::new(
                pool.symbol().to_string(),
                500,
                500,
            );

            let engine =
                ArbitrageEngine::new(SimpleArbitrageStrategy {}, pool.symbol().to_string());
            runner.add_engine(Box::new(engine));

            let price_feed_collector = PriceFeedCollector::new(pool.symbol_owned(), client.clone());
            runner.add_collector(Box::new(price_feed_collector));

            let parameters_clone = parameters.clone();
            let child_token = shutdown.child_token();
            runner_tasks
                .push(tokio::spawn(async move { runner.run(parameters_clone, child_token).await }));
        }
        let results = join_all(runner_tasks).await;

        for result in results {
            match result {
                Ok(Ok(_)) => {
                    continue;
                },
                Ok(Err(e)) => {
                    return Err(e);
                },
                Err(e) => {
                    return Err(e.into());
                },
            }
        }
        Ok(())
    }
}
