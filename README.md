# Valence Domain Clients

Client implementations for interacting with Valence Protocol domains across multiple blockchain ecosystems.

## Project Structure

The project is organized into the following main directories:

- `crates/` - Main source code for the client libraries
  - `core/` - Core types, errors, and utilities shared by all clients
  - `cosmos/` - Cosmos ecosystem client implementations
  - `evm/` - Ethereum Virtual Machine ecosystem client implementations 
  - `proto/` - Protocol buffer definitions and utilities

- `tests/` - Test files for the client implementations
  - `cosmos/` - Tests for Cosmos clients
  - `evm/` - Tests for EVM clients
  - `integration/` - Cross-chain integration tests
  - `proto/` - Tests for protocol buffer utilities

## Features

The library supports the following features:

- **Cosmos ecosystem**:
  - Noble
  - Osmosis
  - Gaia (Cosmos Hub)
  - Neutron
  - Babylon

- **EVM ecosystem**:
  - Ethereum
  - Base

## Building

This project uses Nix for reproducible builds. To build the project:

```bash
# Enter the Nix development shell
nix develop

# Build the project
cargo build

# Run tests
cargo test
```

## Testing Strategy

The project employs several levels of testing:

### Unit Tests

Unit tests are embedded within the source files and focus on testing individual components in isolation. These can be run with:

```bash
cargo test --lib
```

### Integration Tests with Mocks

These tests use mock implementations of blockchain clients to test the interaction logic without connecting to real networks:

```bash
cargo test --test integration::blockchain_mocks
```

### Real Network Tests

Tests that connect to actual blockchain networks. These are skipped by default unless specific environment variables are set:

```bash
# To run real network tests
RUN_NETWORK_TESTS=1 cargo test --test integration::network_tests

# To also run tests that perform actual transfers
RUN_NETWORK_TESTS=1 RUN_TRANSFER_TESTS=1 cargo test --test integration::network_tests

# To run IBC cross-chain tests
RUN_NETWORK_TESTS=1 RUN_IBC_TESTS=1 cargo test --test integration::network_tests
```

For the real network tests, you need to set additional environment variables:

```bash
# Cosmos chain testing
export NOBLE_TEST_MNEMONIC="your test mnemonic"
export OSMOSIS_TEST_MNEMONIC="your test mnemonic"
export NOBLE_TEST_RECIPIENT="optional recipient address"

# EVM chain testing
export ETH_TEST_PRIVATE_KEY="your test private key without 0x prefix"
export ETH_TEST_RPC_URL="optional custom RPC URL"
```

## Coverage

Generate code coverage reports using cargo-tarpaulin:

```bash
nix develop -c cargo tarpaulin --skip-clean -p valence-domain-clients --lib
```

## License

This project is licensed under the Apache License, Version 2.0.
