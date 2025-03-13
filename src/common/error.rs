use std::{error::Error, fmt::Display};

/// error type to be returned by all client types.
#[derive(Debug)]
pub enum StrategistError {
    ClientError(String),
    QueryError(String),
    ParseError(String),
    TransactionError(String),
}

impl Error for StrategistError {}

impl Display for StrategistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategistError::ClientError(msg) => write!(f, "Client error: {msg}"),
            StrategistError::QueryError(msg) => write!(f, "Query error: {msg}"),
            StrategistError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            StrategistError::TransactionError(msg) => write!(f, "Transaction error: {msg}"),
        }
    }
}
