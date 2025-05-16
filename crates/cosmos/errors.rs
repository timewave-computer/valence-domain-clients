use std::num::TryFromIntError;

use alloy::transports::http::reqwest;
use cosmrs::ErrorReport;
use tonic::Status;

use crate::core::error::ClientError;

impl From<cosmrs::rpc::Error> for ClientError {
    fn from(value: cosmrs::rpc::Error) -> Self {
        ClientError::ClientError(value.to_string())
    }
}

impl From<Status> for ClientError {
    fn from(value: Status) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<ErrorReport> for ClientError {
    fn from(value: ErrorReport) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<TryFromIntError> for ClientError {
    fn from(value: TryFromIntError) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<serde_json::error::Error> for ClientError {
    fn from(value: serde_json::error::Error) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<bip32::Error> for ClientError {
    fn from(value: bip32::Error) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<cosmrs::tendermint::Error> for ClientError {
    fn from(value: cosmrs::tendermint::Error) -> Self {
        ClientError::ParseError(value.to_string())
    }
}

impl From<tonic::transport::Error> for ClientError {
    fn from(value: tonic::transport::Error) -> Self {
        ClientError::ClientError(value.to_string())
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::ClientError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use tonic::Status;
    use tonic::Code;

    #[test]
    fn test_status_conversion() {
        let status = Status::new(Code::InvalidArgument, "invalid argument");
        let error: ClientError = status.into();
        
        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("invalid argument"));
            },
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_try_from_int_error_conversion() {
        // Create a TryFromIntError by attempting an invalid conversion
        let result = u8::try_from(256i32);
        let err = result.unwrap_err();
        
        let error: ClientError = err.into();
        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("out of range"));
            },
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Create a serde_json error
        let json = "invalid json";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        let err = result.unwrap_err();
        
        let error: ClientError = err.into();
        match error {
            ClientError::ParseError(msg) => {
                assert!(msg.contains("expected"));
            },
            _ => panic!("Expected ParseError variant"),
        }
    }

    // We skip testing tonic::transport::Error since it's challenging to create
    // in a test environment without actual network operations

    // Note: We can't easily test the other error conversions without creating
    // complex structs, but the pattern is the same as the tests above
}
