use alloy::transports::http::reqwest;
use serde_json::Value;

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

impl IBCEurekaRouteClient {
    pub async fn query_skip_eureka_route(
        &self,
        amount: impl Into<String>,
    ) -> anyhow::Result<Value> {
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
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(resp)
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
    println!("{}", serde_json::to_string_pretty(&resp).unwrap());
}
