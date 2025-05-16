//-----------------------------------------------------------------------------
// Integration Tests Module
//-----------------------------------------------------------------------------
//
// This module organizes all integration tests that interact with either:
// 1. Mock blockchain implementations
// 2. Real blockchain networks
//
// To run the real network tests, set the following environment variables:
// - RUN_NETWORK_TESTS=1 - Required for any real network tests
// - RUN_TRANSFER_TESTS=1 - Required for tests that send actual transactions
// - RUN_IBC_TESTS=1 - Required for cross-chain IBC tests
//
// Chain-specific environment variables:
// For Cosmos chains:
// - NOBLE_TEST_MNEMONIC="your test mnemonic"
// - OSMOSIS_TEST_MNEMONIC="your test mnemonic"
// - NOBLE_TEST_RECIPIENT="optional recipient address"
//
// For EVM chains:
// - ETH_TEST_PRIVATE_KEY="your test private key without 0x prefix"
// - ETH_TEST_RPC_URL="optional custom RPC URL"

// Mock implementation with in-memory state
// pub mod blockchain_mocks; // Temporarily disabled - moved to tests/temp

// Real network tests that connect to actual blockchain networks
// pub mod network_tests; // Temporarily disabled - moved to tests/temp 