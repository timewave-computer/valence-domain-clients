use async_trait::async_trait;
use serde_json::Value;
use valence_coprocessor::Proof;
use valence_coprocessor_client::Client;

use crate::coprocessor::base_client::CoprocessorBaseClient;

/// A co-processor proving client.
///
/// By default, it connects to the default public instance of the service.
#[derive(Debug, Default, Clone)]
pub struct CoprocessorClient {
    client: Client,
}

impl CoprocessorClient {
    /// Starts a client that connects to a localhost co-processor
    pub fn local() -> Self {
        Self {
            client: Client::local(),
        }
    }
}

#[async_trait]
impl CoprocessorBaseClient for CoprocessorClient {
    fn stats(&self) -> anyhow::Result<Value> {
        self.client.stats()
    }

    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<Proof> {
        self.client.prove(circuit, args).await
    }
}

#[test]
fn client_stats_works() {
    CoprocessorClient::default().stats().unwrap();
}

#[tokio::test]
async fn client_prove_works() {
    let circuit = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";
    let args = serde_json::json!({"value": 42});

    Client::default().prove(circuit, &args).await.unwrap();
}
