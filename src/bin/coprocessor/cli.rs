use clap::{Parser, Subcommand};
use valence_domain_clients::clients::coprocessor::CoprocessorClient;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Socket address of the co-processor.
    #[arg(short, long, value_name = "SOCKET", default_value = CoprocessorClient::DEFAULT_COPROCESSOR)]
    pub socket: String,

    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deploys a circuit.
    Deploy {
        /// Path to the valence manifest file.
        #[arg(short, long, value_name = "MANIFEST", default_value = "valence.toml")]
        manifest: String,

        /// Circuit name. If absent, will deploy all circuits.
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,
    },

    /// Downloads a circuit.
    Download {
        /// Path to the valence manifest file.
        #[arg(short, long, value_name = "MANIFEST", default_value = "valence.toml")]
        manifest: Option<String>,

        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,
    },

    /// Get co-processor data.
    #[command(subcommand)]
    Get(CmdGet),

    /// Migrates an app from the provided source instance.
    Migrate {
        /// Source co-processor socket
        #[arg(short, long, value_name = "SOURCE", default_value = CoprocessorClient::DEFAULT_COPROCESSOR)]
        source: String,

        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,
    },

    /// Proves the circuit with the provided arguments.
    Prove {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,

        /// Arguments
        #[arg(short, long, value_name = "ARGS")]
        args: String,

        /// Debug witnesses computation.
        #[arg(long, value_name = "DEBUG")]
        debug: bool,
    },

    /// Prover scheduler for the client.
    #[command(subcommand)]
    Provers(CmdProvers),
}

#[derive(Subcommand)]
pub enum CmdGet {
    /// Get the circuit base64 bytecode.
    Circuit {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,
    },

    /// Get the controller runtime base64 bytecode.
    Runtime {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,
    },

    /// Get the raw VK base64 bytes.
    Vk {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,
    },
}

#[derive(Subcommand)]
pub enum CmdProvers {
    /// Get the available provers for the client.
    Get,

    /// Adds a new scheduled prover for the client.
    Add {
        /// Address of the prover.
        #[arg(
            short,
            long,
            value_name = "CIRCUIT",
            default_value = "wss://prover.coprocessor.valence.zone"
        )]
        address: String,
    },

    /// Removes a scheduled prover for the client.
    Remove {
        /// Address of the prover.
        #[arg(
            short,
            long,
            value_name = "CIRCUIT",
            default_value = "wss://prover.coprocessor.valence.zone"
        )]
        address: String,
    },
}
