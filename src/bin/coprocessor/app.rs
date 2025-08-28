use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::{json, Value};
use valence_domain_clients::{
    clients::coprocessor::CoprocessorClient,
    coprocessor::{
        base_client::{Base64, CoprocessorBaseClient as _},
        Manifest,
    },
};

use crate::{error, info};

pub async fn deploy(
    client: &CoprocessorClient,
    manifest: &str,
    name: &str,
) -> anyhow::Result<Value> {
    let (dir, manifest) = Manifest::load_from_path(manifest)?;

    fn _load(manifest: &Manifest, dir: &Path, name: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let artifacts = dir
            .join(&manifest.valence.artifacts)
            .join(name)
            .canonicalize()?;

        info!("Loading artifacts from `{}`...", artifacts.display());

        let circuit = fs::read(artifacts.join("circuit.bin"))?;
        let controller = fs::read(artifacts.join("controller.bin"))?;

        Ok((circuit, controller))
    }

    let (circuit, controller) = _load(&manifest, &dir, name).inspect_err(|_| {
        error!("Artifacts failed to load! Did you forget to build?");
    })?;

    info!("Artifacts loaded; deploying...");

    let id = client
        .deploy_controller(&controller, &circuit, None)
        .await?;

    Ok(Value::String(id))
}

pub async fn download(
    client: &CoprocessorClient,
    path: Option<&str>,
    id: &str,
) -> anyhow::Result<Value> {
    let (dir, manifest) = match path {
        Some(m) => Manifest::load_from_path(m)?,
        None => {
            info!("No manifest provided; searching on current structure...");

            let root = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .stderr(Stdio::inherit())
                .output()?;

            anyhow::ensure!(root.status.success(), "No git root found.");

            let root = String::from_utf8(root.stdout)?;
            let root = PathBuf::from(root.trim()).canonicalize()?;
            let manifest = root.join("valence.toml");

            info!("Git root found at `{}`...", root.display());

            if !manifest.is_file() {
                info!("No manifest found; creating...");

                let template = Manifest::default();
                let contents = toml::to_string_pretty(&template)?;

                fs::write(&manifest, contents)?;
            }

            Manifest::load_from_path(&manifest)?
        }
    };

    let artifacts = dir.join(&manifest.valence.artifacts).join(id);

    fs::create_dir_all(&artifacts)?;

    let artifacts = artifacts.canonicalize()?;
    let out_cir = artifacts.join("circuit.bin");
    let out_con = artifacts.join("controller.bin");

    info!("Storing artifacts at `{}`...", artifacts.display());

    let circuit = client.get_circuit(id).await?;
    let controller = client.get_runtime(id).await?;

    fs::write(&out_cir, circuit)?;
    fs::write(&out_con, controller)?;

    info!("Artifacts stored; updating manifest...");

    let manifest = manifest.with_circuit(id, None, None);
    let manifest = toml::to_string_pretty(&manifest)?;
    let path = match path {
        Some(p) => PathBuf::from(p),
        None => dir.join("valence.toml"),
    };

    fs::write(path, manifest)?;

    Ok(json!({
        "circuit": out_cir,
        "controller": out_con,
    }))
}

pub async fn deploy_all(client: &CoprocessorClient, manifest: &str) -> anyhow::Result<Value> {
    let m = fs::read_to_string(manifest)?;
    let m: Manifest = toml::from_str(&m)?;
    let names: Vec<String> = m.circuit.keys().map(|name| name.into()).collect();
    let mut resources = Vec::with_capacity(names.len());

    for n in names {
        let id = deploy(client, manifest, &n).await?;

        resources.push(json!({
            "name": n,
            "id": id,
        }));
    }

    Ok(json!(resources))
}

pub async fn get_circuit(client: &CoprocessorClient, circuit: &str) -> anyhow::Result<Value> {
    info!("Fetching base64 circuit bytecode...");

    let circuit = client.get_circuit(circuit).await?;
    let circuit = Base64::encode(circuit);

    Ok(Value::String(circuit))
}

pub async fn get_runtime(client: &CoprocessorClient, circuit: &str) -> anyhow::Result<Value> {
    info!("Fetching base64 runtime bytecode...");

    let runtime = client.get_runtime(circuit).await?;
    let runtime = Base64::encode(runtime);

    Ok(Value::String(runtime))
}

pub async fn get_vk(client: &CoprocessorClient, circuit: &str) -> anyhow::Result<Value> {
    info!("Fetching base64 vk...");

    let vk = client.get_vk(circuit).await?;
    let vk = Base64::encode(vk);

    Ok(Value::String(vk))
}

pub async fn prove(client: &CoprocessorClient, circuit: &str, args: &str) -> anyhow::Result<Value> {
    info!("Proving...");

    let args = serde_json::from_str(args)?;
    let proof = client.prove(circuit, &args).await?;

    Ok(serde_json::to_value(proof)?)
}

pub async fn witnesses(
    client: &CoprocessorClient,
    circuit: &str,
    args: &str,
) -> anyhow::Result<Value> {
    info!("Computing witnesses...");

    let args = serde_json::from_str(args)?;
    let witnesses = client.get_witnesses(circuit, &args).await?;

    Ok(serde_json::to_value(witnesses)?)
}
