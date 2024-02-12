use serde_json::to_string_pretty;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use tokio::time::Duration;

pub async fn fetch_dex_data(network: &str) -> Result<Value, Box<dyn Error>> {
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
