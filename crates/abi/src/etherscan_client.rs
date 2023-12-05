use reqwest;
use serde_json::{self, Value};
use ethers_core::abi::Event;
use std::error::Error;


pub struct EtherscanClient {
    base_url: String,
    api_key: String,
}

impl EtherscanClient {
    pub fn new(api_key: String) -> Self {
        EtherscanClient {
            base_url: "https://api.etherscan.io/api".to_string(),
            api_key,
        }
    }

    pub async fn get_abi(&self, contract_address: &str) -> Result<Vec<Event>, Box<dyn Error>> {
        let url = format!(
            "{}?module=contract&action=getabi&address={}&format=raw&apikey={}",
            self.base_url, contract_address, self.api_key
        );

        let resp = reqwest::get(&url).await?.text().await?;
        let abi_value: Value = serde_json::from_str(&resp)?;

        // Add error handling for the specific API response
        if let Value::Object(obj) = &abi_value {
            if let Some(Value::String(status)) = obj.get("status") {
                if status == "0" {
                    return Err("Invalid contract address".into());
                }
            }
        }


        let mut events = Vec::new();
        if let Value::Array(abis) = abi_value {
            for abi in abis {
                if let Value::Object(obj) = abi {
                    if let Some(Value::String(type_str)) = obj.get("type") {
                        if type_str == "event" {
                            if let Ok(event) = serde_json::from_value::<Event>(Value::Object(obj.clone())) {
                                events.push(event);
                            }
                        }
                    }
                }
            }
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    // use mockito::{mock, server_url};


    #[tokio::test]
    async fn fetch_abi_with_api_key() {
        env::set_var("apikey", "sample_api_key");
        let api_key = env::var("apikey").unwrap();

        let client = EtherscanClient::new(api_key);
        // Replace with a valid test address
        let result = client.get_abi("0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f").await;
        assert!(result.is_ok());

        env::remove_var("apikey");
    }

    #[tokio::test]
    async fn fetch_abi_without_api_key() {
        env::remove_var("apikey");

        let api_key = match env::var("apikey") {
            Ok(key) => key,
            Err(_) => "".to_string(),
        };

        let client = EtherscanClient::new(api_key);
        let result = client.get_abi("valid_test_address").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_abi_valid_address() {
        // Mock or set an API key for testing
        let api_key = "sample_api_key".to_string();
    
        // Initialize EtherscanClient with the API key
        let client = EtherscanClient::new(api_key);
    
        // Use a mock address or a testing API
        let result = client.get_abi("0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f").await;
    
        // Check for the result and print error if there is one
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
    
        // Assert that the result is Ok
        assert!(result.is_ok());
        // Additional assertions can be added based on expected behavior
    }

    #[tokio::test]
    async fn test_get_abi_valid_address_2() {
        // Mock or set an API key for testing
        let api_key = "sample_api_key".to_string();
    
        // Initialize EtherscanClient with the API key
        let client = EtherscanClient::new(api_key);
    
        // Use a mock address or a testing API
        let result = client.get_abi("0x388c818ca8b9251b393131c08a736a67ccb19297").await;
    
        // Check for the result and print error if there is one
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
    
        // Assert that the result is Ok
        assert!(result.is_ok());
        // Additional assertions can be added based on expected behavior
    }



    #[tokio::test]
    async fn test_get_abi_invalid_address() {
        // Mock or set an API key for testing
        let api_key = "sample_api_key".to_string();
    
        // Initialize EtherscanClient with the API key
        let client = EtherscanClient::new(api_key);
    
        // Use a mock address or a testing API
        let result = client.get_abi("0x5c69bee701ef814a2b6a3edd4b16").await;
    
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_err());
        // Check for specific error messages if applicable
    }


    #[tokio::test]
    async fn test_get_abi_invalid_address_2() {
        // Mock or set an API key for testing
        let api_key = "sample_api_key".to_string();
    
        // Initialize EtherscanClient with the API key
        let client = EtherscanClient::new(api_key);
    
        // Use a mock address or a testing API
        let result = client.get_abi("0x388c818ca").await;
    
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_err());
        // Check for specific error messages if applicable
    }
        // Additional assertions can be added based on expected behavior
    }

    // #[tokio::test]
    // async fn test_get_abi_valid_address() {
    //     let client = EtherscanClient::new();
    //     // Use a mock address or a testing API
    //     let result = client.get_abi("0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f").await;
    //     if let Err(e) = &result {
    //         println!("Error: {:?}", e);
    //     }
    //     assert!(result.is_ok());
    //     // Additional assertions can be added based on expected behavior
    // }

    // async fn test_get_abi_valid_address_2() {
    //     let client = EtherscanClient::new();
    //     // Use a mock address or a testing API
    //     let result = client.get_abi("0x388c818ca8b9251b393131c08a736a67ccb19297").await;
    //     if let Err(e) = &result {
    //         println!("Error: {:?}", e);
    //     }
    //     assert!(result.is_ok());
    //     // Additional assertions can be added based on expected behavior
    // }

    // async fn test_get_abi_invalid_address() {
    //     let client = EtherscanClient::new();
    //     let result = client.get_abi("0x5c69bee701ef8d4b1652").await;
    //     if let Err(e) = &result {
    //         println!("Error: {:?}", e);
    //     }
    //     assert!(result.is_err());
    //     // Check for specific error messages if applicable
    // }

    // async fn test_get_abi_invalid_address_2() {
    //     let client = EtherscanClient::new();
    //     let result = client.get_abi("0x388c818ca").await;
    //     if let Err(e) = &result {
    //         println!("Error: {:?}", e);
    //     }
    //     assert!(result.is_err());
    //     // Check for specific error messages if applicable
    // }

