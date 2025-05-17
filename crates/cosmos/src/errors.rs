use std::num::TryFromIntError;

use alloy::transports::http::reqwest;
use cosmrs::ErrorReport;
use tonic::Status;

use valence_core::error::ClientError;

/// Wrapper for cosmos-specific errors that can be converted to the common ClientError
#[derive(Debug)]
pub enum CosmosError {
    /// Tendermint RPC errors
    RpcError(cosmrs::rpc::Error),
    /// Tonic status errors
    StatusError(Status),
    /// Cosmrs general errors
    CosmrsError(ErrorReport),
    /// Integer conversion errors
    IntConversionError(TryFromIntError),
    /// JSON serialization errors
    JsonError(serde_json::error::Error),
    /// BIP32 crypto errors
    Bip32Error(bip32::Error),
    /// Tendermint errors
    TendermintError(cosmrs::tendermint::Error),
    /// Tonic transport errors
    TransportError(tonic::transport::Error),
    /// HTTP client errors
    HttpError(reqwest::Error),
}

impl From<CosmosError> for ClientError {
    fn from(err: CosmosError) -> Self {
        match err {
            CosmosError::RpcError(e) => ClientError::ClientError(e.to_string()),
            CosmosError::StatusError(e) => ClientError::ParseError(e.to_string()),
            CosmosError::CosmrsError(e) => ClientError::ParseError(e.to_string()),
            CosmosError::IntConversionError(e) => {
                ClientError::ParseError(e.to_string())
            }
            CosmosError::JsonError(e) => ClientError::ParseError(e.to_string()),
            CosmosError::Bip32Error(e) => ClientError::ParseError(e.to_string()),
            CosmosError::TendermintError(e) => {
                ClientError::ParseError(e.to_string())
            }
            CosmosError::TransportError(e) => {
                ClientError::ClientError(e.to_string())
            }
            CosmosError::HttpError(e) => ClientError::ClientError(e.to_string()),
        }
    }
}

// Implement From for each error type to CosmosError
impl From<cosmrs::rpc::Error> for CosmosError {
    fn from(err: cosmrs::rpc::Error) -> Self {
        CosmosError::RpcError(err)
    }
}

impl From<Status> for CosmosError {
    fn from(err: Status) -> Self {
        CosmosError::StatusError(err)
    }
}

impl From<ErrorReport> for CosmosError {
    fn from(err: ErrorReport) -> Self {
        CosmosError::CosmrsError(err)
    }
}

impl From<TryFromIntError> for CosmosError {
    fn from(err: TryFromIntError) -> Self {
        CosmosError::IntConversionError(err)
    }
}

impl From<serde_json::error::Error> for CosmosError {
    fn from(err: serde_json::error::Error) -> Self {
        CosmosError::JsonError(err)
    }
}

impl From<bip32::Error> for CosmosError {
    fn from(err: bip32::Error) -> Self {
        CosmosError::Bip32Error(err)
    }
}

impl From<cosmrs::tendermint::Error> for CosmosError {
    fn from(err: cosmrs::tendermint::Error) -> Self {
        CosmosError::TendermintError(err)
    }
}

impl From<tonic::transport::Error> for CosmosError {
    fn from(err: tonic::transport::Error) -> Self {
        CosmosError::TransportError(err)
    }
}

impl From<reqwest::Error> for CosmosError {
    fn from(err: reqwest::Error) -> Self {
        CosmosError::HttpError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use tonic::Code;
    use tonic::Status;

    #[test]
    fn test_status_conversion() {
        let status = Status::new(Code::InvalidArgument, "invalid argument");
        let cosmos_error: CosmosError = status.into();
        let error: ClientError = cosmos_error.into();

        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("invalid argument"));
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_try_from_int_error_conversion() {
        // Create a TryFromIntError by attempting an invalid conversion
        let result = u8::try_from(256i32);
        let err = result.unwrap_err();

        let cosmos_error: CosmosError = err.into();
        let error: ClientError = cosmos_error.into();

        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("out of range"));
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Create a serde_json error
        let json = "invalid json";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        let err = result.unwrap_err();

        let cosmos_error: CosmosError = err.into();
        let error: ClientError = cosmos_error.into();

        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("expected"));
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    // We skip testing tonic::transport::Error since it's challenging to create
    // in a test environment without actual network operations

    // Note: We can't easily test the other error conversions without creating
    // complex structs, but the pattern is the same as the tests above
}
