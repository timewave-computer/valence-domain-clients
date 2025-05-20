//-----------------------------------------------------------------------------
// EVM Error Types
//-----------------------------------------------------------------------------

use thiserror::Error;

/// Error types for EVM operations
#[derive(Debug, Error)]
pub enum EvmError {
    /// Failed to connect to the node
    #[error("Failed to connect to node: {0}")]
    ConnectionError(String),

    /// Serialization or deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Contract interaction error
    #[error("Contract error: {0}")]
    ContractError(String),

    /// Insufficient balance for transaction
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// General client error
    #[error("Client error: {0}")]
    ClientError(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_creation() {
        // Test each error variant creation
        let connection_err =
            EvmError::ConnectionError("failed to connect".to_string());
        let serialization_err =
            EvmError::SerializationError("invalid format".to_string());
        let transaction_err = EvmError::TransactionError("tx failed".to_string());
        let contract_err = EvmError::ContractError("invalid contract".to_string());
        let balance_err =
            EvmError::InsufficientBalance("not enough ETH".to_string());
        let param_err = EvmError::InvalidParameter("bad parameter".to_string());
        let client_err = EvmError::ClientError("general error".to_string());
        let not_impl_err =
            EvmError::NotImplemented("feature not available".to_string());

        // Verify errors are created correctly
        assert!(matches!(connection_err, EvmError::ConnectionError(_)));
        assert!(matches!(serialization_err, EvmError::SerializationError(_)));
        assert!(matches!(transaction_err, EvmError::TransactionError(_)));
        assert!(matches!(contract_err, EvmError::ContractError(_)));
        assert!(matches!(balance_err, EvmError::InsufficientBalance(_)));
        assert!(matches!(param_err, EvmError::InvalidParameter(_)));
        assert!(matches!(client_err, EvmError::ClientError(_)));
        assert!(matches!(not_impl_err, EvmError::NotImplemented(_)));
    }

    #[test]
    fn test_error_messages() {
        // Test error messages for each variant
        let connection_err =
            EvmError::ConnectionError("failed to connect".to_string());
        assert_eq!(
            connection_err.to_string(),
            "Failed to connect to node: failed to connect"
        );

        let serialization_err =
            EvmError::SerializationError("invalid format".to_string());
        assert_eq!(
            serialization_err.to_string(),
            "Serialization error: invalid format"
        );

        let transaction_err = EvmError::TransactionError("tx failed".to_string());
        assert_eq!(transaction_err.to_string(), "Transaction error: tx failed");

        let contract_err = EvmError::ContractError("invalid contract".to_string());
        assert_eq!(contract_err.to_string(), "Contract error: invalid contract");

        let balance_err =
            EvmError::InsufficientBalance("not enough ETH".to_string());
        assert_eq!(
            balance_err.to_string(),
            "Insufficient balance: not enough ETH"
        );

        let param_err = EvmError::InvalidParameter("bad parameter".to_string());
        assert_eq!(param_err.to_string(), "Invalid parameter: bad parameter");

        let client_err = EvmError::ClientError("general error".to_string());
        assert_eq!(client_err.to_string(), "Client error: general error");

        let not_impl_err =
            EvmError::NotImplemented("feature not available".to_string());
        assert_eq!(
            not_impl_err.to_string(),
            "Not implemented: feature not available"
        );
    }

    #[test]
    fn test_error_trait_implementation() {
        // Verify EvmError implements the Error trait
        let err = EvmError::ClientError("test error".to_string());
        let dyn_err: &dyn Error = &err;

        // Test that error can be cast to trait object
        assert!(dyn_err.source().is_none());
        assert_eq!(dyn_err.to_string(), "Client error: test error");
    }
}
