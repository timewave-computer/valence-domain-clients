use async_trait::async_trait;
use serde_json::Value;
use valence_coprocessor::{Proof, ValidatedDomainBlock, Witness};
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
    async fn stats(&self) -> anyhow::Result<Value> {
        self.client.stats().await
    }

    async fn deploy_controller(
        &self,
        controller: &[u8],
        circuit: &[u8],
        nonce: Option<u64>,
    ) -> anyhow::Result<String> {
        self.client
            .deploy_controller(controller, circuit, nonce)
            .await
    }

    async fn deploy_domain(&self, domain: &str, controller: &[u8]) -> anyhow::Result<String> {
        self.client.deploy_domain(domain, controller).await
    }

    async fn get_storage_file(&self, controller: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        self.client.get_storage_file(controller, path).await
    }

    async fn get_witnesses(&self, circuit: &str, args: &Value) -> anyhow::Result<Vec<Witness>> {
        self.client.get_witnesses(circuit, args).await
    }

    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<Proof> {
        self.client.prove(circuit, args).await
    }

    async fn get_vk(&self, circuit: &str) -> anyhow::Result<Vec<u8>> {
        self.client.get_vk(circuit).await
    }

    async fn entrypoint(&self, controller: &str, args: &Value) -> anyhow::Result<Value> {
        self.client.entrypoint(controller, args).await
    }

    async fn get_latest_domain_block(&self, domain: &str) -> anyhow::Result<ValidatedDomainBlock> {
        self.client.get_latest_domain_block(domain).await
    }

    async fn add_domain_block(&self, domain: &str, args: &Value) -> anyhow::Result<Value> {
        self.client.add_domain_block(domain, args).await
    }
}

#[tokio::test]
async fn client_stats_works() {
    CoprocessorClient::default().stats().await.unwrap();
}

#[tokio::test]
async fn client_deploy_controller_works() {
    CoprocessorClient::default()
        .deploy_controller(b"foo", b"bar", Some(15))
        .await
        .unwrap();
}

#[tokio::test]
async fn client_deploy_domain_works() {
    CoprocessorClient::default()
        .deploy_domain("foo", b"bar")
        .await
        .unwrap();
}

#[tokio::test]
async fn client_get_storage_file_works() {
    let controller = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";
    let path = "/var/share/proof.bin";

    CoprocessorClient::default()
        .get_storage_file(controller, path)
        .await
        .unwrap();
}

#[tokio::test]
async fn client_get_witnesses_works() {
    let circuit = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";
    let args = serde_json::json!({"value": 42});

    CoprocessorClient::default()
        .get_witnesses(circuit, &args)
        .await
        .unwrap();
}

#[tokio::test]
async fn client_prove_works() {
    let circuit = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";
    let args = serde_json::json!({"value": 42});

    CoprocessorClient::default()
        .prove(circuit, &args)
        .await
        .unwrap();
}

#[tokio::test]
async fn client_get_vk_works() {
    let circuit = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";

    CoprocessorClient::default().get_vk(circuit).await.unwrap();
}

#[tokio::test]
async fn client_entrypoint_works() {
    let controller = "7e0207a1fa0a979282b7246c028a6a87c25bc60f7b6d5230e943003634e897fd";
    let args = serde_json::json!({
        "payload": {
            "cmd": "store",
            "path": "/etc/foo.bin",
        }
    });

    CoprocessorClient::default()
        .entrypoint(controller, &args)
        .await
        .unwrap();
}

#[tokio::test]
async fn client_get_latest_domain_block_works() {
    CoprocessorClient::default()
        .get_latest_domain_block("ethereum-alpha")
        .await
        .unwrap();
}
