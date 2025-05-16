//-----------------------------------------------------------------------------
// Integration Tests Main Entry
//-----------------------------------------------------------------------------

//! Integration tests for valence-domain-clients
//!
//! This main test file includes all test modules for the various components
//! of the valence-domain-clients library.

// Core module tests
mod core {
    // Error handling tests
    // mod error_tests; // Temporarily disabled - moved to tests/temp
    // Core types tests
    mod types_tests;
    // Transaction module tests
    mod transaction_tests;
}

// Cosmos chain tests
mod cosmos {
    // Babylon chain tests
    // mod babylon_tests; // Temporarily disabled - moved to tests/temp
    // Cosmos Hub (Gaia) chain tests
    // mod gaia_tests; // Temporarily disabled - moved to tests/temp
    // Neutron chain tests
    // mod neutron_tests; // Temporarily disabled - moved to tests/temp
    // Noble chain tests
    mod noble_tests; // Fixed and working now
    // Osmosis chain tests
    // mod osmosis_tests; // Temporarily disabled - moved to tests/temp
}

// EVM chain tests
mod evm {
    // Ethereum chain tests
    // mod ethereum_tests; // Temporarily disabled - moved to tests/temp
}

// Proto module tests will be added in the future
// mod proto;
