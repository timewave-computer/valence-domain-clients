// Client library for interacting with the Valence coprocessor service

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use valence_coprocessor_client::CoprocessorClient;

/// Command line arguments for the coprocessor client example
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the domain WASM file to deploy
    #[clap(short, long, value_parser)]
    domain_wasm: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> { // Changed to sync main
    let args = Args::parse();

    let client = CoprocessorClient::new();

    if let Some(domain_wasm_path) = args.domain_wasm {
        if domain_wasm_path.exists() {
            println!("Deploying domain from: {domain_wasm_path:?}");
            // Using file name as domain name for simplicity, adjust if needed
            let domain_name = domain_wasm_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("default_domain_name");

            match client.deploy_domain(domain_name, &domain_wasm_path) { // Corrected call, removed .await
                Ok(domain_id) => { // Adjusted to handle Result<String, _>
                    println!("Domain deployed with ID: {domain_id}")
                }
                Err(e) => println!("Failed to deploy domain: {e}"),
            }
        } else {
            println!("Domain WASM file not found at: {domain_wasm_path:?}");
        }
    } else {
        println!("No domain WASM file provided. Use --domain-wasm <PATH>");
    }

    Ok(())
} 