use std::str::FromStr;

use crate::{
    common::transaction::TransactionResponse,
    cosmos::{
        base_client::BaseClient, grpc_client::GrpcSigningClient, proto_timestamp::ProtoTimestamp,
        wasm_client::WasmClient, CosmosServiceClient,
    },
};
use async_trait::async_trait;
use cosmrs::Denom;

const CHAIN_PREFIX: &str = "neutron";
const CHAIN_DENOM: &str = "untrn";

pub struct NeutronClient {
    grpc_url: String,
    mnemonic: String,
    chain_id: String,
    chain_denom: String,
    gas_price: f64,
}

impl NeutronClient {
    pub async fn new(
        rpc_url: &str,
        rpc_port: &str,
        mnemonic: &str,
        chain_id: &str,
    ) -> anyhow::Result<Self> {
        let avg_gas_price = Self::query_chain_gas_config("neutron", CHAIN_DENOM).await?;

        Ok(Self {
            grpc_url: format!("{rpc_url}:{rpc_port}"),
            mnemonic: mnemonic.to_string(),
            chain_id: chain_id.to_string(),
            chain_denom: CHAIN_DENOM.to_string(),
            gas_price: avg_gas_price,
        })
    }
}

impl NeutronClient {
    pub async fn create_tokenfactory_denom(
        &self,
        subdenom: &str,
    ) -> anyhow::Result<TransactionResponse> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        // Create the message to create a new denom
        let create_denom_msg = neutron_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom {
            sender: signing_client.address.to_string(),
            subdenom: subdenom.to_string(),
        }
        .to_any();

        // Convert to cosmrs::Any
        let any_msg = cosmrs::Any {
            type_url: create_denom_msg.type_url,
            value: create_denom_msg.value,
        };

        // Create and sign the transaction
        let raw_tx = signing_client
            .create_tx(
                any_msg,
                cosmrs::tx::Fee {
                    amount: vec![cosmrs::Coin {
                        denom: Denom::from_str("untrn")
                            .map_err(|e| anyhow::anyhow!("Failed to parse denom: {e}"))?,
                        amount: 100_000,
                    }],
                    gas_limit: 300_000,
                    payer: None,
                    granter: None,
                },
                None,
            )
            .await?;

        // Broadcast the transaction
        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        TransactionResponse::try_from(broadcast_tx_response.tx_response)
    }

    pub async fn mint_tokenfactory_tokens(
        &self,
        subdenom: &str,
        amount: u128,
        mint_to_address: Option<&str>,
    ) -> anyhow::Result<TransactionResponse> {
        let signing_client = self.get_signing_client().await?;
        let channel = self.get_grpc_channel().await?;

        // Format the full denom - follows factory/{creator_address}/{subdenom} pattern
        let full_denom = format!("factory/{}/{}", signing_client.address, subdenom);

        // Determine the recipient address (default to sender if not specified)
        let recipient = match mint_to_address {
            Some(addr) => addr.to_string(),
            None => signing_client.address.to_string(),
        };

        // Create the mint message
        let mint_msg = neutron_std::types::osmosis::tokenfactory::v1beta1::MsgMint {
            sender: signing_client.address.to_string(),
            amount: Some(neutron_std::types::cosmos::base::v1beta1::Coin {
                denom: full_denom,
                amount: amount.to_string(),
            }),
            mint_to_address: recipient,
        }
        .to_any();

        // Convert to cosmrs::Any
        let any_msg = cosmrs::Any {
            type_url: mint_msg.type_url,
            value: mint_msg.value,
        };

        // Simulate tx to get fee estimation
        let simulation_response = self.simulate_tx(any_msg.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        // Create and sign the transaction
        let raw_tx = signing_client.create_tx(any_msg, fee, None).await?;

        // Broadcast the transaction
        let mut grpc_client = CosmosServiceClient::new(channel);
        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        TransactionResponse::try_from(broadcast_tx_response.tx_response)
    }
}

#[async_trait]
impl BaseClient for NeutronClient {
    /// neutron has custom ibc logic so we override the default BaseClient ibc_transfer
    async fn ibc_transfer(
        &self,
        to: String,
        denom: String,
        amount: String,
        channel_id: String,
        timeout_seconds: u64,
        memo: Option<String>,
    ) -> anyhow::Result<TransactionResponse> {
        // first we query the latest block header to respect the chain time for timeouts
        let latest_block_header = self.latest_block_header().await?;

        let mut current_time = ProtoTimestamp::try_from(latest_block_header)?;

        current_time.extend_by_seconds(timeout_seconds)?;

        let timeout_nanos = current_time.to_nanos()?;

        let signing_client = self.get_signing_client().await?;

        let ibc_transfer_msg = neutron_std::types::ibc::applications::transfer::v1::MsgTransfer {
            source_port: "transfer".to_string(),
            source_channel: channel_id.to_string(),
            token: Some(neutron_std::types::cosmos::base::v1beta1::Coin {
                denom: denom.to_string(),
                amount,
            }),
            sender: signing_client.address.to_string(),
            receiver: to.to_string(),
            timeout_height: None,
            timeout_timestamp: timeout_nanos,
            memo: memo.unwrap_or_default(),
        }
        .to_any();

        // convert to cosmrs::Any
        let valid_any = cosmrs::Any {
            type_url: ibc_transfer_msg.type_url,
            value: ibc_transfer_msg.value,
        };

        let simulation_response = self.simulate_tx(valid_any.clone()).await?;
        let fee = self.get_tx_fee(simulation_response)?;

        let raw_tx = signing_client
            .create_tx(valid_any, fee, None)
            .await
            .unwrap();

        let channel = self.get_grpc_channel().await?;

        let mut grpc_client = CosmosServiceClient::new(channel);

        let broadcast_tx_response = grpc_client.broadcast_tx(raw_tx).await?.into_inner();

        TransactionResponse::try_from(broadcast_tx_response.tx_response)
    }
}

#[async_trait]
impl WasmClient for NeutronClient {}

#[async_trait]
impl GrpcSigningClient for NeutronClient {
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use serde::Deserialize;
    use serde_json::json;

    use crate::clients::osmosis::OsmosisClient;

    use super::*;

    const LOCAL_GRPC_URL: &str = "http://127.0.0.1";
    const LOCAL_GRPC_PORT: &str = "32889";
    const LOCAL_MNEMONIC: &str = "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry";
    const LOCAL_ALT_ADDR: &str = "neutron1kljf09rj77uxeu5lye7muejx6ajsu55cuw2mws";
    const LOCAL_CHAIN_ID: &str = "localneutron-1";
    const LOCAL_PROCESSOR_ADDR: &str =
        "neutron12p7twsmksqw8lhj98hlxld7hxfl3tmwn6853ggtsalzm2ryx7ylsrmdfr6";

    const NEUTRON_ON_OSMO: &str =
        "ibc/4E41ED8F3DCAEA15F4D6ADC6EDD7C04A676160735C9710B904B7BF53525B56D6";

    // update during dev to a real one for mainnet testing
    const _CHAIN_ID: &str = "neutron-1";
    const _GRPC_URL: &str = "-";
    const _GRPC_PORT: &str = "-";
    const _NEUTRON_DAO_ADDR: &str =
        "neutron1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrstdxvff";
    const _MNEMONIC: &str = "-";

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_latest_block_height() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let block_height = client
            .latest_block_header()
            .await
            .expect("Failed to get latest block height")
            .height;

        println!("block height: {block_height}");
        assert!(block_height > 0);
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_query_balance() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let admin_addr = client
            .get_signing_client()
            .await
            .unwrap()
            .address
            .to_string();

        let balance = client.query_balance(&admin_addr, "untrn").await.unwrap();

        assert!(balance > 0);
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_query_contract_state() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        #[derive(Deserialize, Debug, PartialEq)]
        enum State {
            Paused,
            Active,
        }

        #[derive(Deserialize, Debug)]
        enum ProcessorDomain {
            Main,
        }

        #[derive(Deserialize, Debug)]
        struct Config {
            pub _authorization_contract: String,
            pub _processor_domain: ProcessorDomain,
            pub state: State,
        }

        let query = json!({"config": {}});

        let state: Config = client
            .query_contract_state(LOCAL_PROCESSOR_ADDR, query)
            .await
            .unwrap();

        assert_eq!(state.state, State::Active);
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_transfer() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let pre_transfer_balance = client
            .query_balance(LOCAL_ALT_ADDR, CHAIN_DENOM)
            .await
            .unwrap();

        let rx = client
            .transfer(LOCAL_ALT_ADDR, 100_000, CHAIN_DENOM, None)
            .await
            .unwrap();

        client.poll_for_tx(&rx.hash).await.unwrap();

        let post_transfer_balance = client
            .query_balance(LOCAL_ALT_ADDR, CHAIN_DENOM)
            .await
            .unwrap();

        assert_eq!(pre_transfer_balance + 100_000, post_transfer_balance);
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_execute_wasm() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let tick_msg = json!({
            "permisionless_action": { "tick": {}}
        });

        let rx = client
            .execute_wasm(LOCAL_PROCESSOR_ADDR, tick_msg, vec![], None)
            .await
            .unwrap();

        let response = client.poll_for_tx(&rx.hash).await.unwrap();

        assert!(response.height > 0);
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_upload_wasm() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let authorizations_code = client
            .upload_code("valence_authorization.wasm")
            .await
            .unwrap();
        let processor_code = client.upload_code("valence_processor.wasm").await.unwrap();

        assert_eq!(authorizations_code + 1, processor_code);

        let code_info_response = client.query_code_info(authorizations_code).await.unwrap();
        assert_eq!(
            code_info_response.code_info.unwrap().code_id,
            authorizations_code
        );
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_instantiate_wasm() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let authorizations_code = client
            .upload_code("valence_authorization.wasm")
            .await
            .unwrap();

        let instantiate_msg = json!({
            "owner": LOCAL_PROCESSOR_ADDR,
            "sub_owners": [],
            "processor": LOCAL_PROCESSOR_ADDR.to_string(),
        });
        let authorizations_addr = client
            .instantiate(
                authorizations_code,
                "authorizations_test".to_string(),
                instantiate_msg,
                None,
            )
            .await
            .unwrap();
        assert!(authorizations_addr.starts_with("neutron"));
    }

    #[tokio::test]
    #[ignore = "requires local neutron grpc node active"]
    async fn test_instantiate2_wasm() {
        let client = NeutronClient::new(_GRPC_URL, _GRPC_PORT, _MNEMONIC, _CHAIN_ID)
            .await
            .unwrap();

        let signing_client = client.get_signing_client().await.unwrap();

        let base_acc_code = 3319;

        let instantiate_msg = json!({
            "admin": signing_client.address.to_string(),
            "approved_libraries": [],
        });

        let salt = hex::encode(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        );

        let predicted_base_acc_addr = client
            .predict_instantiate2_addr(
                base_acc_code,
                salt.clone(),
                signing_client.address.to_string(),
            )
            .await
            .unwrap()
            .address;

        let instantiated_base_acc_addr = client
            .instantiate2(
                base_acc_code,
                "test_instantiate_2".to_string(),
                instantiate_msg,
                None,
                salt,
            )
            .await
            .unwrap();

        assert_eq!(predicted_base_acc_addr, instantiated_base_acc_addr);
    }

    #[tokio::test]
    #[ignore = "requires local neutron & osmosis grpc nodes active"]
    async fn test_ibc_transfer() {
        let client = NeutronClient::new(
            LOCAL_GRPC_URL,
            LOCAL_GRPC_PORT,
            LOCAL_MNEMONIC,
            LOCAL_CHAIN_ID,
        )
        .await
        .unwrap();

        let osmosis_client =
            OsmosisClient::new(LOCAL_GRPC_URL, "45355", LOCAL_MNEMONIC, "localosmosis-1")
                .await
                .unwrap();

        let osmo_signer = osmosis_client.get_signing_client().await.unwrap();
        let ntrn_signer = client.get_signing_client().await.unwrap();

        let osmo_admin_addr = osmo_signer.address.to_string();
        let ntrn_admin_addr = ntrn_signer.address.to_string();

        let osmo_balance_0 = osmosis_client
            .query_balance(&osmo_admin_addr, NEUTRON_ON_OSMO)
            .await
            .unwrap();
        println!("osmo_balance_0: {osmo_balance_0}");

        let tx_response = client
            .ibc_transfer(
                osmo_admin_addr.to_string(),
                client.chain_denom().to_string(),
                "100000".to_string(),
                "channel-0".to_string(),
                5,
                None,
            )
            .await
            .unwrap();

        client.poll_for_tx(&tx_response.hash).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let osmo_balance_1 = osmosis_client
            .query_balance(&osmo_admin_addr, NEUTRON_ON_OSMO)
            .await
            .unwrap();
        println!("osmo_balance_1: {osmo_balance_1}");

        // assert that first transfer worked
        assert_eq!(osmo_balance_0 + 100_000, osmo_balance_1);

        let osmo_rx = osmosis_client
            .ibc_transfer(
                ntrn_admin_addr.to_string(),
                NEUTRON_ON_OSMO.to_string(),
                "100000".to_string(),
                "channel-0".to_string(),
                5,
                None,
            )
            .await
            .unwrap();

        osmosis_client.poll_for_tx(&osmo_rx.hash).await.unwrap();

        tokio::time::sleep(Duration::from_secs(5)).await;

        let osmo_balance_2 = osmosis_client
            .query_balance(&osmo_admin_addr, NEUTRON_ON_OSMO)
            .await
            .unwrap();
        println!("osmo_balance_2: {osmo_balance_2}");

        // assert that the second transfer worked
        assert_eq!(osmo_balance_0, osmo_balance_2);
    }
}
