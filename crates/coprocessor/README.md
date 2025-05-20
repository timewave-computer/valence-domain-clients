# Valence Coprocessor Client

A Rust client library for interacting with the Valence coprocessor service. This library abstracts the HTTP API calls and provides a clean interface for the main operations.

## Features

- Deploy domain and program definitions to the coprocessor
- Submit proof requests
- Read from the coprocessor's virtual filesystem
- Get verification keys for programs

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
# Using the client from GitHub
valence-coprocessor-client = { git = "https://github.com/timewave-computer/valence-domain-clients.git" }

# If you're building coprocessor applications, you'll likely need these dependencies as well
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.9" }
valence-coprocessor-wasm = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.9" }
```

### Basic Example

```rust
use std::path::PathBuf;
use valence_coprocessor_client::CoprocessorClient;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    // Create a client with the default configuration (127.0.0.1:37281)
    let client = CoprocessorClient::new();
    
    // Deploy a program
    let program_id = client.deploy_program(
        &PathBuf::from("path/to/program.wasm"),
        &PathBuf::from("path/to/program.elf"),
        0
    )?;
    
    // Submit a proof request
    let proof_path = PathBuf::from("/var/share/proof.bin");
    let response = client.submit_proof_request(
        &program_id,
        Some(json!({"value": 42})),
        &proof_path
    )?;
    
    // Read from storage
    let data = client.read_storage(&program_id, &proof_path)?;
    
    // Get verification key
    let vk = client.get_verification_key(&program_id)?;
    
    Ok(())
}
```

See the top-level `examples/coprocessor_client.rs` file for a complete usage example.

## Configuration

The client can be configured in several ways:

```rust
// Default configuration (127.0.0.1:37281)
let client = CoprocessorClient::new();

// Custom socket address
let socket: SocketAddr = "192.168.1.100:8080".parse()?;
let client = CoprocessorClient::with_socket(socket);

// Fully custom configuration
let config = CoprocessorConfig {
    socket: "192.168.1.100:8080".parse()?,
    base_url: "http://192.168.1.100:8080/api/registry".parse()?,
};
let client = CoprocessorClient::with_config(config);
```

## Related Projects

This client is designed to work with the following projects:

- [valence-coprocessor](https://github.com/timewave-computer/valence-coprocessor) - The core coprocessor service that this client interacts with
- [valence-coprocessor-app](https://github.com/timewave-computer/valence-coprocessor-app) - Template application for building coprocessor applications 