use env_file_reader::read_file;
use ethers::{
    contract::{abigen, Contract},
    core::types::ValueOrArray,
    providers::{Provider, StreamExt, Ws},
};
use std::{error::Error, sync::Arc};

abigen!(
    AggregatorInterface,
    r#"[
        event AnswerUpdated(int256 indexed current, uint256 indexed roundId, uint256 updatedAt)
    ]"#,
);

const PRICE_FEED_1: &str = "0x7de93682b9b5d80d45cd371f7a14f74d49b0914c";
const PRICE_FEED_2: &str = "0x0f00392fcb466c0e4e4310d81b941e07b4d5a079";
const PRICE_FEED_3: &str = "0xebf67ab8cff336d3f609127e8bbf8bd6dd93cd81";

/// Subscribe to a typed event stream without requiring a `Contract` instance.
/// In this example we subscribe Chainlink price feeds and filter out them
/// by address.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = get_client().await.unwrap();
    let client = Arc::new(client);

    // Build an Event by type. We are not tied to a contract instance. We use builder functions to
    // refine the event filter
    let event = Contract::event_of_type::<AnswerUpdatedFilter>(client)
        .from_block(16022082)
        .address(ValueOrArray::Array(vec![
            PRICE_FEED_1.parse()?,
            PRICE_FEED_2.parse()?,
            PRICE_FEED_3.parse()?,
        ]));

    let mut stream = event.subscribe_with_meta().await?.take(2);

    // Note that `log` has type AnswerUpdatedFilter
    while let Some(Ok((log, meta))) = stream.next().await {
        println!("{log:?}");
        println!("{meta:?}")
    }

    Ok(())
}

async fn get_client() -> Result<Provider<Ws>, Box<dyn Error>> {
    let env_variables = read_file("./.env")?;
    if let Some(alchemy_api) = env_variables.get("alchemyAPIKey") {
        let alchemy_url = format!("wss://polygon-mainnet.g.alchemy.com/v2/{}", alchemy_api);
        let provider = Provider::<Ws>::connect(&alchemy_url).await?;
        Ok(provider)
    } else {
        Err("alchemyAPIKey not found in .env file".into())
    }
}
