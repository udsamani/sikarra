use std::time::Duration;

use tokio::{
    signal::unix::{signal, SignalKind},
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::prelude::*;

use crate::{AppResult, Runner};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
const ENV_LOG_LEVEL: &str = "LOG_LEVEL";
const ENV_LOG_JSON: &str = "LOG_JSON";

pub fn run<P, R>(params: P, runner: R)
where
    R: Runner<P> + Send,
{
    // Setup telemetry for logging
    setup_telemetry().expect("Failed to setup telemetry");

    // Initialize the runtime environment
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the provided runner with the given parameters
    runtime
        .block_on(async {
            let shutdown = CancellationToken::new();
            let mut run = runner.run(params, shutdown.child_token());

            // Handle shutdown signals
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigint = signal(SignalKind::interrupt())?;
            tokio::select! {
                res = run.as_mut() => match res {
                        Ok(_) => info!("runner exited cleanly"),
                        Err(e) => error!("runner terminated with error: {:#}", e),
                    },
                _ = sigterm.recv() => (),
                _ = sigint.recv() => (),
            }
            info!("shutting down...");
            shutdown.cancel();

            // Wait for actual shutdown or timeout
            match timeout(SHUTDOWN_TIMEOUT, run.as_mut()).await {
                Ok(Ok(_)) => info!("runner exited cleanly"),
                Ok(Err(e)) => error!("runner terminated with error: {:#}", e),
                Err(_) => {
                    error!(
                        "runner failed to shutdown gracefully after {:?}, aborting",
                        SHUTDOWN_TIMEOUT
                    );
                },
            }

            AppResult::<()>::Ok(())
        })
        .expect("runtime failed to run");
}

fn setup_telemetry() -> AppResult<()> {
    let log_level = std::env::var(ENV_LOG_LEVEL).unwrap_or_else(|_| "info".to_string());
    let log_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(log_level.parse()?)
        .from_env_lossy();

    // Two versions of log formatter have different implementations, so to eliminate
    // boxing (and corresponding performance penalty as it would be on a logging
    // hot path) the registration code is duplicated for each of the branches
    if !std::env::var(ENV_LOG_JSON).unwrap_or_default().is_empty() {
        let log_formatter = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_thread_names(true)
            .with_thread_ids(true);
        tracing_subscriber::registry()
            .with(log_filter)
            .with(log_formatter)
            .init();
    } else {
        let log_formatter = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_thread_ids(true);
        tracing_subscriber::registry()
            .with(log_filter)
            .with(log_formatter)
            .init();
    };
    Ok(())
}
