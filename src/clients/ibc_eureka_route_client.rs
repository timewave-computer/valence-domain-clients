use alloy::transports::http::reqwest;

#[derive(Debug, Default, Clone)]
pub struct IBCEurekaRouteClient {
    client: reqwest::Client,
    api_url: String,
    src_chain_id: String,
    src_asset_denom: String,
    dest_chain_id: String,
    dest_chain_denom: String,
}

impl IBCEurekaRouteClient {
    pub fn new(
        api_url: &str,
        src_chain_id: &str,
        src_asset_denom: &str,
        dest_chain_id: &str,
        dest_chain_denom: &str,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url: api_url.to_string(),
            src_chain_id: src_chain_id.to_string(),
            src_asset_denom: src_asset_denom.to_string(),
            dest_chain_id: dest_chain_id.to_string(),
            dest_chain_denom: dest_chain_denom.to_string(),
        }
    }
}

/// these types can be extended with new fields.
/// if skip api extends the response type with new fields,
/// they will be ignored until added here.
pub mod skip_api_types {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct SkipRouteResponse {
        pub amount_in: Option<String>,
        pub amount_out: Option<String>,
        pub estimated_amount_out: Option<String>,
        pub does_swap: Option<bool>,
        pub usd_amount_in: Option<String>,
        pub usd_amount_out: Option<String>,
        pub chain_ids: Option<Vec<String>>,
        pub source_asset_chain_id: Option<String>,
        pub source_asset_denom: Option<String>,
        pub dest_asset_chain_id: Option<String>,
        pub dest_asset_denom: Option<String>,
        pub required_chain_addresses: Option<Vec<String>>,
        pub swap_venues: Option<Vec<Value>>,
        pub txs_required: Option<u64>,
        pub operations: Vec<Operation>,
        pub estimated_route_duration_seconds: u64,
        pub estimated_fees: Option<Vec<EstimatedFee>>,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct Operation {
        pub amount_in: Option<String>,
        pub amount_out: Option<String>,
        pub tx_index: Option<u64>,
        pub evm_swap: Option<EvmSwap>,
        pub eureka_transfer: Option<EurekaTransfer>,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct EvmSwap {
        pub input_token: Option<String>,
        pub amount_in: String,
        pub swap_calldata: Option<String>,
        pub amount_out: String,
        pub from_chain_id: Option<String>,
        pub denom_in: String,
        pub denom_out: String,
        pub swap_venues: Option<Vec<Value>>,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct EurekaTransfer {
        pub source_client: String,
        pub to_chain_entry_contract_address: String,
        pub to_chain_callback_contract_address: String,
        pub bridge_id: Option<String>,
        pub denom_in: Option<String>,
        pub denom_out: Option<String>,
        pub destination_port: Option<String>,
        pub from_chain_id: Option<String>,
        pub to_chain_id: Option<String>,
        pub pfm_enabled: Option<bool>,
        pub supports_memo: Option<bool>,
        pub smart_relay: Option<bool>,
        pub smart_relay_fee_quote: SmartRelayFeeQuote,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct SmartRelayFeeQuote {
        pub fee_amount: String,
        pub fee_denom: String,
        pub expiration: String,
        pub fee_payment_address: String,
        pub relayer_address: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct EstimatedFee {
        pub amount: String,
        pub bridge_id: String,
        pub chain_id: String,
        pub fee_behavior: String,
        pub fee_type: String,
        pub tx_index: Option<u64>,
        pub usd_amount: String,
        pub origin_asset: OriginAsset,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    #[serde(default)]
    pub struct OriginAsset {
        pub chain_id: String,
        pub coingecko_id: Option<String>,
        pub decimals: Option<u8>,
        pub denom: String,
        pub description: Option<String>,
        pub is_cw20: Option<bool>,
        pub is_evm: Option<bool>,
        pub is_svm: Option<bool>,
        pub logo_uri: Option<String>,
        pub name: Option<String>,
        pub origin_chain_id: Option<String>,
        pub origin_denom: Option<String>,
        pub recommended_symbol: Option<String>,
        pub symbol: Option<String>,
        pub trace: Option<String>,
        pub token_contract: Option<String>,
    }
}

impl IBCEurekaRouteClient {
    pub async fn query_skip_eureka_route(
        &self,
        amount: impl Into<String>,
    ) -> anyhow::Result<skip_api_types::SkipRouteResponse> {
        // build the eureka route request body
        let skip_request_body = serde_json::json!({
            "source_asset_chain_id": self.src_chain_id,
            "source_asset_denom": self.src_asset_denom,
            "dest_asset_chain_id": self.dest_chain_id,
            "dest_asset_denom": self.dest_chain_denom,
            "amount_in": amount.into(),
            "allow_unsafe": true,
            "allow_multi_tx": true,
            "go_fast": true,
            "smart_relay": true,
            "smart_swap_options": {
                "split_routes": true,
                "evm_swaps": true
            },
            "experimental_features": [
                "eureka"
            ]
        });

        // post the request to configured api and parse it into json Value
        let resp = self
            .client
            .post(self.api_url.to_string())
            .header("Content-Type", "application/json")
            .json(&skip_request_body)
            .send()
            .await?;

        let typed_resp: skip_api_types::SkipRouteResponse = resp.json().await?;

        Ok(typed_resp)
    }
}

#[tokio::test]
async fn test_route_query() {
    let api_url = "https://go.skip.build/api/skip/v2/fungible/route";

    let client = IBCEurekaRouteClient::new(
        api_url,
        "cosmoshub-4",
        "ibc/D742E8566B0B8CC8F569D950051C09CF57988A88F0E45574BFB3079D41DE6462",
        "1",
        "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
    );

    let resp = client.query_skip_eureka_route("10000000").await.unwrap();

    assert_eq!(resp.amount_in.unwrap(), "10000000");
    assert_eq!(resp.dest_asset_chain_id.unwrap(), "1");
    assert_eq!(
        resp.source_asset_denom.unwrap(),
        "ibc/D742E8566B0B8CC8F569D950051C09CF57988A88F0E45574BFB3079D41DE6462"
    );
    assert_eq!(
        resp.dest_asset_denom.unwrap(),
        "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
    );
    assert_eq!(resp.source_asset_chain_id.unwrap(), "cosmoshub-4");

    let eureka_transfer_op = resp
        .operations
        .iter()
        .find(|op| op.eureka_transfer.is_some())
        .unwrap();
    assert!(eureka_transfer_op.eureka_transfer.is_some());
}
