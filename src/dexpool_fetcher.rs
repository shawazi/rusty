use serde_json::{to_string_pretty, Value};
use std::{error::Error, fs::File, io::Write};

use crate::DexPool;

pub async fn fetch_polygon_dex_pools(dex: &str) -> Result<Value, Box<dyn Error>> {
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

    fn extract_token_address(id: &str) -> &str {
        if let Some(pos) = id.rfind('_') {
            &id[pos + 1..]
        } else {
            id
        }
    }

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
