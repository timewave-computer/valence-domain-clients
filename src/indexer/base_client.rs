use alloy::transports::http::reqwest;
use tonic::async_trait;

#[async_trait]
pub trait ValenceIndexerBaseClient {
    fn get_request_client(&self) -> reqwest::Client;
    fn get_api_key(&self) -> String;
    fn get_indexer_url(&self) -> String;
}
