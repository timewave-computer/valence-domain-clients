use std::str::FromStr;

use alloy::{
    primitives::{Address, U256},
    transports::http::reqwest,
};
use serde_json::Value;
use tonic::async_trait;

use crate::indexer::{base_client::ValenceIndexerBaseClient, one_way_vault::OneWayVaultIndexer};

#[derive(Debug, Default, Clone)]
pub struct OneWayVaultIndexerClient {
    api_key: String,
    indexer_url: String,
    vault_addr: String,
}

impl OneWayVaultIndexerClient {
    pub fn new(indexer_url: &str, api_key: &str, vault_addr: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            indexer_url: indexer_url.to_string(),
            vault_addr: vault_addr.to_string(),
        }
    }
}

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
    ) -> anyhow::Result<Vec<(u64, Address, String, U256)>> {
        let indexer_url = self.get_indexer_url();
        let vault_addr = self.get_vault_addr();

        // if `from_id` is passed, we format it in the expected format.
        // otherwise default to empty tring
        let start_from = match from_id {
            Some(id) => format!("?from={id}"),
            None => "".to_string(),
        };

        let indexer_endpoint =
            format!("{indexer_url}/vault/{vault_addr}/withdrawRequests{start_from}&order=asc");

        let json_response: Value = self
            .get_request_client()
            .get(indexer_endpoint)
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

    indexer_client
        .query_vault_withdraw_requests(Some(1))
        .await
        .unwrap();
}
