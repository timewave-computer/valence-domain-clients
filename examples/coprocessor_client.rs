use std::path::PathBuf;
use std::net::SocketAddr;

use valence_coprocessor_client::CoprocessorClient;

fn main() -> anyhow::Result<()> {
    // Create a client with default configuration (localhost:37281)
    let _client = CoprocessorClient::new();
    
    // Or create with a custom socket address
    let socket: SocketAddr = "127.0.0.1:37281".parse()?;
    let client = CoprocessorClient::with_socket(socket);
    
    // Example paths (these would need to point to your actual files)
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // Example of deploying a domain
    let domain_wasm_path = base_dir
        .join("path")
        .join("to")
        .join("domain-wasm.wasm");
    
    if domain_wasm_path.exists() {
        println!("Deploying domain...");
        match client.deploy_domain("example-domain", &domain_wasm_path) {
            Ok(domain_id) => println!("Domain deployed with ID: {}", domain_id),
            Err(e) => println!("Failed to deploy domain: {}", e),
        }
    } else {
        println!("Domain WASM file not found at: {:?}", domain_wasm_path);
        println!("This is expected in this example as the path is placeholder.");
    }
    
    // Example of deploying a program
    let _program_wasm_path = base_dir
        .join("path")
        .join("to")
        .join("program.wasm");
    
    let _program_elf_path = base_dir
        .join("path")
        .join("to")
        .join("program.elf");
    
    // Since these files don't exist, we'll just show the API usage
    println!("Program deployment example (these files don't exist):");
    println!("client.deploy_program(&program_wasm_path, &program_elf_path, 0)");
    
    // Example of submitting a proof request
    println!("\nProof request example:");
    println!("client.submit_proof_request(\"program_id\", Some(json!({{\"value\": 42}})), &PathBuf::from(\"/var/share/proof.bin\"))");
    
    // Example of reading from storage
    println!("\nStorage read example:");
    println!("client.read_storage(\"program_id\", &PathBuf::from(\"/var/share/proof.bin\"))");
    
    // Example of getting the verification key
    println!("\nVerification key example:");
    println!("client.get_verification_key(\"program_id\")");
    
    Ok(())
} 