use tonic::{
    async_trait,
    transport::{Channel, ClientTlsConfig},
};

use crate::cosmos::{
    base_client::BaseClient, grpc_client::GrpcSigningClient, wasm_client::WasmClient,
};

const CHAIN_NAME: &str = "lombardledger";
const CHAIN_PREFIX: &str = "lom";
const CHAIN_DENOM: &str = "ulom";

pub struct LombardClient {
    grpc_url: String,
    mnemonic: String,
    chain_id: String,
    chain_denom: String,
    gas_price: f64,
}

impl LombardClient {
    pub async fn new(
        rpc_url: &str,
        rpc_port: &str,
        mnemonic: &str,
        chain_id: &str,
    ) -> anyhow::Result<Self> {
        let avg_gas_price = Self::query_chain_gas_config(CHAIN_NAME, CHAIN_DENOM).await?;

        Ok(Self {
            grpc_url: format!("{rpc_url}:{rpc_port}"),
            mnemonic: mnemonic.to_string(),
            chain_id: chain_id.to_string(),
            chain_denom: CHAIN_DENOM.to_string(),
            gas_price: avg_gas_price,
        })
    }
}

#[async_trait]
impl BaseClient for LombardClient {}

/// permissioned-cw
#[async_trait]
impl WasmClient for LombardClient {}

#[async_trait]
impl GrpcSigningClient for LombardClient {
    // overriding the default grpc channel getter to use system certs for tls
    async fn get_grpc_channel(&self) -> anyhow::Result<Channel> {
        let channel = Channel::from_shared(self.grpc_url())
            .map_err(|_| anyhow::anyhow!("failed to build grpc channel"))?
            // using the system certs
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await?;

        Ok(channel)
    }

    fn grpc_url(&self) -> String {
        self.grpc_url.to_string()
    }

    fn mnemonic(&self) -> String {
        self.mnemonic.to_string()
    }

    fn chain_prefix(&self) -> String {
        CHAIN_PREFIX.to_string()
    }

    fn chain_id(&self) -> String {
        self.chain_id.to_string()
    }

    fn chain_denom(&self) -> String {
        self.chain_denom.to_string()
    }

    fn gas_price(&self) -> f64 {
        self.gas_price
    }

    fn gas_adjustment(&self) -> f64 {
        1.8
    }
}
