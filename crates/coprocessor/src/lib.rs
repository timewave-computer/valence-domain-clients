// Client library for interacting with the Valence coprocessor service

use std::fs;
use std::path::Path;
use std::net::SocketAddr;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use thiserror::Error;
use url::Url;

/// Errors that can occur when interacting with the coprocessor service
#[derive(Error, Debug)]
pub enum CoprocessorError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to decode response: {0}")]
    DecodingError(String),
    
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("No data received from service")]
    NoDataReceived,
    
    #[error("Invalid data received from service")]
    InvalidDataReceived,
}

/// Configuration for the coprocessor client
#[derive(Debug, Clone)]
pub struct CoprocessorConfig {
    /// Socket address of the coprocessor service
    pub socket: SocketAddr,
    /// Base URL for the coprocessor API
    pub base_url: Url,
}

impl Default for CoprocessorConfig {
    fn default() -> Self {
        let socket = "127.0.0.1:37281".parse().unwrap();
        let base_url = format!("http://{socket}/api/registry").parse().unwrap();
        
        Self {
            socket,
            base_url,
        }
    }
}

/// Client for interacting with the Valence coprocessor service
#[derive(Debug, Clone)]
pub struct CoprocessorClient {
    /// HTTP client for making requests
    client: Client,
    /// Configuration for the client
    config: CoprocessorConfig,
}

impl Default for CoprocessorClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CoprocessorClient {
    /// Create a new coprocessor client with the default configuration
    pub fn new() -> Self {
        Self::with_config(CoprocessorConfig::default())
    }
    
    /// Create a new coprocessor client with a custom configuration
    pub fn with_config(config: CoprocessorConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
    
    /// Create a new coprocessor client with a custom socket address
    pub fn with_socket(socket: SocketAddr) -> Self {
        let base_url = format!("http://{socket}/api/registry").parse().unwrap();
        let config = CoprocessorConfig {
            socket,
            base_url,
        };
        
        Self::with_config(config)
    }
    
    /// Deploy a domain to the coprocessor
    pub fn deploy_domain(&self, name: &str, wasm_path: &Path) -> Result<String, CoprocessorError> {
        let bytes = fs::read(wasm_path)?;
        let lib = Base64.encode(bytes);
        let uri = format!("{}/domain", self.config.base_url);
        
        let response = self.client
            .post(uri)
            .json(&json!({
                "name": name,
                "lib": lib,
            }))
            .send()?
            .json::<Value>()?;
        
        response
            .get("domain")
            .ok_or(CoprocessorError::NoDataReceived)?
            .as_str()
            .ok_or(CoprocessorError::InvalidDataReceived)
            .map(|s| s.to_string())
    }
    
    /// Deploy a program to the coprocessor
    pub fn deploy_program(&self, wasm_path: &Path, elf_path: &Path, nonce: u64) -> Result<String, CoprocessorError> {
        let wasm = fs::read(wasm_path)?;
        let elf = fs::read(elf_path)?;
        let uri = format!("{}/program", self.config.base_url);
        
        let lib = Base64.encode(wasm);
        let circuit = Base64.encode(elf);
        
        let response = self.client
            .post(uri)
            .json(&json!({
                "lib": lib,
                "circuit": circuit,
                "nonce": nonce,
            }))
            .send()?
            .json::<Value>()?;
        
        response
            .get("program")
            .ok_or(CoprocessorError::NoDataReceived)?
            .as_str()
            .ok_or(CoprocessorError::InvalidDataReceived)
            .map(|s| s.to_string())
    }
    
    /// Submit a proof request to the coprocessor
    pub fn submit_proof_request(&self, program: &str, args: Option<Value>, path: &Path) -> Result<String, CoprocessorError> {
        let args = args.unwrap_or(Value::Null);
        let uri = format!("{}/program/{}/prove", self.config.base_url, program);
        
        let response = self.client
            .post(uri)
            .json(&json!({
                "args": args,
                "payload": {
                    "cmd": "store",
                    "path": path
                }
            }))
            .send()?
            .text()?;
        
        Ok(response)
    }
    
    /// Read a file from the coprocessor's storage
    pub fn read_storage(&self, program: &str, path: &Path) -> Result<String, CoprocessorError> {
        let uri = format!("{}/program/{}/storage/fs", self.config.base_url, program);
        
        let response = self.client
            .post(uri)
            .json(&json!({
                "path": path
            }))
            .send()?
            .json::<Value>()?;
        
        response
            .get("data")
            .ok_or(CoprocessorError::NoDataReceived)?
            .as_str()
            .ok_or(CoprocessorError::InvalidDataReceived)
            .map(|s| s.to_string())
    }
    
    /// Get the verification key for a program
    pub fn get_verification_key(&self, program: &str) -> Result<String, CoprocessorError> {
        let uri = format!("{}/program/{}/vk", self.config.base_url, program);
        
        let response = self.client
            .get(uri)
            .send()?
            .json::<Value>()?;
        
        response
            .get("base64")
            .ok_or(CoprocessorError::NoDataReceived)?
            .as_str()
            .ok_or(CoprocessorError::InvalidDataReceived)
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = CoprocessorConfig::default();
        assert_eq!(config.socket.to_string(), "127.0.0.1:37281");
        assert_eq!(config.base_url.to_string(), "http://127.0.0.1:37281/api/registry");
    }
    
    #[test]
    fn test_with_socket() {
        let socket: SocketAddr = "192.168.1.100:8080".parse().unwrap();
        let client = CoprocessorClient::with_socket(socket);
        assert_eq!(client.config.socket.to_string(), "192.168.1.100:8080");
        assert_eq!(client.config.base_url.to_string(), "http://192.168.1.100:8080/api/registry");
    }
    
    #[test]
    fn test_custom_config() {
        let socket: SocketAddr = "192.168.1.100:8080".parse().unwrap();
        let base_url: Url = "http://custom-url.example.com/api".parse().unwrap();
        let config = CoprocessorConfig {
            socket,
            base_url,
        };
        let client = CoprocessorClient::with_config(config);
        assert_eq!(client.config.socket.to_string(), "192.168.1.100:8080");
        assert_eq!(client.config.base_url.to_string(), "http://custom-url.example.com/api");
    }
    
    // The following tests would require a mock HTTP client, which is beyond
    // the scope of this simple test suite. In a real-world scenario, you would:
    // 1. Create a trait for the HTTP client interface
    // 2. Implement the trait for reqwest::blocking::Client
    // 3. Create a mock implementation for testing
    // 4. Inject the client implementation into CoprocessorClient
    
    // For now, we'll create simple file handling tests for our utilities
    
    #[test]
    fn test_file_error_handling() {
        let client = CoprocessorClient::new();
        let non_existent_path = PathBuf::from("/path/to/non-existent-file.wasm");
        
        // Testing deploy_domain with a non-existent file
        let result = client.deploy_domain("test-domain", &non_existent_path);
        assert!(result.is_err());
        
        // Verify it's an IoError
        match result {
            Err(CoprocessorError::IoError(_)) => { /* Test passed */ },
            _ => panic!("Expected IoError, got different error or success"),
        }
        
        // Testing deploy_program with non-existent files
        let result = client.deploy_program(&non_existent_path, &non_existent_path, 0);
        assert!(result.is_err());
        
        // Verify it's an IoError
        match result {
            Err(CoprocessorError::IoError(_)) => { /* Test passed */ },
            _ => panic!("Expected IoError, got different error or success"),
        }
    }
    
    #[test]
    fn test_file_reading_for_deploy() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        
        // Create test files
        let wasm_path = temp_dir.path().join("test.wasm");
        let elf_path = temp_dir.path().join("test.elf");
        
        {
            let mut wasm_file = File::create(&wasm_path)?;
            wasm_file.write_all(b"mock wasm content")?;
            
            let mut elf_file = File::create(&elf_path)?;
            elf_file.write_all(b"mock elf content")?;
        }
        
        // These tests won't actually make HTTP requests since we can't mock the client easily
        // We just verify that the files can be read correctly
        
        let bytes = fs::read(&wasm_path)?;
        assert_eq!(bytes, b"mock wasm content");
        
        let bytes = fs::read(&elf_path)?;
        assert_eq!(bytes, b"mock elf content");
        
        // Clean up
        temp_dir.close()?;
        
        Ok(())
    }
} 