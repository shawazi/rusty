use reqwest;
// use reqwest::Error;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use tokio::time::Duration;

// Structure to hold the price data
// Outer HashMap key: Token pair (e.g., "ETH/USDC")
// Inner HashMap key: DEX name, value: Price
type PriceData = HashMap<String, HashMap<String, f64>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Fetch current Ethereum price from CoinGecko API

    let eth_price_usd = fetch_ethereum_price().await?;

    // transitional data storage
    // let mut network_data: Vec<Value> = Vec::new();
    // let mut arbitrumDex_data: Vec<Value> = Vec::new();
    // let mut polygonDex_data: Vec<Value> = Vec::new();
    // let mut avaxDex_data: Vec<Value> = Vec::new();
    // let mut optimismDex_data: Vec<Value> = Vec::new();

    // Define the list of networks
    // let networks = vec!["polygon_pos", "avax", "arbitrum", "optimism"];

    // Loop through each network and make the API call
    // for network in networks {
    // Construct the URL for the GeckoTerminal API with the network parameter
    // fetch_dex_data(network).await?;
    // }

    // Read the JSON file
    let data = fs::read_to_string("polygonPOS.json")?;

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

    println!("ETH Price USD: {}", eth_price_usd);

    // Print the vector to verify
    println!("{:?}", dex_ids);

    for dex in dex_ids {
        fetch_polygon_dex_pools(&dex).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let (safe_gas, fast_gas) = fetch_polygon_gas_price().await?;

    Ok(())
}

fn populate_price_data(/* parameters to fetch data */) -> PriceData {
    let mut price_data = PriceData::new();

    // Logic to populate price_data
    // You would loop through your DEX data, parse out the token pairs and prices,
    // and insert them into the price_data HashMap

    price_data
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

// async fn fetch_dex_data(network: &str) -> Result<Value, Box<dyn Error>> {
//     let url = format!(
//         "https://api.geckoterminal.com/api/v2/networks/{}/dexes",
//         network
//     );

//     // Make the API call
//     let response = reqwest::get(url).await?;

//     if response.status().is_success() {
//         // Parse the response JSON
//         let data: Value = response.json().await?;

//         // store in transitional data
//         // network_data.push(data.clone());

//         // Process the data as needed
//         println!("Network: {}", network);
//         println!("{:#?}", data);

//         // Introduce a 2-second delay before the next API call
//         tokio::time::sleep(Duration::from_secs(2)).await;
//     } else {
//         eprintln!(
//             "Error: Request to GeckoTerminal API for {} failed with status code {:?}",
//             network,
//             response.status()
//         );
//     }
//     Ok(().into())
// }

async fn fetch_polygon_dex_pools(dex: &str) -> Result<Value, Box<dyn Error>> {
    let url: String = format!(
        "https://api.geckoterminal.com/api/v2/networks/polygon_pos/dexes/{}/pools",
        dex
    );

    let directory_path = "./polygon";
    fs::create_dir_all(directory_path)?;

    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let data: Value = response.json().await?;
        println!("Dex: {}", dex);
        println!("{:#?}", data);

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
