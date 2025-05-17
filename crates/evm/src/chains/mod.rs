//-----------------------------------------------------------------------------
// EVM Chain Implementations
//-----------------------------------------------------------------------------

// Base (Ethereum L2)
#[cfg(feature = "base")]
pub mod base;

// Ethereum mainnet and testnets
#[cfg(feature = "ethereum")]
pub mod ethereum;
