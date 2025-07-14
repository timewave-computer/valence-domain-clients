#[cfg(feature = "evm")]
use std::str::FromStr;

#[cfg(feature = "evm")]
use alloy::{
    primitives::{Address, U256},
    transports::http::reqwest,
};
#[cfg(feature = "evm")]
use serde_json::Value;
#[cfg(feature = "evm")]
use async_trait::async_trait;

#[cfg(feature = "evm")]
use crate::indexer::{base_client::ValenceIndexerBaseClient, one_way_vault::OneWayVaultIndexer};

#[cfg(feature = "evm")]
#[derive(Debug, Default, Clone)]
pub struct OneWayVaultIndexerClient {
    api_key: String,
    indexer_url: String,
    vault_addr: String,
}

#[cfg(feature = "evm")]
impl OneWayVaultIndexerClient {
    pub fn new(indexer_url: &str, api_key: &str, vault_addr: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            indexer_url: indexer_url.to_string(),
            vault_addr: vault_addr.to_string(),
        }
    }
}

#[cfg(feature = "evm")]
#[async_trait]
impl ValenceIndexerBaseClient for OneWayVaultIndexerClient {
    fn get_api_key(&self) -> String {
        self.api_key.to_string()
    }

    fn get_indexer_url(&self) -> String {
        self.indexer_url.to_string()
    }

    fn get_request_client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[cfg(feature = "evm")]
#[async_trait]
impl OneWayVaultIndexer for OneWayVaultIndexerClient {
    fn get_vault_addr(&self) -> String {
        self.vault_addr.to_string()
    }

    /// queries the withdraw requests of a given vault.
    /// if `from_id` is set, only requests with id gte
    /// to the passed value will be fetched.
    /// requests are returned in ascending order.
    async fn query_vault_withdraw_requests(
        &self,
        from_id: Option<u64>,
        finalized: bool,
    ) -> anyhow::Result<Vec<(u64, Address, String, U256)>> {
        let indexer_url = self.get_indexer_url();
        let vault_addr = self.get_vault_addr();

        let mut query_flags = vec![];

        // if specified, only fetch requests starting from (incl.) id
        if let Some(id) = from_id {
            query_flags.push(format!("from={id}"));
        }

        // if enabled, only fetch requests from finalized blocks
        if finalized {
            query_flags.push("blockTag=finalized".to_string());
        }

        // fetch all requests in ascending order
        query_flags.push("order=asc".to_string());

        let query_params = query_flags.join("&");

        let query_url = format!("{indexer_url}/vault/{vault_addr}/withdrawRequests?{query_params}");

        let json_response: Value = self
            .get_request_client()
            .get(query_url)
            .header("Content-Type", "application/json")
            .header("X-Api-Key", self.get_api_key())
            .send()
            .await?
            .json()
            .await?;

        let mut resp_array = vec![];

        if let Some(requests) = json_response["data"].as_array() {
            for request in requests.iter() {
                // extract relevant fields from the response
                let request_id = match request.get("id").and_then(|v| v.as_u64()) {
                    Some(id) => id,
                    None => return Err(anyhow::anyhow!("Failed to parse request_id as u64")),
                };

                let owner_addr_str = match request.get("owner_address").and_then(|v| v.as_str()) {
                    Some(addr) => addr,
                    None => return Err(anyhow::anyhow!("Failed to get owner_address")),
                };

                let owner_addr = Address::from_str(owner_addr_str)?;

                let receiver_addr = match request.get("receiver_address").and_then(|v| v.as_str()) {
                    Some(addr) => addr.to_string(),
                    None => return Err(anyhow::anyhow!("Failed to get receiver_address")),
                };

                let amount_str = match request.get("amount").and_then(|v| v.as_str()) {
                    Some(amount) => amount,
                    None => return Err(anyhow::anyhow!("Failed to get amount")),
                };

                let amount = match U256::from_str(amount_str) {
                    Ok(amount) => amount,
                    Err(e) => return Err(anyhow::anyhow!("Failed to parse amount: {}", e)),
                };

                // Add the parsed data to the response array
                resp_array.push((request_id, owner_addr, receiver_addr, amount));
            }
        }

        Ok(resp_array)
    }
}

#[tokio::test]
async fn indexer_works() {
    let vault_addr = "-";
    let api_key = "-";
    let indexer_url = "-";

    let indexer_client = OneWayVaultIndexerClient::new(indexer_url, api_key, vault_addr);

    let resp = indexer_client
        .query_vault_withdraw_requests(Some(0), true)
        .await
        .unwrap();

    println!("resp: {:?}", resp);
}
