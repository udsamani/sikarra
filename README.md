# Sikarra


A sophisticated Rust based arbitrage and marjet making system that monitors decentralized exhange (DEX) pools and centralized exchange (CEX) price feeds to identify tradin opportunities on **Base** Network.

## 🚀 Overview

The system is based on an even driven architecture.



![](./docs/images/image.png)

## Core Components
- `sikkara-adapters`: Exchange adapters for CEX (Coinbase WebSocket) and DEX (Uniswap V4) integration
- `sikkara-bot`: Main bot logic
- `sikkara-core`: Shared utilities, engine framework, and common types
- `sikkara-wsclient`: WebSocket client infrastructure with reconnection and heartbeat

## 🛠️ Features

### Price Monitoring
- **Real-time CEX Data**: WebSocket connection to Coinbase Pro for live ETH/USDC pricing
- **On-chain DEX Data**: Direct integration with Uniswap V4 pool contracts on Base
- **High-precision Calculations**: Uses 512-bit decimal arithmetic for financial accuracy

### Arbitrage Detection
- **Cross-venue Price Comparison**: Identifies price discrepancies between CEX and DEX
- **Configurable Thresholds**: Set minimum profit percentages and absolute amounts
- **Directional Analysis**: Determines optimal buy/sell venue combinations

## Market Making Simulation
- Fair Value Reference: Uses CEX prices as baseline for range calculations
- Dynamic Spread Management: Adjusts spreads based on market volatility and conditions

## 🚦 Getting Started

### Prerequisites

- Rust 1.70+ with Cargo
- Access to Base network RPC endpoint
- Coinbase Pro WebSocket access

### Installation

```bash
# Install Cargo Maker For Easy
cargo install cargo-make

# Build the project
cargo make build-release

# Run tests
cargo test
```


### Running

```bash
# Start the bot
cargo run --bin sikarra-bot


# Enable debug logging
RUST_LOG=debug cargo run --bin sikarra-bot
```

- A default configuration is provided in `config` folder.
- It subscribes to `ETH-USDC` trading pair.
- Please note that events from CEX and DEX are only logged in debug mode. So if interested in seeing those events please turn on the DEBUG logs, as shown above



