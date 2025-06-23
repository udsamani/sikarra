use std::{sync::Arc, time::Duration};

use alloy::{
    primitives::{address, keccak256, Address, B256},
    providers::ProviderBuilder,
    transports::http::reqwest::Url,
};
use futures::StreamExt;
use sikkara_adapters::UniswapV4StateViewManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("https://mainnet.base.org")?;
    let provider = ProviderBuilder::new().connect_http(url);
    let address = address!("0xA3c0c9b65baD0b08107Aa264b0f3dB444b867A71");

    // This works because provider is cloneable
    let manager = UniswapV4StateViewManager::new(Arc::new(provider), address);

    let weth_address =
        Address::parse_checksummed("0x4200000000000000000000000000000000000006", None)?;
    let usdc_address =
        Address::parse_checksummed("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", None)?;

    let pool_id = compute_pool_id(weth_address, usdc_address, 500, 10, Address::ZERO)?;
    let invert = weth_address < usdc_address;
    println!("Invert: {}", invert);
    let mut stream = manager.watch_pool(pool_id, Duration::from_secs(30), invert);

    while let Some(pool_data) = stream.next().await {
        println!("Current Pool Data: {:?}", pool_data);
        println!("Spot Price: {}", pool_data.spot_price.to_fixed(2, None));
    }

    Ok(())
}

fn compute_pool_id(
    currency0: Address,
    currency1: Address,
    fee: u32,
    tick_spacing: i32,
    hook: Address,
) -> Result<B256, Box<dyn std::error::Error>> {
    // Ensure currency0 < currency1
    let (currency0, currency1) =
        if currency0 < currency1 { (currency0, currency1) } else { (currency1, currency0) };

    let encoded =
        alloy::sol_types::SolValue::abi_encode(&(currency0, currency1, fee, tick_spacing, hook));

    Ok(keccak256(encoded))
}
