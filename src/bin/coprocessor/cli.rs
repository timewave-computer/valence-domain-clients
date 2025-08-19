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
    /// Builds a circuit.
    Build {
        /// Path to the valence manifest file.
        #[arg(short, long, value_name = "MANIFEST", default_value = "valence.toml")]
        manifest: String,

        /// Circuit name. If absent, will compile all circuits.
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,

        /// Build only controller?
        #[arg(short, long, value_name = "ONLY_CONTROLLER")]
        only_controller: bool,
    },

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

    /// Proves the circuit with the provided arguments.
    Prove {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,

        /// Arguments
        #[arg(short, long, value_name = "ARGS")]
        args: String,
    },

    /// Prints the computed witnesses for the provided arguments.
    Witnesses {
        /// Circuit ID
        #[arg(short, long, value_name = "CIRCUIT")]
        circuit: String,

        /// Arguments
        #[arg(short, long, value_name = "ARGS")]
        args: String,
    },
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
