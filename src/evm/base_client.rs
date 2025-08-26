use std::str::FromStr;

use alloy::contract::{CallBuilder, CallDecoder};
use alloy::network::{Ethereum, TransactionBuilder};
use alloy::network::{EthereumWallet, Network};
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{
    fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
    Identity, RootProvider,
};
use alloy::transports::Transport;

use alloy::providers::Provider;
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::transports::http::{Client, Http};
use tonic::async_trait;

use super::request_provider_client::RequestProviderClient;

pub type CustomProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

pub trait EvmQueryRequest: Clone {
    /// decoded output type for this query
    type Output;

    /// every query request must be convertible to a transaction request
    fn get_tx_request(&self) -> TransactionRequest;

    /// decode the raw bytes of the EVM call into the output type
    fn decode_response(&self, bytes: Bytes) -> anyhow::Result<Self::Output>;
}

// this is a bit loaded but I couldn't find a way around it if we want to
// keep the query devex clean and avoid decoding the responses manually
impl<T, P, D, N> EvmQueryRequest for CallBuilder<T, P, D, N>
where
    T: Transport + Clone + Send + Sync,
    P: Provider<T, N> + Send + Sync,
    D: CallDecoder,
    D::CallOutput: Send + Sync,
    N::TransactionRequest: Into<TransactionRequest>,
    N: Network,
    CallBuilder<T, P, D, N>: Clone,
{
    type Output = D::CallOutput;

    fn get_tx_request(&self) -> TransactionRequest {
        self.clone().into_transaction_request().into()
    }

    fn decode_response(&self, raw: Bytes) -> anyhow::Result<Self::Output> {
        let resp = self.decode_output(raw, true)?;
        Ok(resp)
    }
}

/// base client trait with default implementations for evm based clients.
///
/// for chains which are somehow unique in their common module implementations,
/// these function definitions can be overridden to match the custom chain logic.
#[async_trait]
pub trait EvmBaseClient: RequestProviderClient {
    async fn latest_block_height(&self) -> anyhow::Result<u64> {
        let client = self.get_request_provider().await?;

        let block = client.get_block_number().await?;

        Ok(block)
    }

    async fn query_balance(&self, address: &str) -> anyhow::Result<U256> {
        let client = self.get_request_provider().await?;

        let addr = Address::from_str(address)?;
        let balance = client.get_balance(addr).await?;

        Ok(balance)
    }

    async fn execute_tx(&self, tx: TransactionRequest) -> anyhow::Result<TransactionReceipt> {
        let client = self.get_request_provider().await?;

        let signed_tx = tx.from(self.signer().address());

        let tx_response = client
            .send_transaction(signed_tx)
            .await?
            .get_receipt()
            .await?;

        Ok(tx_response)
    }

    async fn sign_and_send(&self, tx: TransactionRequest) -> anyhow::Result<TransactionReceipt> {
        let wallet = EthereumWallet::from(self.signer());
        let rp = self.get_request_provider().await?;

        // Get current nonce
        let nonce = rp.get_transaction_count(self.signer().address()).await?;
        // Add nonce and sender to transaction request
        let tx_with_nonce_and_sender = tx.nonce(nonce).from(self.signer().address());
        // Fill the transaction request with all other necessary information from request provider
        let tx_request = match rp.fill(tx_with_nonce_and_sender).await?.as_builder() {
            Some(tx_request) => tx_request.clone(),
            None => {
                return Err(anyhow::anyhow!("Failed to fill transaction request"));
            }
        };
        // Sign the transaction
        let tx_envelope = tx_request
            .build(&wallet)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;
        // Send the transaction
        let tx_hash = rp
            .send_tx_envelope(tx_envelope)
            .await?
            .get_receipt()
            .await?;

        Ok(tx_hash)
    }

    async fn query<Q: EvmQueryRequest + Send>(&self, builder: Q) -> anyhow::Result<Q::Output> {
        let client = self.get_request_provider().await?;

        let tx_request: TransactionRequest = builder.get_tx_request();

        let raw_response = client.call(&tx_request).await?;

        let decoded = builder.decode_response(raw_response)?;

        Ok(decoded)
    }

    async fn blocking_query<Q, F>(
        &self,
        builder: Q,   // query definition
        predicate: F, // assertion fn on the response type
        interval_sec: u64,
        max_attempts: u32,
    ) -> anyhow::Result<Q::Output>
    where
        Q: EvmQueryRequest + Send,
        F: Fn(&Q::Output) -> bool + Send,
    {
        let client = self.get_request_provider().await?;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_sec));
        let tx_request = builder.get_tx_request();

        for attempt in 1..max_attempts + 1 {
            interval.tick().await;

            match client.call(&tx_request).await {
                Ok(raw_response) => {
                    let decoded = builder.decode_response(raw_response)?;
                    if predicate(&decoded) {
                        log::info!("query attempt {attempt}/{max_attempts}: condition met");
                        return Ok(decoded);
                    }
                    log::info!("query attempt {attempt}/{max_attempts}: condition not met");
                }
                Err(e) => {
                    log::warn!("query attempt {attempt}/{max_attempts} failed: {e:?}");
                }
            }
        }

        Err(anyhow::anyhow!(
            "blocking query failed after max attempts; condition not met"
        ))
    }
}
