pub async fn fetch_ethereum_price() -> Result<f64, Box<dyn std::error::Error>> {
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
