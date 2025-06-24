use std::{env, path::Path};

use sikkara_core::run;

// Internal module for the arbitrager application
mod collectors;
#[allow(unused)]
mod config;
#[allow(unused)]
mod engine;
#[allow(unused)]
mod runner;
#[allow(unused)]
mod strategy;

fn main() {
    // Read config path from environment variable, with a default fallback
    let config_path =
        env::var("ARBITRAGE_CONFIG_PATH").unwrap_or_else(|_| "config/arbitrage.json".to_string());

    // Check if config file exists
    if !Path::new(&config_path).exists() {
        eprintln!("Config file not found at: {}", config_path);
        eprintln!("Please set ARBITRAGE_CONFIG_PATH environment variable or ensure the default config file exists");
        std::process::exit(1);
    }
    let content =
        std::fs::read_to_string(&config_path).expect("Failed to read the configuration file");

    let params: config::ArbitrageConfig =
        serde_json::from_str(&content).expect("Failed to parse the configuration file");

    let runner = runner::ArbitrageRunner {};
    run(params, runner);
}
