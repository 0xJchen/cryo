// crates/freeze/src/event_hash.rs

use ethers_core::abi::Event;
use serde_json::Value;

pub fn calculate_topic_0() {
    let abi_json = r#"
    {
        anonymous: false,
        inputs: [
        {
        indexed: true,
        internalType: "address",
        name: "token0",
        type: "address"
        },
        {
        indexed: true,
        internalType: "address",
        name: "token1",
        type: "address"
        },
        {
        indexed: false,
        internalType: "address",
        name: "pair",
        type: "address"
        },
        {
        indexed: false,
        internalType: "uint256",
        name: "",
        type: "uint256"
        }
        ],
        name: "PairCreated",
        type: "event"
        }"#;

    let abi_value: Value = serde_json::from_str(abi_json).expect("Invalid JSON");
    let event: Event = serde_json::from_value(abi_value).expect("Invalid Event ABI");
    let topic_0 = event.signature(); // Assuming this is the hash
    println!("Topic 0: {:?}", topic_0);

    println!("Topic 0: {:?}", topic_0);
}
