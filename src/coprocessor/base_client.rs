use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use msgpacker::MsgPacker;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A ZK proven circuit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, MsgPacker)]
pub struct Proof {
    /// The base64 encoded ZK proof.
    pub proof: String,

    /// The base64 encoded public inputs of the proof.
    pub inputs: String,
}

/// A runtime computed witnesses
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witnesses {
    /// Computed witnesses.
    pub witnesses: Value,

    /// Runtime logs.
    pub log: Vec<String>,
}

/// The returned value of an entrypoint call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entrypoint {
    /// Result of the computation.
    pub ret: Value,

    /// Runtime logs.
    pub log: Vec<String>,
}

/// A ZK proven circuit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, MsgPacker)]
pub struct DomainProof {
    pub program: Proof,
    pub domain: Proof,
}

/// A base64 encoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct Base64;

impl Base64 {
    /// Encodes the provided bytes into base64.
    pub fn encode<B: AsRef<[u8]>>(bytes: B) -> String {
        STANDARD.encode(bytes.as_ref())
    }

    /// Decodes the provided base64 into bytes.
    pub fn decode<B: AsRef<str>>(b64: B) -> anyhow::Result<Vec<u8>> {
        STANDARD
            .decode(b64.as_ref())
            .map_err(|e| anyhow::anyhow!("failed to decode base64: {e}"))
    }
}

#[async_trait]
pub trait CoprocessorBaseClient {
    /// Returns statistics of the running instance.
    async fn stats(&self) -> anyhow::Result<Value>;

    /// Co-processor historical root.
    async fn root(&self) -> anyhow::Result<String>;

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
    async fn deploy_domain(
        &self,
        domain: &str,
        controller: &[u8],
        circuit: &[u8],
    ) -> anyhow::Result<String>;

    /// Fetch the raw storage.
    async fn get_storage_raw(&self, circuit: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// Replaces the storage contents with the provided bytes
    async fn set_storage_raw(&self, circuit: &str, contents: &[u8]) -> anyhow::Result<()>;

    /// Fetch a storage file, returning its contents.
    ///
    /// The co-processor storage is a FAT-16 virtual filesystem, and bound to its limitations.
    async fn get_storage_file(&self, circuit: &str, path: &str) -> anyhow::Result<Option<Vec<u8>>>;

    /// Replaces the storage file path with the provided contents.
    ///
    /// Note: This is a FAT-16 filesystem, so extensions must have max 3 characters.
    async fn set_storage_file(
        &self,
        circuit: &str,
        path: &str,
        contents: &[u8],
    ) -> anyhow::Result<()>;

    /// Computes the witnesses of a controller for the provided arguments.
    async fn get_witnesses(&self, circuit: &str, args: &Value) -> anyhow::Result<Witnesses>;

    /// Proves the deployed `circuit` with the given `args`.
    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<DomainProof>;

    /// Get the verifying key for the provided circuit
    async fn get_vk(&self, circuit: &str) -> anyhow::Result<Vec<u8>>;

    /// Get the verifying key for the domain circuit
    async fn get_domain_vk(&self) -> anyhow::Result<String>;

    /// Get the circuit bytecode
    async fn get_circuit(&self, circuit: &str) -> anyhow::Result<Vec<u8>>;

    /// Get the circuit runtime bytecode
    async fn get_runtime(&self, circuit: &str) -> anyhow::Result<Vec<u8>>;

    /// Calls the controller entrypoint
    async fn entrypoint(&self, circuit: &str, args: &Value) -> anyhow::Result<Entrypoint>;

    /// Returns the latest validated domain block.
    async fn get_latest_domain_block(&self, domain: &str) -> anyhow::Result<Value>;

    /// Appends a block to the domain, validating it with the controller.
    ///
    /// Returns a JSON representation of `AddedDomainBlock`
    async fn add_domain_block(&self, domain: &str, args: &Value) -> anyhow::Result<Value>;

    /// Returns the available provers.
    async fn provers(&self) -> anyhow::Result<Value>;

    /// Adds the provided prover to the owned schedule.
    async fn provers_add(&self, prover: &str) -> anyhow::Result<Value>;

    /// Removes the provided prover from the owned schedule.
    async fn provers_remove(&self, prover: &str) -> anyhow::Result<Value>;

    /// Migrates an App from the source co-processor instance.
    async fn migrate(&self, circuit: &str, source: &str) -> anyhow::Result<Value>;
}
