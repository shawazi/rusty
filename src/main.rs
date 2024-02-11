#![allow(unused)]

mod abi_fetcher;

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

    let eth_price_usd = fetch_ethereum_price().await?;
    println!("ETH Price USD: {}", eth_price_usd);

    fetch_dex_data("polygon_pos").await?;

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
    fetch_polygon_dex_pools("uniswap_v3_polygon_pos").await?;
    // Print the vector to verify
    println!("{:?}", dex_ids);
    for dex in dex_ids {
        fetch_polygon_dex_pools(&dex).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    let (safe_gas, fast_gas) = fetch_polygon_gas_price().await?;
    Ok(())
}

fn calculate_slippage(
    amount_in: f64,   // Amount of input token you are trading
    reserve_in: f64,  // Reserve of the input token in the pool
    reserve_out: f64, // Reserve of the output token in the pool
) -> f64 {
    // Price without trade impact
    let price_initial = reserve_out / reserve_in;

    // New reserves after trade
    let new_reserve_in = reserve_in + amount_in;
    let new_reserve_out = reserve_in * reserve_out / new_reserve_in; // Derived from x * y = k

    // Price with trade impact
    let price_impact = new_reserve_out / new_reserve_in;

    // Slippage
    let slippage = (price_impact - price_initial) / price_initial;

    slippage.abs() * 100.0 // Return slippage as a percentage
}

async fn fetch_polygon_gas_price() -> Result<(f64, f64), Box<dyn Error>> {
    let url =
        "https://api.polygonscan.com/api?module=gastracker&action=gasoracle&apikey=HWMN6JFES5MQ376KDHAGHMECYIFD9AABNH";

    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let data: Value = response.json().await?;

        let safe_gas_price = data["result"]["SafeGasPrice"]
            .as_str()
            .ok_or("SafeGasPrice not found")?
            .parse::<f64>()
            .map_err(|_| "Failed to parse SafeGasPrice")?;

        let fast_gas_price = data["result"]["FastGasPrice"]
            .as_str()
            .ok_or("FastGasPrice not found")?
            .parse::<f64>()
            .map_err(|_| "Failed to parse FastGasPrice")?;

        Ok((safe_gas_price, fast_gas_price))
    } else {
        Err(format!("Failed to fetch gas price: {}", response.status()).into())
    }
}

async fn fetch_ethereum_price() -> Result<f64, Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";

    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await?;

        if let Some(ethereum_price) = data["ethereum"]["usd"].as_f64() {
            Ok(ethereum_price)
        } else {
            Err("Unable to retrieve Ethereum price from the API response.".into())
        }
    } else {
        Err(format!(
            "Request to CoinGecko API failed with status code {:?}",
            response.status()
        )
        .into())
    }
}

async fn fetch_dex_data(network: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.geckoterminal.com/api/v2/networks/{}/dexes",
        network
    );
    // Make the API call
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let data: Value = response.json().await?;
        let json_str = to_string_pretty(&data)?;
        let mut f =
            File::create(format!("./polygon/{}.json", network)).expect("Unable to create file");
        f.write_all(json_str.as_bytes())
            .expect("Unable to write data");

        // Introduce a 2-second delay before the next API call
        tokio::time::sleep(Duration::from_secs(2)).await;
    } else {
        eprintln!(
            "Error: Request to GeckoTerminal API for {} failed with status code {:?}",
            network,
            response.status()
        );
    }
    Ok(().into())
}

async fn fetch_polygon_dex_pools(dex: &str) -> Result<Value, Box<dyn Error>> {
    let url: String = format!(
        "https://api.geckoterminal.com/api/v2/networks/polygon_pos/dexes/{}/pools",
        dex
    );
    let response = reqwest::get(url).await?;
    let empty_vec = Vec::new();

    if response.status().is_success() {
        let data: Value = response.json().await?;

        let pools_array = data
            .get("data")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec);

        // Parse the data and create DexPool instances
        for pool_data in pools_array {
            let name = pool_data["attributes"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let pool_address: String = pool_data["attributes"]["address"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let (base_token, quote_token) = separate_token_names(&name);
            let base_token_address = pool_data["relationships"]["base_token"]["data"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let base_token_price_usd_str = pool_data["attributes"]["base_token_price_usd"]
                .as_str()
                .unwrap_or_default()
                .replace('\"', "");
            let base_token_price_usd: Option<f64> = base_token_price_usd_str.parse().ok();
            let base_token_price_native_str = pool_data["attributes"]
                ["base_token_price_native_currency"]
                .as_str()
                .unwrap_or_default()
                .replace('\"', "");
            let base_token_price_native = base_token_price_native_str.parse().ok();
            let quote_token_address = pool_data["relationships"]["quote_token"]["data"]["id"]
                .as_str()
                .unwrap_or_default()
                .to_string(); // Adjust this if it's incorrect
            let quote_token_price_usd_str = pool_data["attributes"]["quote_token_price_usd"]
                .as_str()
                .unwrap_or_default()
                .replace('\"', "");
            let quote_token_price_usd: Option<f64> = quote_token_price_usd_str.parse().ok();
            let quote_token_price_native_str = pool_data["attributes"]
                ["quote_token_price_native_currency"]
                .as_str()
                .unwrap_or_default()
                .replace('\"', "");
            let quote_token_price_native: Option<f64> = quote_token_price_native_str.parse().ok();
            let reserve_usd_str = pool_data["attributes"]["reserve_in_usd"]
                .as_str()
                .unwrap_or_default()
                .replace('\"', ""); // Remove escaped quotes
            let reserve_usd: Option<f64> = reserve_usd_str.parse().ok();
            let dex_pool = DexPool {
                name: format!("{}_{}", dex, name),
                pool_address,
                base_token,
                base_token_address: extract_token_address(&base_token_address).to_string(),
                base_token_price_usd: base_token_price_usd.unwrap_or(0.0666),
                base_token_price_native: base_token_price_native.unwrap_or(0.0666),
                quote_token,
                quote_token_address: extract_token_address(&quote_token_address).to_string(),
                quote_token_price_usd: quote_token_price_usd.unwrap_or(0.0666),
                quote_token_price_native: quote_token_price_native.unwrap_or(0.0666),
                reserve_usd: reserve_usd.unwrap_or(0.666),
            };
            println!("{:?}", &dex_pool);
        }
        let json_str = to_string_pretty(&data)?;
        let mut f = File::create(format!("./polygon/{}.json", dex)).expect("Unable to create file");
        f.write_all(json_str.as_bytes())
            .expect("Unable to write data");
    } else {
        eprintln!(
            "Error: Request to GeckoTerminal API for {} failed with status code {:?}",
            dex,
            response.status()
        )
    }
    Ok(().into())
}

fn extract_token_address(id: &str) -> &str {
    if let Some(pos) = id.rfind('_') {
        &id[pos + 1..]
    } else {
        id
    }
}

fn separate_token_names(pool_name: &str) -> (String, String) {
    let tokens: Vec<&str> = pool_name.split(" / ").collect();
    if tokens.len() == 2 {
        let base_token_name = tokens[0].trim().to_uppercase();
        let quote_token_name = tokens[1].trim().to_uppercase();
        (base_token_name, quote_token_name)
    } else {
        (String::new(), String::new())
    }
}
