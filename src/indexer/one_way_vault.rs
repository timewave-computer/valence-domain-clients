use serde_json::Value;
use tonic::async_trait;

use super::base_client::ValenceIndexerBaseClient;


#[async_trait]
pub trait OneWayVaultIndexer: ValenceIndexerBaseClient {
    fn get_vault_addr(&self) -> String;

    async fn query_vault_withdraw_requests(&self) -> anyhow::Result<Value>;
}
