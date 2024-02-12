use env_file_reader::read_file;
use ethers::prelude::*;
use ethers::providers::{Provider, Ws};
use std::error::Error;

pub async fn init_provider() -> Result<Provider<Ws>, Box<dyn Error>> {
    let env_variables = read_file("./.env")?;
    let alchemy_api = &env_variables["alchemyAPIKey"];
    let alchemy_url = format!("wss://polygon-mainnet.g.alchemy.com/v2/{}", alchemy_api);
    let provider = Provider::<Ws>::connect(&alchemy_url).await?;
    println!("{}", alchemy_url);
    Ok(provider)
}
