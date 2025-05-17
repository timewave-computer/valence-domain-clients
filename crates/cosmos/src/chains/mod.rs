//-----------------------------------------------------------------------------
// Cosmos Chain Implementations
//-----------------------------------------------------------------------------

//! Chain-specific implementations for Cosmos ecosystem blockchains.
//!
//! This module contains client implementations for various Cosmos-based chains.

// Babylon
#[cfg(feature = "babylon")]
pub mod babylon;

// Cosmos Hub (Gaia)
#[cfg(feature = "gaia")]
pub mod gaia;

// Neutron
#[cfg(feature = "neutron")]
pub mod neutron;

// Noble
#[cfg(feature = "noble")]
pub mod noble;

// Osmosis
#[cfg(feature = "osmosis")]
pub mod osmosis;

// Re-export client structs for direct imports
pub use babylon::BabylonClient;
pub use gaia::GaiaClient;
pub use neutron::NeutronClient;
pub use noble::NobleClient;
pub use osmosis::OsmosisClient;
