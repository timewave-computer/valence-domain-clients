//-----------------------------------------------------------------------------
// Erigon Tracing API Implementation
//-----------------------------------------------------------------------------

//! Erigon tracing API support for enhanced transaction and block analysis.
//!
//! This module provides tracing capabilities that are compatible with the Erigon
//! Ethereum client, allowing for detailed analysis of transaction execution,
//! state changes, and VM operations.

#[cfg(feature = "erigon-tracing")]
use async_trait::async_trait;

#[cfg(feature = "erigon-tracing")]
use crate::types::{
    BlockTrace, CallTraceRequest, EvmAddress, EvmHash, TraceFilter, TraceType,
    TransactionTrace,
};

#[cfg(feature = "erigon-tracing")]
use valence_core::error::ClientError;

/// Trait for Erigon tracing API operations
#[cfg(feature = "erigon-tracing")]
#[async_trait]
pub trait ErigonTracing {
    /// Trace a specific transaction by hash
    ///
    /// # Arguments
    /// * `tx_hash` - The transaction hash to trace
    /// * `trace_types` - Types of traces to include (trace, vmTrace, stateDiff)
    ///
    /// # Returns
    /// Complete trace information for the transaction
    async fn trace_transaction(
        &self,
        tx_hash: &EvmHash,
        trace_types: Vec<TraceType>,
    ) -> Result<TransactionTrace, ClientError>;

    /// Trace all transactions in a specific block
    ///
    /// # Arguments
    /// * `block_number` - The block number to trace
    /// * `trace_types` - Types of traces to include
    ///
    /// # Returns
    /// Complete trace information for all transactions in the block
    async fn trace_block(
        &self,
        block_number: u64,
        trace_types: Vec<TraceType>,
    ) -> Result<BlockTrace, ClientError>;

    /// Filter traces based on criteria
    ///
    /// # Arguments
    /// * `filter` - Filter criteria for traces
    ///
    /// # Returns
    /// Traces matching the filter criteria
    async fn trace_filter(
        &self,
        filter: &TraceFilter,
    ) -> Result<Vec<TransactionTrace>, ClientError>;

    /// Trace a call without executing it on-chain
    ///
    /// # Arguments
    /// * `call_request` - The call to trace
    /// * `trace_types` - Types of traces to include
    /// * `block_number` - Block number to execute the call at (None for latest)
    ///
    /// # Returns
    /// Trace of the call execution
    async fn trace_call(
        &self,
        call_request: &CallTraceRequest,
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<TransactionTrace, ClientError>;

    /// Trace multiple calls in sequence
    ///
    /// # Arguments
    /// * `call_requests` - Vector of calls to trace
    /// * `trace_types` - Types of traces to include
    /// * `block_number` - Block number to execute the calls at (None for latest)
    ///
    /// # Returns
    /// Traces of all call executions
    async fn trace_call_many(
        &self,
        call_requests: &[CallTraceRequest],
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<Vec<TransactionTrace>, ClientError>;

    /// Trace a raw transaction
    ///
    /// # Arguments
    /// * `raw_tx` - The raw transaction bytes
    /// * `trace_types` - Types of traces to include
    /// * `block_number` - Block number to execute the transaction at (None for latest)
    ///
    /// # Returns
    /// Trace of the transaction execution
    async fn trace_raw_transaction(
        &self,
        raw_tx: &[u8],
        trace_types: Vec<TraceType>,
        block_number: Option<u64>,
    ) -> Result<TransactionTrace, ClientError>;

    /// Replay transactions in a block with tracing
    ///
    /// # Arguments
    /// * `block_number` - The block number to replay
    /// * `trace_types` - Types of traces to include
    ///
    /// # Returns
    /// Traces of all replayed transactions
    async fn trace_replay_block_transactions(
        &self,
        block_number: u64,
        trace_types: Vec<TraceType>,
    ) -> Result<Vec<TransactionTrace>, ClientError>;

    /// Replay a specific transaction with tracing
    ///
    /// # Arguments
    /// * `tx_hash` - The transaction hash to replay
    /// * `trace_types` - Types of traces to include
    ///
    /// # Returns
    /// Trace of the replayed transaction
    async fn trace_replay_transaction(
        &self,
        tx_hash: &EvmHash,
        trace_types: Vec<TraceType>,
    ) -> Result<TransactionTrace, ClientError>;
}

/// Helper functions for constructing trace requests
#[cfg(feature = "erigon-tracing")]
pub mod helpers {
    use super::*;

    /// Create a basic trace filter for a specific address range
    pub fn create_address_filter(
        from_block: u64,
        to_block: u64,
        addresses: Vec<EvmAddress>,
    ) -> TraceFilter {
        TraceFilter {
            from_block: Some(from_block),
            to_block: Some(to_block),
            from_address: Some(addresses.clone()),
            to_address: Some(addresses),
            after: None,
            count: None,
        }
    }

    /// Create a trace filter for recent blocks
    pub fn create_recent_filter(last_n_blocks: u64, latest_block: u64) -> TraceFilter {
        let from_block = latest_block.saturating_sub(last_n_blocks);

        TraceFilter {
            from_block: Some(from_block),
            to_block: Some(latest_block),
            from_address: None,
            to_address: None,
            after: None,
            count: None,
        }
    }

    /// Create a paginated trace filter
    pub fn create_paginated_filter(
        from_block: u64,
        to_block: u64,
        offset: u64,
        limit: u64,
    ) -> TraceFilter {
        TraceFilter {
            from_block: Some(from_block),
            to_block: Some(to_block),
            from_address: None,
            to_address: None,
            after: Some(offset),
            count: Some(limit),
        }
    }

    /// Get all trace types (comprehensive tracing)
    pub fn all_trace_types() -> Vec<TraceType> {
        vec![TraceType::Trace, TraceType::VmTrace, TraceType::StateDiff]
    }

    /// Get basic trace types (just call traces)
    pub fn basic_trace_types() -> Vec<TraceType> {
        vec![TraceType::Trace]
    }

    /// Get VM trace types (calls + VM execution details)
    pub fn vm_trace_types() -> Vec<TraceType> {
        vec![TraceType::Trace, TraceType::VmTrace]
    }

    /// Get state diff trace types (calls + state changes)
    pub fn state_diff_trace_types() -> Vec<TraceType> {
        vec![TraceType::Trace, TraceType::StateDiff]
    }
}

#[cfg(feature = "erigon-tracing")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EvmAddress;
    use std::str::FromStr;

    #[test]
    fn test_create_address_filter() {
        let address1 = EvmAddress::from_str("0x742d35Cc6634C0532925a3b844Bc454e4438f44e").unwrap();
        let address2 = EvmAddress::from_str("0x8a3106a3e50576d4b6794a0e74d3bb5f8c9deda0").unwrap();
        
        let filter = helpers::create_address_filter(100, 200, vec![address1.clone(), address2.clone()]);
        
        assert_eq!(filter.from_block, Some(100));
        assert_eq!(filter.to_block, Some(200));
        assert_eq!(filter.from_address.as_ref().unwrap().len(), 2);
        assert_eq!(filter.to_address.as_ref().unwrap().len(), 2);
        assert!(filter.from_address.as_ref().unwrap().contains(&address1));
        assert!(filter.to_address.as_ref().unwrap().contains(&address2));
    }

    #[test]
    fn test_create_recent_filter() {
        let filter = helpers::create_recent_filter(10, 100);
        
        assert_eq!(filter.from_block, Some(90));
        assert_eq!(filter.to_block, Some(100));
        assert!(filter.from_address.is_none());
        assert!(filter.to_address.is_none());
    }

    #[test]
    fn test_create_paginated_filter() {
        let filter = helpers::create_paginated_filter(100, 200, 50, 25);
        
        assert_eq!(filter.from_block, Some(100));
        assert_eq!(filter.to_block, Some(200));
        assert_eq!(filter.after, Some(50));
        assert_eq!(filter.count, Some(25));
    }

    #[test]
    fn test_trace_types() {
        assert_eq!(helpers::all_trace_types().len(), 3);
        assert_eq!(helpers::basic_trace_types().len(), 1);
        assert_eq!(helpers::vm_trace_types().len(), 2);
        assert_eq!(helpers::state_diff_trace_types().len(), 2);
    }
} 