use alloy::transports::http::reqwest;
use serde_json::Value;
use tonic::async_trait;

use crate::indexer::{base_client::ValenceIndexerBaseClient, one_way_vault::OneWayVaultIndexer};


#[derive(Debug, Default, Clone)]
pub struct OneWayVaultIndexerClient {
    pub api_key: String,
    pub indexer_url: String,
    pub vault_addr: String,
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

    async fn query_vault_withdraw_requests(&self) -> anyhow::Result<Value> {
        let indexer_url = self.get_indexer_url();
        let vault_addr = self.get_vault_addr();

        let indexer_endpoint = format!(
            "{indexer_url}/vault/{vault_addr}/withdrawRequests",
        );

        let resp = self.get_request_client()
            .get(indexer_endpoint)
            .header("Content-Type", "application/json")
            .header("X-Api-Key", "valence_team_api_key")
            .send()
            .await?;

        println!("raw response: {:?}", resp);

        let json_response: Value = resp.json().await?;

        println!("JSON parsed: {}", serde_json::to_string_pretty(&json_response)?);

        if let Some(requests) = json_response.as_array() {
            println!("withdraw requests len: {}", requests.len());

            for (i, request) in requests.iter().enumerate() {
                println!("withdraw request #{}: {}", i, serde_json::to_string_pretty(request)?);
            }
        }

        Ok(Value::default())
    }
}
