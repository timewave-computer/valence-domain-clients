//-----------------------------------------------------------------------------
// Transaction Types
//-----------------------------------------------------------------------------

use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse as ProtoTxResponse;
use hex;
use serde::{Deserialize, Serialize};

use super::error::ClientError;

//-----------------------------------------------------------------------------
// Event Definitions
//-----------------------------------------------------------------------------

/// Attribute for blockchain events
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventAttribute {
    pub key: String,   // from proto Vec<u8>
    pub value: String, // from proto Vec<u8>
    pub index: bool,   // Available in tendermint_proto::abci::EventAttribute
}

/// Standard event structure used across different blockchain ecosystems
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    pub event_type: String,
    pub attributes: Vec<(String, String)>,
}

/// Helper function to convert protocol buffer events to our local Event type
#[allow(dead_code)]
pub(crate) fn convert_proto_events<E>(proto_events: Vec<E>) -> Vec<Event>
where
    E: Into<cosmos_sdk_proto::tendermint::abci::Event> + Clone,
{
    proto_events.into_iter().map(|proto_event| -> Event {
        let proto_event: cosmos_sdk_proto::tendermint::abci::Event = proto_event.into();
        let attributes: Vec<(String, String)> = proto_event.attributes.into_iter().map(|attr: cosmos_sdk_proto::tendermint::abci::EventAttribute| -> (String, String) {
            (attr.key, attr.value)
        }).collect();
        Event {
            event_type: proto_event.r#type,
            attributes,
        }
    }).collect()
}

//-----------------------------------------------------------------------------
// Transaction Response
//-----------------------------------------------------------------------------

/// A unified transaction response structure.
///
/// This provides a consistent interface for transaction results across different
/// blockchain ecosystems, abstracting protocol-specific details.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub height: u64,
    pub gas_wanted: Option<i64>,
    pub gas_used: Option<i64>,
    pub code: Option<u32>,
    pub events: Vec<Event>,
    pub data: Option<String>,
    pub raw_log: Option<String>,
    pub timestamp: i64,             // Unix timestamp in seconds
    pub block_hash: Option<String>, // Hex-encoded block hash, if available
    pub original_request_payload: Option<Vec<u8>>, // The original payload that initiated this transaction
}

impl TransactionResponse {
    /// Creates a new transaction response with the provided parameters
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_hash: String,
        height: u64,
        gas_wanted: i64,
        gas_used: i64,
        code: u32,
        events: Vec<Event>,
        data: Option<String>,
        timestamp: i64,
        block_hash: Option<String>,
        original_request_payload: Option<Vec<u8>>,
    ) -> Self {
        Self {
            tx_hash,
            height,
            gas_wanted: Some(gas_wanted),
            gas_used: Some(gas_used),
            code: Some(code),
            events,
            data,
            raw_log: None,
            timestamp,
            block_hash,
            original_request_payload,
        }
    }
}

/// Converts a `ProtoTxResponse` to our domain `TransactionResponse`.
#[allow(dead_code)]
pub(crate) fn convert_from_proto_tx_response(
    value: ProtoTxResponse,
) -> Result<TransactionResponse, ClientError> {
    let data_hex_string: Option<String> = if value.data.is_empty() {
        None
    } else {
        Some(hex::encode(value.data))
    };

    Ok(TransactionResponse {
        tx_hash: value.txhash,
        height: value.height as u64,
        gas_wanted: Some(value.gas_wanted),
        gas_used: Some(value.gas_used),
        code: Some(value.code),
        events: {
            // Convert each event type to our format
            let converted_events = value
                .events
                .iter()
                .map(|evt| {
                    // Create our own Event structure with compatible types
                    Event {
                        event_type: evt.r#type.clone(),
                        attributes: evt
                            .attributes
                            .iter()
                            .map(|attr| {
                                (
                                    String::from_utf8_lossy(&attr.key).to_string(),
                                    String::from_utf8_lossy(&attr.value).to_string(),
                                )
                            })
                            .collect(),
                    }
                })
                .collect::<Vec<_>>();

            converted_events
        },
        data: data_hex_string,
        raw_log: None,
        timestamp: value.timestamp.parse::<i64>().map_err(|e| {
            ClientError::ParseError(format!("Failed to parse timestamp: {e}"))
        })?,
        block_hash: None,
        original_request_payload: None,
    })
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::tendermint::abci::{
        Event as ProtoEvent, EventAttribute as ProtoEventAttribute,
    };
    use serde_json;

    #[test]
    fn test_event_serialization() {
        // Create a test event
        let event = Event {
            event_type: "transfer".to_string(),
            attributes: vec![
                ("sender".to_string(), "cosmos1abc...".to_string()),
                ("recipient".to_string(), "cosmos1xyz...".to_string()),
                ("amount".to_string(), "100000".to_string()),
            ],
        };

        // Test serialization/deserialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
        assert_eq!(event.event_type, "transfer");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].0, "sender");
        assert_eq!(event.attributes[1].0, "recipient");
        assert_eq!(event.attributes[2].0, "amount");
    }

    #[test]
    fn test_convert_proto_events() {
        // Create test proto events
        let mut proto_event1 = ProtoEvent::default();
        proto_event1.r#type = "transfer".to_string();

        let mut attr1 = ProtoEventAttribute::default();
        attr1.key = "sender".to_string();
        attr1.value = "cosmos1abc...".to_string();

        let mut attr2 = ProtoEventAttribute::default();
        attr2.key = "recipient".to_string();
        attr2.value = "cosmos1xyz...".to_string();

        proto_event1.attributes = vec![attr1, attr2];

        // Convert proto events to our format
        let events = convert_proto_events(vec![proto_event1.clone()]);

        // Verify conversion
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "transfer");
        assert_eq!(events[0].attributes.len(), 2);
        assert_eq!(events[0].attributes[0].0, "sender");
        assert_eq!(events[0].attributes[0].1, "cosmos1abc...");
        assert_eq!(events[0].attributes[1].0, "recipient");
        assert_eq!(events[0].attributes[1].1, "cosmos1xyz...");
    }

    #[test]
    fn test_transaction_response_new() {
        // Create a test transaction response
        let tx_hash =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                .to_string();
        let height = 12345u64;
        let gas_wanted = 100000i64;
        let gas_used = 80000i64;
        let code = 0u32;
        let events = vec![Event {
            event_type: "transfer".to_string(),
            attributes: vec![
                ("sender".to_string(), "alice".to_string()),
                ("recipient".to_string(), "bob".to_string()),
                ("amount".to_string(), "100000".to_string()),
            ],
        }];
        let data = Some("payload".to_string());
        let timestamp = 1625097600i64; // 2021-07-01T00:00:00Z
        let block_hash = Some(
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
        );
        let original_request_payload = Some(vec![1, 2, 3, 4]);

        // Create transaction response
        let tx_response = TransactionResponse::new(
            tx_hash.clone(),
            height,
            gas_wanted,
            gas_used,
            code,
            events.clone(),
            data.clone(),
            timestamp,
            block_hash.clone(),
            original_request_payload.clone(),
        );

        // Verify fields
        assert_eq!(tx_response.tx_hash, tx_hash);
        assert_eq!(tx_response.height, height);
        assert_eq!(tx_response.gas_wanted, Some(gas_wanted));
        assert_eq!(tx_response.gas_used, Some(gas_used));
        assert_eq!(tx_response.code, Some(code));
        assert_eq!(tx_response.events, events);
        assert_eq!(tx_response.data, data);
        assert_eq!(tx_response.raw_log, None);
        assert_eq!(tx_response.timestamp, timestamp);
        assert_eq!(tx_response.block_hash, block_hash);
        assert_eq!(
            tx_response.original_request_payload,
            original_request_payload
        );
    }
}
