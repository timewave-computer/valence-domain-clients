use alloy::primitives::{Address, U256};
use tonic::async_trait;

use super::base_client::ValenceIndexerBaseClient;

#[async_trait]
pub trait OneWayVaultIndexer: ValenceIndexerBaseClient {
    fn get_vault_addr(&self) -> String;

    async fn query_vault_withdraw_requests(
        &self,
        from_id: Option<u64>,
    ) -> anyhow::Result<Vec<(u64, Address, String, U256)>>;
}
