pub mod etherscan_client;
pub mod event_selector;

use std::env;
use etherscan_client::EtherscanClient;
use event_selector::EventSelector;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => println!("Execution completed successfully."),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Please provide a contract address".into());
    }
    let contract_address = &args[1];

    let api_key = env::var("apikey").map_err(|_| "API key not set in environment".to_string())?;

    let client = EtherscanClient::new(api_key);
    let events = client.get_abi(contract_address).await.map_err(|e| e.to_string())?;


    let event = EventSelector::select_event(&events)?;
    let topic_0 = event.signature();
    println!("Selected Event: {}, Topic 0: {:?}", event.name, topic_0);

    Ok(())
}
