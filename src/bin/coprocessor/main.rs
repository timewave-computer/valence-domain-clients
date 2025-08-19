use clap::Parser as _;
use cli::Cli;
use valence_domain_clients::clients::coprocessor::CoprocessorClient;

use crate::cli::{CmdGet, Commands};

mod app;
mod cli;
mod macros;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli { socket, cmd } = Cli::parse();

    let client = CoprocessorClient::new(socket);

    if let Ok(signer) = valence_crypto_utils::Signer::try_from_env() {
        info!("Using valence signer `{signer}`...");
    }

    use Commands as C;

    let ret = match cmd {
        C::Build {
            manifest,
            name: Some(name),
            only_controller,
        } => app::build(&manifest, &name, only_controller)?,

        C::Build {
            manifest,
            name: None,
            only_controller,
        } => app::build_all(&manifest, only_controller)?,

        C::Deploy {
            manifest,
            name: Some(name),
        } => app::deploy(&client, &manifest, &name).await?,

        C::Deploy {
            manifest,
            name: None,
        } => app::deploy_all(&client, &manifest).await?,

        C::Download { manifest, circuit } => {
            app::download(&client, manifest.as_deref(), &circuit).await?
        }

        C::Get(CmdGet::Circuit { circuit }) => app::get_circuit(&client, &circuit).await?,

        C::Get(CmdGet::Runtime { circuit }) => app::get_runtime(&client, &circuit).await?,

        C::Get(CmdGet::Vk { circuit }) => app::get_vk(&client, &circuit).await?,

        C::Prove { circuit, args } => app::prove(&client, &circuit, &args).await?,

        C::Witnesses { circuit, args } => app::witnesses(&client, &circuit, &args).await?,
    };

    println!("{}", serde_json::to_string(&ret)?);

    Ok(())
}
