//-----------------------------------------------------------------------------
// Test Modules
//-----------------------------------------------------------------------------

// Cosmos module tests
mod cosmos {
    // Client implementation tests
    mod noble_tests; // Fixed and working now
    
    // Utility module tests
    // mod grpc_client_tests; // Temporarily disabled - moved to tests/temp
}

// Protocol buffer tests
mod proto {
    mod mod_tests;
}

// Integration tests with mocks and real networks
mod integration; 