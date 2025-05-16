//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Error type to be returned by all client types.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("client error: {0}")]
    ClientError(String),
    
    #[error("query error: {0}")]
    QueryError(String),
    
    #[error("parse error: {0}")]
    ParseError(String),
    
    #[error("transaction error: {0}")]
    TransactionError(String),
    
    #[error("not implemented: {0}")]
    NotImplemented(String),
    
    #[error("timeout error: {0}")]
    TimeoutError(String),
    
    #[error("serialization error: {0}")]
    SerializationError(String),
    
    #[error("configuration error: {0}")]
    ConfigError(String),
    
    #[error("resource not found: {0}")]
    NotFoundError(String),
    
    #[error("state mismatch: {0}")]
    StateMismatch(String),
    
    #[error("action failed: {0}")]
    ActionFailed(String),
    
    #[error("contract error: {0}")]
    ContractError(String),
    
    #[error("ABI encoding/decoding error: {0}")]
    AbiError(String),
    
    #[error("invalid message type: {0}")]
    InvalidMsgType(String),
}

//-----------------------------------------------------------------------------
// Error Conversions
//-----------------------------------------------------------------------------

/// Protocol buffer encode error conversion
impl From<prost::EncodeError> for ClientError {
    fn from(err: prost::EncodeError) -> Self {
        ClientError::SerializationError(format!("Protocol buffer encoding error: {}", err))
    }
}

/// Protocol buffer decode error conversion
impl From<prost::DecodeError> for ClientError {
    fn from(err: prost::DecodeError) -> Self {
        ClientError::SerializationError(format!("Protocol buffer decoding error: {}", err))
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_client_error_creation() {
        // Test creating each error variant
        let client_err = ClientError::ClientError("general client error".to_string());
        let query_err = ClientError::QueryError("failed query".to_string());
        let parse_err = ClientError::ParseError("parsing failed".to_string());
        let tx_err = ClientError::TransactionError("tx failed".to_string());
        let not_impl_err = ClientError::NotImplemented("feature not available".to_string());
        let timeout_err = ClientError::TimeoutError("request timed out".to_string());
        let ser_err = ClientError::SerializationError("serialization failed".to_string());
        let config_err = ClientError::ConfigError("invalid config".to_string());
        let not_found_err = ClientError::NotFoundError("resource missing".to_string());
        let state_err = ClientError::StateMismatch("unexpected state".to_string());
        let action_err = ClientError::ActionFailed("action couldn't complete".to_string());
        let contract_err = ClientError::ContractError("contract execution failed".to_string());
        let abi_err = ClientError::AbiError("ABI encoding failed".to_string());
        let msg_type_err = ClientError::InvalidMsgType("wrong message type".to_string());
        
        // Verify errors are created correctly
        assert!(matches!(client_err, ClientError::ClientError(_)));
        assert!(matches!(query_err, ClientError::QueryError(_)));
        assert!(matches!(parse_err, ClientError::ParseError(_)));
        assert!(matches!(tx_err, ClientError::TransactionError(_)));
        assert!(matches!(not_impl_err, ClientError::NotImplemented(_)));
        assert!(matches!(timeout_err, ClientError::TimeoutError(_)));
        assert!(matches!(ser_err, ClientError::SerializationError(_)));
        assert!(matches!(config_err, ClientError::ConfigError(_)));
        assert!(matches!(not_found_err, ClientError::NotFoundError(_)));
        assert!(matches!(state_err, ClientError::StateMismatch(_)));
        assert!(matches!(action_err, ClientError::ActionFailed(_)));
        assert!(matches!(contract_err, ClientError::ContractError(_)));
        assert!(matches!(abi_err, ClientError::AbiError(_)));
        assert!(matches!(msg_type_err, ClientError::InvalidMsgType(_)));
    }

    #[test]
    fn test_client_error_messages() {
        // Test error messages for each variant
        let client_err = ClientError::ClientError("general client error".to_string());
        assert_eq!(
            client_err.to_string(),
            "client error: general client error"
        );
        
        let query_err = ClientError::QueryError("failed query".to_string());
        assert_eq!(
            query_err.to_string(),
            "query error: failed query"
        );
        
        let parse_err = ClientError::ParseError("parsing failed".to_string());
        assert_eq!(
            parse_err.to_string(),
            "parse error: parsing failed"
        );
        
        let tx_err = ClientError::TransactionError("tx failed".to_string());
        assert_eq!(
            tx_err.to_string(),
            "transaction error: tx failed"
        );
        
        let not_impl_err = ClientError::NotImplemented("feature not available".to_string());
        assert_eq!(
            not_impl_err.to_string(),
            "not implemented: feature not available"
        );
        
        let timeout_err = ClientError::TimeoutError("request timed out".to_string());
        assert_eq!(
            timeout_err.to_string(),
            "timeout error: request timed out"
        );
    }

    #[test]
    fn test_error_trait_implementation() {
        // Verify ClientError implements the Error trait
        let err = ClientError::ClientError("test error".to_string());
        let dyn_err: &dyn Error = &err;
        
        // Test that error can be cast to trait object
        assert!(dyn_err.source().is_none());
        assert_eq!(dyn_err.to_string(), "client error: test error");
    }
}
