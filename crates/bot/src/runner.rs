use std::sync::Arc;

use alloy::{
    contract,
    primitives::Address,
    providers::ProviderBuilder,
    transports::{http::reqwest::Url, ws},
};
use futures::future::join_all;
use sikkara_adapters::{CoinbaseWsClient, UniswapV4StateViewManager};
use sikkara_core::{AppResult, Collector, EngineRunner, ExponentialBackoff, Runner};
use sikkara_wsclient::WsConsumer;
use tokio::sync::{broadcast, mpsc};

use crate::{
    collectors::{PoolFeedCollector, PriceFeedCollector},
    config::{BotConfig, CexConfig, PoolConfig},
    engine::{ArbitrageEngine, InternalAction, InternalEvent, Pool},
    strategy::LoggingBotStrategy,
};

#[derive(Debug, Clone)]
pub struct BotRunner {}

#[async_trait::async_trait]
impl Runner<BotConfig> for BotRunner {
    fn name(&self) -> &str { "arbitrage_runner" }

    // TODO: Break this funtion down. Hard to read.
    async fn run(
        self,
        parameters: BotConfig,
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

            // Setup the engine
            let engine = ArbitrageEngine::new(
                LoggingBotStrategy::new(pool.symbol_owned()),
                pool.symbol().to_string(),
            );
            runner.add_engine(Box::new(engine));

            // Setup the price feed collector
            let price_feed_collector = PriceFeedCollector::new(pool.symbol_owned(), client.clone());
            runner.add_collector(Box::new(price_feed_collector));

            // Setup the pool feed collector
            let PoolConfig::UniswapV4 {
                address,
                symbol,
                token_0,
                token_1,
                fee_tier,
                node_url,
                hook_address,
                tick_spacing,
                scaling,
            } = pool;
            let url = Url::parse(node_url).expect("Invalid node URL");
            let provider = ProviderBuilder::new().connect_http(url);
            let contract_address =
                Address::parse_checksummed(address, None).expect("Invalid contract address");
            let state_manager =
                UniswapV4StateViewManager::new(Arc::new(provider), contract_address);

            let hook = if hook_address.is_none() {
                Address::ZERO
            } else {
                Address::parse_checksummed(hook_address.as_ref().unwrap(), None)
                    .expect("Invalid hook address")
            };
            let pool = Pool {
                symbol: symbol.clone(),
                token_0: token_0.into(),
                token_1: token_1.into(),
                fee_tier: *fee_tier,
                tick_spacing: *tick_spacing,
                hook,
                scaling: *scaling,
            };
            let pool_feed_collector = PoolFeedCollector::new(pool, state_manager);
            runner.add_collector(Box::new(pool_feed_collector));

            // Run all tasks
            let parameters_clone = parameters.clone();
            let child_token = shutdown.child_token();
            runner_tasks
                .push(tokio::spawn(async move { runner.run(parameters_clone, child_token).await }));
        }

        // Wait for all tasks to complete
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
