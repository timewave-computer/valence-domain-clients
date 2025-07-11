use crate::evm::base_client::EvmBaseClient;
use crate::evm::request_provider_client::RequestProviderClient;

use alloy_signer_local::coins_bip39::English;
use alloy_signer_local::{MnemonicBuilder, PrivateKeySigner};
use tonic::async_trait;

pub struct EthereumClient {
    rpc_url: String,
    signer: PrivateKeySigner,
}

impl EthereumClient {
    pub fn new(
        rpc_url: &str,
        mnemonic: &str,
        mnemonic_derivation_index: Option<u32>,
    ) -> anyhow::Result<Self> {
        let builder = MnemonicBuilder::<English>::default().phrase(mnemonic);

        let derivation_index = mnemonic_derivation_index.unwrap_or_default();

        let signer = builder.index(derivation_index)?.build()?;

        Ok(Self {
            rpc_url: rpc_url.to_string(),
            signer,
        })
    }
}

#[async_trait]
impl EvmBaseClient for EthereumClient {}

#[async_trait]
impl RequestProviderClient for EthereumClient {
    fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    fn signer(&self) -> PrivateKeySigner {
        self.signer.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{
        network::TransactionBuilder,
        primitives::{Address, U256},
        providers::Provider,
        rpc::types::TransactionRequest,
    };

    use crate::evm::testing::solidity_contracts::MockERC20;

    use super::*;

    // These would be replaced with actual test values
    const TEST_RPC_URL: &str = "http://127.0.0.1:8545";
    const TEST_MNEMONIC: &str = "test test test test test test test test test test test junk";
    const TEST_CONTRACT_ADDR: &str = "0x610178dA211FEF7D417bC0e6FeD39F05609AD788";

    #[tokio::test]
    #[ignore = "requires local anvil instance"]
    async fn test_eth_latest_block_height() {
        let client = EthereumClient::new(TEST_RPC_URL, TEST_MNEMONIC, None).unwrap();

        let block_number = client.latest_block_height().await.unwrap();
        assert_ne!(block_number, 0);
    }

    #[tokio::test]
    #[ignore = "requires local anvil instance"]
    async fn test_eth_query_balance() {
        let client = EthereumClient::new(TEST_RPC_URL, TEST_MNEMONIC, None).unwrap();
        let accounts = client.get_provider_accounts().await.unwrap();

        let balance = client
            .query_balance(&accounts[0].to_string())
            .await
            .unwrap();

        assert_ne!(balance, U256::from(0));
    }

    #[tokio::test]
    #[ignore = "requires local anvil instance"]
    async fn test_eth_transfer() {
        let client = EthereumClient::new(TEST_RPC_URL, TEST_MNEMONIC, None).unwrap();
        let accounts = client.get_provider_accounts().await.unwrap();

        let pre_balance = client
            .query_balance(&accounts[1].to_string())
            .await
            .unwrap();

        let transfer_request = TransactionRequest::default()
            .with_to(accounts[1])
            .with_value(U256::from(200));

        client.execute_tx(transfer_request).await.unwrap();

        let post_balance = client
            .query_balance(&accounts[1].to_string())
            .await
            .unwrap();

        assert_eq!(pre_balance + U256::from(200), post_balance);
    }

    #[tokio::test]
    #[ignore = "requires local anvil instance"]
    async fn test_eth_erc20_transfer_and_query() {
        let client = EthereumClient::new(TEST_RPC_URL, TEST_MNEMONIC, None).unwrap();
        let provider = client.get_request_provider().await.unwrap();
        let accounts = provider.get_accounts().await.unwrap();

        let token_1_tx =
            MockERC20::deploy_builder(&provider, "Token1".to_string(), "T1".to_string())
                .into_transaction_request();

        let token_addr = client
            .execute_tx(token_1_tx)
            .await
            .unwrap()
            .contract_address
            .unwrap();

        let token_1 = MockERC20::new(token_addr, provider);

        let mint_token1_tx = token_1
            .mint(accounts[0], U256::from(1000))
            .into_transaction_request();

        client.execute_tx(mint_token1_tx).await.unwrap();

        let pre_balance_call = token_1.balanceOf(accounts[1]);
        let pre_balance = client.query(pre_balance_call).await.unwrap()._0;

        let transfer_request_builder = token_1
            .transfer(accounts[1], U256::from(200))
            .into_transaction_request();

        client.execute_tx(transfer_request_builder).await.unwrap();

        let post_balance_call = token_1.balanceOf(accounts[1]);
        let post_balance = client.query(post_balance_call).await.unwrap()._0;

        assert_eq!(pre_balance + U256::from(200), post_balance);
    }

    #[tokio::test]
    #[ignore = "requires local anvil instance"]
    async fn test_eth_query_contract_states() {
        let client = EthereumClient::new(TEST_RPC_URL, TEST_MNEMONIC, None).unwrap();
        let provider = client.get_request_provider().await.unwrap();
        let accounts = provider.get_accounts().await.unwrap();

        let contract_addr = Address::from_str(TEST_CONTRACT_ADDR).unwrap();

        let mock_erc_20 = MockERC20::new(contract_addr, provider);

        let req = mock_erc_20.totalSupply();

        let response = client.query(req).await.unwrap();

        assert_ne!(U256::from(0), response._0);

        let req = mock_erc_20.balanceOf(accounts[0]);
        let response = client.query(req).await.unwrap();
        assert_eq!(U256::from(0), response._0);
    }
}
