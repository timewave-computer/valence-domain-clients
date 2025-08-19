use async_trait::async_trait;
use msgpacker::Unpackable as _;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::time::{self, Duration};
use uuid::Uuid;

use crate::coprocessor::base_client::{
    Base64, CoprocessorBaseClient, DomainProof, Entrypoint, Proof, Witnesses,
};

#[derive(Debug, Clone)]
pub struct RequestBuilder<'a> {
    uri: &'a str,
    circuit: Option<&'a str>,
    root: Option<&'a str>,
    args: Option<&'a Value>,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(uri: &'a str) -> Self {
        Self {
            uri,
            circuit: None,
            root: None,
            args: None,
        }
    }

    pub fn with_circuit(mut self, circuit: &'a str) -> Self {
        self.circuit.replace(circuit);
        self
    }

    pub fn with_root(mut self, root: &'a str) -> Self {
        self.root.replace(root);
        self
    }

    pub fn with_args(mut self, args: &'a Value) -> Self {
        self.args.replace(args);
        self
    }

    async fn _send<T: DeserializeOwned>(
        self,
        mut client: reqwest::RequestBuilder,
        get: bool,
    ) -> anyhow::Result<T> {
        let Self {
            circuit,
            root,
            args,
            ..
        } = self;

        // signer settings might change at runtime
        if !get {
            if let Ok(signer) = valence_crypto_utils::Signer::try_from_env() {
                let message = args.unwrap_or(&Value::Null);
                let message = serde_json::to_vec(&message)?;
                let signature = signer.sign_json(&message)?;
                let signature = const_hex::encode(signature);

                client = client.header("valence-coprocessor-signature", signature);
            }
        }

        if let Some(circuit) = circuit {
            client = client.header("valence-coprocessor-circuit", circuit)
        }

        if let Some(root) = root {
            client = client.header("valence-coprocessor-root", root)
        }

        if let Some(args) = args {
            client = client.json(args);
        }

        Ok(client.send().await?.json().await?)
    }

    pub async fn get<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        let client = reqwest::Client::new().get(self.uri);

        self._send::<T>(client, true).await
    }

    pub async fn post<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        let client = reqwest::Client::new().post(self.uri);

        self._send::<T>(client, false).await
    }
}

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
    pub async fn get_proof_from_storage<P: AsRef<str>>(
        &self,
        circuit: &str,
        path: P,
    ) -> anyhow::Result<Option<Proof>> {
        let data = match self
            .get_storage_file(circuit.as_ref(), path.as_ref())
            .await?
        {
            Some(d) => d,
            None => return Ok(None),
        };

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
        // timeout 600000millisecs = 10min
        let retries = 50;
        let frequency = 12000;

        let uri = self.uri("circuit/prove");

        let output = Uuid::new_v4();
        let output = output.as_u128().to_le_bytes();
        let output = const_hex::encode(output);
        let path = format!("/var/share/proofs/{}.bin", &output[..8]);
        let args = json!({
            "args": args,
            "payload": {
                "cmd": "store",
                "path": &path
            }
        });

        let res: Value = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_root(root)
            .with_args(&args)
            .post()
            .await?;

        let status = res
            .get("status")
            .and_then(Value::as_str)
            .filter(|s| *s == "received")
            .is_some();

        anyhow::ensure!(status, "failed to submit proof");

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
        let stats = RequestBuilder::new(&uri).get().await?;

        Ok(stats)
    }

    async fn root(&self) -> anyhow::Result<String> {
        let uri = self.uri("historical");
        let root: Value = RequestBuilder::new(&uri).get().await?;

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
        let ret: Value = RequestBuilder::new(&uri)
            .with_args(&json!({
                "controller": Base64::encode(controller),
                "circuit": Base64::encode(circuit),
                "nonce": nonce,
            }))
            .post()
            .await?;

        ret.get("controller")
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
        let ret: Value = RequestBuilder::new(&uri)
            .with_args(&json!({
                "name": domain,
                "controller": Base64::encode(controller),
                "circuit": Base64::encode(circuit),
            }))
            .post()
            .await?;

        ret.get("domain")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("invalid response"))
    }

    async fn get_storage_raw(&self, circuit: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let uri = self.uri("circuit/storage/raw");
        let response: Option<String> = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .get()
            .await?;

        let response = response.map(Base64::decode).transpose()?;

        Ok(response)
    }

    async fn set_storage_raw(&self, circuit: &str, contents: &[u8]) -> anyhow::Result<()> {
        let uri = self.uri("circuit/storage/raw");
        let contents = Base64::encode(contents);

        RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_args(&json!(contents))
            .post::<Value>()
            .await?;

        Ok(())
    }

    async fn get_storage_file(&self, circuit: &str, path: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let uri = self.uri("circuit/storage/fs");

        let response: Option<String> = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_args(&json!({
                "path": path,
            }))
            .get()
            .await?;

        let response = response.map(Base64::decode).transpose()?;

        Ok(response)
    }

    async fn set_storage_file(
        &self,
        circuit: &str,
        path: &str,
        contents: &[u8],
    ) -> anyhow::Result<()> {
        let uri = self.uri("circuit/storage/fs");
        let contents = Base64::encode(contents);

        RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_args(&json!({
                "path": path,
                "contents": contents,
            }))
            .post::<Value>()
            .await?;

        Ok(())
    }

    async fn get_witnesses(&self, circuit: &str, args: &Value) -> anyhow::Result<Witnesses> {
        let uri = self.uri("circuit/witnesses");
        let witnesses = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_args(&json!({
                "args": args
            }))
            .post()
            .await?;

        Ok(witnesses)
    }

    async fn prove(&self, circuit: &str, args: &Value) -> anyhow::Result<DomainProof> {
        let uri = "http://prover.timewave.computer:37279/api/latest";
        let data = reqwest::Client::new()
            .get(uri)
            .header("valence-coprocessor-circuit", circuit)
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
        let root = const_hex::encode(root);
        let program = self.get_single_proof(circuit, args, &root).await?;

        Ok(DomainProof { program, domain })
    }

    async fn get_vk(&self, circuit: &str) -> anyhow::Result<Vec<u8>> {
        let uri = self.uri("circuit/vk");
        let data: String = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .get()
            .await?;

        Base64::decode(data)
    }

    async fn get_domain_vk(&self) -> anyhow::Result<String> {
        Ok("0x009bd7e557762036be20824c6c4571b7ae70c6cd7a103915a4c5cbe32350b95a".into())
    }

    async fn get_circuit(&self, circuit: &str) -> anyhow::Result<Vec<u8>> {
        let uri = self.uri("circuit/bytecode");
        let data: String = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .get()
            .await?;

        Base64::decode(data)
    }

    async fn get_runtime(&self, circuit: &str) -> anyhow::Result<Vec<u8>> {
        let uri = self.uri("circuit/runtime");
        let data: String = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .get()
            .await?;

        Base64::decode(data)
    }

    async fn entrypoint(&self, circuit: &str, args: &Value) -> anyhow::Result<Entrypoint> {
        let uri = self.uri("circuit/entrypoint");
        let data = RequestBuilder::new(&uri)
            .with_circuit(circuit)
            .with_args(args)
            .post()
            .await?;

        Ok(data)
    }

    async fn get_latest_domain_block(&self, domain: &str) -> anyhow::Result<Value> {
        let uri = format!("registry/domain/{}/latest", domain);
        let uri = self.uri(uri);
        let data = RequestBuilder::new(&uri).get().await?;

        Ok(data)
    }

    async fn add_domain_block(&self, domain: &str, args: &Value) -> anyhow::Result<Value> {
        let uri = format!("registry/domain/{}", domain);
        let uri = self.uri(uri);
        let data = RequestBuilder::new(&uri).with_args(args).post().await?;

        Ok(data)
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
async fn coprocessor_storage_raw_works() {
    let circuit = blake3::hash(b"coprocessor_storage_raw_works")
        .to_hex()
        .to_string();

    let data = b"foo";

    CoprocessorClient::default()
        .set_storage_raw(&circuit, data)
        .await
        .unwrap();

    let storage = CoprocessorClient::default()
        .get_storage_raw(&circuit)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(&data[..], storage.as_slice());
}

#[tokio::test]
async fn coprocessor_storage_file_works() {
    let circuit = blake3::hash(b"coprocessor_storage_file_works")
        .to_hex()
        .to_string();

    let path = "/var/share/data.txt";
    let data = b"foo";

    CoprocessorClient::default()
        .set_storage_file(&circuit, path, data)
        .await
        .unwrap();

    let contents = CoprocessorClient::default()
        .get_storage_file(&circuit, path)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(&data[..], contents.as_slice());
}

#[tokio::test]
async fn coprocessor_get_witnesses_works() {
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";

    let value = 42u64;
    let args = serde_json::json!({"value": value});

    let ret = CoprocessorClient::default()
        .get_witnesses(circuit, &args)
        .await
        .unwrap();

    let data = ret.witnesses["witnesses"].as_array().unwrap()[0]["Data"].clone();
    let data: Vec<u8> = serde_json::from_value(data).unwrap();
    let data = u64::from_le_bytes(data.as_slice().try_into().unwrap());

    assert_eq!(data, value);
}

#[tokio::test]
async fn coprocessor_prove_works() {
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";
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
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";

    CoprocessorClient::default().get_vk(circuit).await.unwrap();
}

#[tokio::test]
async fn coprocessor_get_circuit_works() {
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";

    CoprocessorClient::default()
        .get_circuit(circuit)
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_get_runtime_works() {
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";

    CoprocessorClient::default()
        .get_runtime(circuit)
        .await
        .unwrap();
}

#[tokio::test]
async fn coprocessor_entrypoint_works() {
    let circuit = "a5be3bdb449527c80dd493ad5566f6d3f95f84c00c7c38f4189d3c3c5b53f16d";
    let args = serde_json::json!({
        "payload": {
            "cmd": "store",
            "path": "/etc/foo.bin",
        }
    });

    CoprocessorClient::default()
        .entrypoint(circuit, &args)
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
