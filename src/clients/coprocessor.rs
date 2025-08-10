use async_trait::async_trait;
use msgpacker::Unpackable as _;
use serde_json::{json, Value};
use tokio::time::{self, Duration};
use uuid::Uuid;

use crate::coprocessor::base_client::{Base64, CoprocessorBaseClient, DomainProof, Proof};

/// A co-processor proving client.
///
/// By default, it connects to the default public instance of the service.
#[derive(Debug, Clone)]
pub struct CoprocessorClient {
    /// The co-processor address.
    pub coprocessor: String,
}

impl Default for CoprocessorClient {
    fn default() -> Self {
        Self::new(Self::DEFAULT_COPROCESSOR.into())
    }
}

impl CoprocessorClient {
    /// The default co-processor public address.
    pub const DEFAULT_COPROCESSOR: &str = "https://service.coprocessor.valence.zone";

    /// The deployed domain prover circuit id.
    ///
    /// This is a fixed (and temporary) domain circuit that will compute the proof that binds the
    /// program to a given co-processor root.
    ///
    /// It will be replaced by the revamp of the historical co-processor tree and will split domain
    /// trees from the main historical tree, as described in
    ///
    /// https://www.notion.so/Domain-proofs-2365cfa0622c802f89e8d3cbd5f750c3
    pub const DOMAIN_CIRCUIT: &str =
        "cf4d4c2f3bf4ad0114091ea8023ff2456d5572e68bbc4d4e91bfa8f4a6f5d502";

    pub fn new(coprocessor: String) -> Self {
        Self { coprocessor }
    }

    /// Starts a client that connects to a localhost co-processor
    pub fn local() -> Self {
        Self {
            coprocessor: "http://127.0.0.1:37281".into(),
        }
    }

    /// Computes the URI of the co-processor.
    pub fn uri<P: AsRef<str>>(&self, path: P) -> String {
        format!("{}/api/{}", self.coprocessor, path.as_ref(),)
    }

    /// Fetches a proof from the queue, returning if present.
    pub async fn get_proof_from_storage<C: AsRef<str>, P: AsRef<str>>(
        &self,
        circuit: C,
        path: P,
    ) -> anyhow::Result<Option<Proof>> {
        let uri = format!("registry/controller/{}/storage/fs", circuit.as_ref());
        let uri = self.uri(uri);

        let response = reqwest::Client::new()
            .post(uri)
            .json(&json!({
                "path": path.as_ref()
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        let data = match response.get("data") {
            Some(d) => d,
            _ => return Ok(None),
        };

        let data = data
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("unexpected data format"))?;

        if data.is_empty() {
            return Ok(None);
        }

        let data = Base64::decode(data)?;
        let data: Value = serde_json::from_slice(&data)?;

        anyhow::ensure!(
            data.get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            "the proof was computed incorrectly"
        );

        let proof = data
            .get("proof")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("failed to get proof from response"))?;

        let bytes = Base64::decode(proof)?;
        let proof = Proof::unpack(&bytes)
            .map_err(|e| anyhow::anyhow!("failed to unpack proof: {e}"))?
            .1;

        Ok(Some(proof))
    }

    async fn _prove(&self, circuit: &str, path: &str, frequency: u64) -> anyhow::Result<Proof> {
        let frequency = Duration::from_millis(frequency);

        loop {
            if let Some(p) = self.get_proof_from_storage(circuit, path).await? {
                return Ok(p);
            }

            time::sleep(frequency).await;
        }
    }

    async fn get_single_proof(
        &self,
        circuit: &str,
        args: &Value,
        root: &str,
    ) -> anyhow::Result<Proof> {
        // 600000millisecs = 10min
        let retries = 50;
        let frequency = 12000;

        let uri = format!("registry/controller/{circuit}/prove/{root}");
        let uri = self.uri(uri);

        let output = Uuid::new_v4();
        let output = output.as_u128().to_le_bytes();
        let output = hex::encode(output);
        let path = format!("/var/share/proofs/{}.bin", &output[..8]);
        let args = json!({
            "args": args,
            "payload": {
                "cmd": "store",
                "path": &path
            }
        });

        reqwest::Client::new()
            .post(uri)
            .json(&args)
            .send()
            .await?
            .text()
            .await?;

        let duration = retries * frequency;
        let duration = Duration::from_millis(duration);
        let duration = time::sleep(duration);

        tokio::pin!(duration);

        tokio::select! {
            r = self._prove(circuit, &path, frequency) => {
                r
            }

            _ = &mut duration => {
                anyhow::bail!("proof timeout exceeded");
            }
        }
    }
}

#[async_trait]
impl CoprocessorBaseClient for CoprocessorClient {
    async fn stats(&self) -> anyhow::Result<Value> {
        let uri = self.uri("stats");

        Ok(reqwest::Client::new().get(uri).send().await?.json().await?)
    }

    async fn root(&self) -> anyhow::Result<String> {
        let uri = self.uri("historical");
        let root: Value = reqwest::Client::new().get(uri).send().await?.json().await?;

        root.get("root")
            .and_then(Value::as_str)
            .map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("invalid root type"))
    }

    async fn deploy_controller(
        &self,
        controller: &[u8],
        circuit: &[u8],
        nonce: Option<u64>,
    ) -> anyhow::Result<String> {
        let uri = self.uri("registry/controller");

        reqwest::Client::new()
            .post(uri)
            .json(&json!({
                "controller": Base64::encode(controller),
                "circuit": Base64::encode(circuit),
                "nonce": nonce,
            }))
            .send()
            .await?
            .json::<Value>()
            .await?
            .get("controller")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("invalid response"))
    }

    async fn deploy_domain(
        &self,
        domain: &str,
        controller: &[u8],
        circuit: &[u8],
    ) -> anyhow::Result<String> {
        let uri = self.uri("registry/domain");

        reqwest::Client::new()
            .post(uri)
            .json(&json!({
                "name": domain,
                "controller": Base64::encode(controller),
                "circuit": Base64::encode(circuit),
            }))
            .send()
            .await?
            .json::<Value>()
            .await?
            .get("domain")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("invalid response"))
    }

    async fn get_storage_file(&self, controller: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        let uri = format!("registry/controller/{}/storage/fs", controller);
        let uri = self.uri(uri);

        reqwest::Client::new()
            .post(uri)
            .json(&json!({
                "path": path,
            }))
            .send()
            .await?
            .json::<Value>()
            .await?
            .get("data")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("invalid response"))
            .and_then(Base64::decode)
    }

    async fn get_witnesses(&self, circuit: &str, args: &Value) -> anyhow::Result<Value> {
        let uri = format!("registry/controller/{}/witnesses", circuit);
        let uri = self.uri(uri);

        let data = reqwest::Client::new()
            .post(uri)
            .json(&json!({
                "args": args
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;

        data.get("witnesses")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid witnesses response"))
    }

    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<DomainProof> {
        let uri = "http://prover.timewave.computer:37279/api/latest";
        let data = reqwest::Client::new()
            .get(uri)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let wrapper = data
            .get("wrapper")
            .ok_or_else(|| anyhow::anyhow!("no wrapper proof available"))?;

        let domain: Proof = serde_json::from_value(wrapper.clone())?;
        let inputs = Base64::decode(&domain.inputs)?;
        let root = <[u8; 32]>::try_from(inputs.as_slice())?;
        let root = hex::encode(root);
        let program = self.get_single_proof(circuit, args, &root).await?;

        Ok(DomainProof { program, domain })
    }

    async fn get_vk(&self, circuit: &str) -> anyhow::Result<Vec<u8>> {
        let uri = format!("registry/controller/{}/vk", circuit);
        let uri = self.uri(uri);

        let data = reqwest::Client::new()
            .get(uri)
            .send()
            .await?
            .json::<Value>()
            .await?;

        data.get("base64")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("invalid vk response"))
            .and_then(Base64::decode)
    }

    async fn get_domain_vk(&self) -> anyhow::Result<Vec<u8>> {
        self.get_vk(Self::DOMAIN_CIRCUIT).await
    }

    async fn entrypoint(&self, controller: &str, args: &Value) -> anyhow::Result<Value> {
        let uri = format!("registry/controller/{}/entrypoint", controller);
        let uri = self.uri(uri);

        let data = reqwest::Client::new()
            .post(uri)
            .json(args)
            .send()
            .await?
            .json::<Value>()
            .await?;

        data.get("ret")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no response provided"))
    }

    async fn get_latest_domain_block(&self, domain: &str) -> anyhow::Result<Value> {
        let uri = format!("registry/domain/{}/latest", domain);
        let uri = self.uri(uri);

        Ok(reqwest::Client::new().get(uri).send().await?.json().await?)
    }

    async fn add_domain_block(&self, domain: &str, args: &Value) -> anyhow::Result<Value> {
        let uri = format!("registry/domain/{}", domain);
        let uri = self.uri(uri);

        Ok(reqwest::Client::new()
            .post(uri)
            .json(&args)
            .send()
            .await?
            .json()
            .await?)
    }
}

#[tokio::test]
async fn coprocessor_stats_works() {
    CoprocessorClient::default().stats().await.unwrap();
}

#[tokio::test]
async fn coprocessor_root_works() {
    CoprocessorClient::default().root().await.unwrap();
}

#[tokio::test]
async fn coprocessor_deploy_controller_works() {
    CoprocessorClient::default()
        .deploy_controller(b"foo", b"bar", Some(15))
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_deploy_domain_works() {
    CoprocessorClient::default()
        .deploy_domain("foo", b"bar", b"baz")
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_get_storage_file_works() {
    let controller = "1c449e308ab05102e06969bc2f81162b5bfc824269011ac7ad45844a2dabc9c3";
    let path = "/var/share/proof.bin";

    CoprocessorClient::default()
        .get_storage_file(controller, path)
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_get_witnesses_works() {
    let circuit = "1c449e308ab05102e06969bc2f81162b5bfc824269011ac7ad45844a2dabc9c3";
    let args = serde_json::json!({"value": 42});

    CoprocessorClient::default()
        .get_witnesses(circuit, &args)
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_prove_works() {
    let circuit = "1c449e308ab05102e06969bc2f81162b5bfc824269011ac7ad45844a2dabc9c3";
    let args = serde_json::json!({"value": 42});

    let proof = CoprocessorClient::default()
        .prove(circuit, &args)
        .await
        .unwrap();

    let program = Base64::decode(proof.program.inputs).unwrap();
    let domain = Base64::decode(proof.domain.inputs).unwrap();

    assert_eq!(&program[32..], &43u64.to_le_bytes());
    assert_eq!(&program[..32], &domain);
}

#[tokio::test]
async fn coprocessor_get_vk_works() {
    let circuit = "1c449e308ab05102e06969bc2f81162b5bfc824269011ac7ad45844a2dabc9c3";

    CoprocessorClient::default().get_vk(circuit).await.unwrap();
}

#[tokio::test]
async fn coprocessor_entrypoint_works() {
    let controller = "1c449e308ab05102e06969bc2f81162b5bfc824269011ac7ad45844a2dabc9c3";
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
async fn coprocessor_get_latest_domain_block_works() {
    CoprocessorClient::default()
        .get_latest_domain_block("ethereum-electra-alpha")
        .await
        .unwrap();
}
