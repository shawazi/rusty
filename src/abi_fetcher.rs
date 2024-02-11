use env_file_reader::read_file;
use reqwest;
use serde_json::Value;

pub async fn fetch_abi(contract_address: &str) -> Result<String, Box<dyn std::error::Error>> {
    let env_variables = read_file("./.env")?;
    let api_key = &env_variables["polygonScanAPI"];
    println!("API KEY: {}", &api_key);
    let url = format!(
        "https://api.polygonscan.com/api?module=contract&action=getabi&address={}&apikey={}",
        contract_address, api_key
    );

    let response = reqwest::get(&url).await?;
    let data: Value = response.json().await?;

    if data["status"] == "1" {
        if let Some(abi) = data["result"].as_str() {
            Ok(abi.to_string())
        } else {
            Err("ABI not found in response".into())
        }
    } else {
        Err(format!("Error fetching ABI: {:?}", data["result"]).into())
    }
}

// Testing the function in a standalone manner
#[tokio::main]
async fn main() {
    let contract_address = "0xa374094527e1673a86de625aa59517c5de346d32";

    match fetch_abi(contract_address).await {
        Ok(abi) => println!("ABI: {}", abi),
        Err(e) => println!("Error: {}", e),
    }
}
