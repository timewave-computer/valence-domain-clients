use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    transports::http::reqwest,
};
use alloy_signer_local::PrivateKeySigner;
use async_trait::async_trait;

use super::base_client::CustomProvider;

/// trait for evm-based clients to enable signing and request provider functionality.
/// each implementation must provide getters for the rpc url and signer which are used
/// to build the provider and sign transactions.
#[async_trait]
pub trait RequestProviderClient {
    fn rpc_url(&self) -> String;
    fn signer(&self) -> PrivateKeySigner;

    async fn get_request_provider(&self) -> anyhow::Result<CustomProvider> {
        let url: reqwest::Url = self
            .rpc_url()
            .parse()
            .map_err(|_| anyhow::anyhow!("failed to parse url"))?;

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        Ok(provider)
    }

    async fn get_provider_accounts(&self) -> anyhow::Result<Vec<Address>> {
        let provider = self.get_request_provider().await?;
        let accounts = provider.get_accounts().await?;
        Ok(accounts)
    }
}
