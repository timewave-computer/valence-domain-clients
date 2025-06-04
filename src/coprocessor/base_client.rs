use async_trait::async_trait;
use serde_json::Value;
use valence_coprocessor::{Proof, ValidatedDomainBlock, Witness};

#[async_trait]
pub trait CoprocessorBaseClient {
    /// Returns statistics of the running instance.
    async fn stats(&self) -> anyhow::Result<Value>;

    /// Deploy a controller.
    ///
    /// Returns the allocated Id.
    async fn deploy_controller(
        &self,
        controller: &[u8],
        circuit: &[u8],
        nonce: Option<u64>,
    ) -> anyhow::Result<String>;

    /// Deploy a domain.
    ///
    /// Returns the allocated Id.
    async fn deploy_domain(&self, domain: &str, controller: &[u8]) -> anyhow::Result<String>;

    /// Fetch a storage file, returning its contents.
    ///
    /// The co-processor storage is a FAT-16 virtual filesystem, and bound to its limitations.
    async fn get_storage_file(&self, controller: &str, path: &str) -> anyhow::Result<Vec<u8>>;

    /// Computes the witnesses of a controller for the provided arguments.
    ///
    /// This is a dry-run for the prove call, that will use the same components to compute the
    /// witnesses.
    async fn get_witnesses(&self, circuit: &str, args: &Value) -> anyhow::Result<Vec<Witness>>;

    /// Proves the deployed `circuit` with the given `args`.
    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<Proof>;

    /// Get the verifying key for the provided circuit
    async fn get_vk(&self, circuit: &str) -> anyhow::Result<Vec<u8>>;

    /// Calls the controller entrypoint
    async fn entrypoint(&self, controller: &str, args: &Value) -> anyhow::Result<Value>;

    /// Returns the latest validated domain block.
    async fn get_latest_domain_block(&self, domain: &str) -> anyhow::Result<ValidatedDomainBlock>;

    /// Appends a block to the domain, validating it with the controller.
    async fn add_domain_block(&self, domain: &str, args: &Value) -> anyhow::Result<Value>;
}
