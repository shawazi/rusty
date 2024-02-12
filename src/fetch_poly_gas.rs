use serde_json::Value;
use std::error::Error;

pub async fn fetch_polygon_gas_price() -> Result<(f64, f64), Box<dyn Error>> {
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
