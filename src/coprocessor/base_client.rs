use async_trait::async_trait;
use serde_json::Value;
use valence_coprocessor::Proof;

#[async_trait]
pub trait CoprocessorBaseClient {
    /// Returns statistics of the running instance.
    fn stats(&self) -> anyhow::Result<Value>;

    /// Proves the deployed `circuit` with the given `args`.
    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<Proof>;
}
