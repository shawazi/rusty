mod abi_fetcher;
mod calc_slip;
mod dex_fetcher;
mod dexpool_fetcher;
mod ethers_provider;
mod fetch_eth_price;
mod fetch_poly_gas;

use ethers_provider::init_provider;
use reqwest;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::empty;
use std::io::Write;
use tokio::time::Duration;

#[derive(Debug)]
struct Dex {
    name: String,
    id: String,
}

#[derive(Debug)]
struct DexPool {
    name: String,
    pool_address: String,
    base_token: String,
    base_token_address: String,
    base_token_price_usd: f64,
    base_token_price_native: f64,
    quote_token: String,
    quote_token_address: String,
    quote_token_price_usd: f64,
    quote_token_price_native: f64,
    reserve_usd: f64,
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    dex1: Dex,
    dex2: Dex,
    profit_percentage: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let directory_path = "./polygon";
    fs::create_dir_all(directory_path)?;

    let eth_price_usd = fetch_eth_price::fetch_ethereum_price().await?;
    println!("ETH Price USD: {}", eth_price_usd);

    dex_fetcher::fetch_dex_data("polygon_pos").await?;

    // Read the JSON file
    let data = fs::read_to_string("./polygon/polygon_pos.json")?;

    // Parse the JSON data
    let json: Value = serde_json::from_str(&data)?;

    // Initialize a vector to hold the IDs
    let mut dex_ids: Vec<String> = Vec::new();

    // Check if 'data' is an array and iterate through it
    if let Some(array) = json["data"].as_array() {
        for item in array {
            // Extract the 'id' field and add it to the vector
            if let Some(id) = item["id"].as_str() {
                dex_ids.push(id.to_string());
            }
        }
    }
    dexpool_fetcher::fetch_polygon_dex_pools("uniswap_v3_polygon_pos").await?;
    // Print the vector to verify
    println!("{:?}", dex_ids);
    for dex in dex_ids {
        dexpool_fetcher::fetch_polygon_dex_pools(&dex).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let (safe_gas, fast_gas) = fetch_poly_gas::fetch_polygon_gas_price().await?;

    let provider = init_provider();

    // Calculate arbitrage opportunities for the

    Ok(())
}
