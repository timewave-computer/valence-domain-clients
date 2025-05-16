//-----------------------------------------------------------------------------
// Cosmos Chains Module
//-----------------------------------------------------------------------------

//! Chain-specific Cosmos client implementations.
//! 
//! This module contains client implementations for specific Cosmos-based
//! blockchains, each leveraging the shared functionality provided by
//! the base Cosmos client while adding chain-specific features.

// Re-export chain-specific clients
pub mod babylon;
pub mod gaia;
pub mod noble;
pub mod neutron;
pub mod osmosis;

// Re-export client structs for direct imports
pub use babylon::BabylonClient;
pub use gaia::GaiaClient;
pub use noble::NobleClient;
pub use neutron::NeutronClient;
pub use osmosis::OsmosisClient;
