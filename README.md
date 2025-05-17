# Valence Domain Clients

A Rust workspace containing clients for various blockchain domains, supporting both Cosmos and EVM ecosystems.

## Workspace Structure

The workspace is organized into the following crates:

- **valence-core**: Core types and utilities shared across all domain clients
- **valence-cosmos**: Clients for Cosmos-based blockchains
- **valence-evm**: Clients for Ethereum and other EVM-compatible blockchains
- **valence-proto**: Protocol buffer definitions and utilities
- **valence-domain-clients**: The main package that re-exports functionality from the domain-specific crates

## Dependency Isolation

The `valence-proto` crate abstracts away specific dependency versions like `prost` from consumers. This is achieved through:

- Custom traits (`ProtoEncodable` and `ProtoDecodable`) that provide a stable interface
- Hiding implementation details behind these traits
- Preventing dependency conflicts when projects use different versions of the same libraries

This design ensures that applications consuming this library won't have dependency conflicts, even if they use different versions of the underlying libraries.

## Crate Descriptions

### valence-core

Contains core functionality shared by all domain clients:
- Error types
- Transaction types
- Common traits
- Utility functions

### valence-cosmos

Contains clients for Cosmos-based blockchains:
- Cosmos SDK client base functionality
- Chain-specific clients (Gaia, Osmosis, Noble, Neutron, Babylon)
- GRPC and signing client implementations
- Utilities for working with Cosmos transactions

### valence-evm

Contains clients for EVM-compatible blockchains:
- Ethereum client implementations
- Base EVM client functionality
- Flashbots bundle support
- Utilities for working with EVM transactions

### valence-proto

Contains protocol buffer definitions:
- Generated Rust code from proto files
- Utilities for encoding/decoding proto messages
- Dependency shielding through abstraction traits

## Development

### Requirements

This project uses Nix to manage dependencies. Make sure you have Nix installed and enable flakes.

### Building

```bash
# Enter the development environment
nix develop

# Build all crates
cargo build
```

### Testing

```bash
# Run tests for all crates
cargo test
```

## Usage Examples

Several examples are provided in the `examples/` directory:

- **osmosis_dex.rs**: Demonstrates interacting with the Osmosis DEX
- **noble_example.rs**: Shows how to work with the Noble blockchain
- **flashbots_bundle.rs**: Example of creating and submitting Flashbots bundles
- **base_example.rs**: Basic interactions with Base (an EVM-compatible L2)
- **cross_chain_transfer.rs**: Example of cross-chain asset transfers

To run an example:

```bash
cargo run --example osmosis_dex
```

## Features

- Full support for Cosmos-based blockchains
- EVM blockchain support (Ethereum, Base, Optimism, etc.)
- Flashbots bundle integration
- Type-safe transaction building
- Comprehensive error handling
- Protocol buffer dependency isolation
- Cross-chain interactions
