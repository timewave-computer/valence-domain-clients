use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::Header;

use crate::core::error::ClientError;

/// Protocol buffer timestamp wrapper
/// 
/// This wraps the cosmos_sdk_proto::Timestamp to provide a more
/// convenient interface.
#[allow(dead_code)]
pub(crate) struct ProtoTimestamp(cosmos_sdk_proto::Timestamp);

#[allow(dead_code)]
pub(crate) const NANOS_IN_SECOND: u64 = 1_000_000_000;

#[allow(dead_code)]
impl ProtoTimestamp {
    pub(crate) fn extend_by_seconds(
        &self,
        seconds: i64,
    ) -> Self {
        let mut new_timestamp = self.0;
        new_timestamp.seconds += seconds;
        ProtoTimestamp(new_timestamp)
    }

    pub(crate) fn now() -> Self {
        let now = chrono::Utc::now();
        let seconds = now.timestamp();
        let nanos = now.timestamp_nanos_opt()
            .unwrap_or(0) % NANOS_IN_SECOND as i64;
        
        ProtoTimestamp(cosmos_sdk_proto::Timestamp {
            seconds,
            nanos: nanos as i32,
        })
    }

    pub(crate) fn to_nanos(&self) -> Result<u64, ClientError> {
        let seconds = self.0.seconds as u64;
        let nanos = self.0.nanos as u64;
        
        // Prevent overflow
        if seconds > u64::MAX / NANOS_IN_SECOND {
            return Err(ClientError::ParseError("Timestamp seconds overflow".to_string()));
        }
        
        let total_nanos = seconds * NANOS_IN_SECOND + nanos;
        Ok(total_nanos)
    }
}

// This From impl is effectively pub(crate) because ProtoTimestamp is pub(crate)
impl From<cosmos_sdk_proto::Timestamp> for ProtoTimestamp {
    fn from(ts: cosmos_sdk_proto::Timestamp) -> Self {
        ProtoTimestamp(ts)
    }
}

// This TryFrom impl is effectively pub(crate)
impl TryFrom<Header> for ProtoTimestamp {
    type Error = ClientError;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let proto_time = value
            .time
            .ok_or_else(|| {
                ClientError::QueryError("No time in block header".to_string())
            })?
            .into();

        Ok(proto_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::Header;
    use cosmos_sdk_proto::Timestamp as ProtoTimestampProto;
    
    #[test]
    fn test_proto_timestamp_from() {
        // Create a protocol buffer timestamp
        let proto = ProtoTimestampProto {
            seconds: 1625097600, // 2021-07-01
            nanos: 500_000_000,  // 0.5 seconds
        };
        
        // Convert to our wrapper
        let timestamp: ProtoTimestamp = proto.into();
        
        // Check conversion to nanoseconds
        let nanos = timestamp.to_nanos().unwrap();
        let expected = 1625097600 * NANOS_IN_SECOND + 500_000_000;
        assert_eq!(nanos, expected);
    }
    
    #[test]
    fn test_proto_timestamp_extend() {
        // Create a timestamp
        let proto = ProtoTimestampProto {
            seconds: 1625097600, // 2021-07-01
            nanos: 500_000_000,  // 0.5 seconds
        };
        let timestamp: ProtoTimestamp = proto.into();
        
        // Extend by 60 seconds
        let extended = timestamp.extend_by_seconds(60);
        
        // Check new timestamp
        let nanos = extended.to_nanos().unwrap();
        let expected = (1625097600 + 60) * NANOS_IN_SECOND + 500_000_000;
        assert_eq!(nanos, expected);
    }
    
    #[test]
    fn test_proto_timestamp_now() {
        // Get current timestamp
        let now = ProtoTimestamp::now();
        
        // Get current time using chrono
        let chrono_now = chrono::Utc::now();
        let chrono_seconds = chrono_now.timestamp();
        
        // Get seconds from our timestamp (need to convert back to nanoseconds then to seconds)
        let now_nanos = now.to_nanos().unwrap();
        let now_seconds = now_nanos / NANOS_IN_SECOND;
        
        // They should be very close (within a second)
        let diff = (now_seconds as i64 - chrono_seconds).abs();
        assert!(diff <= 1, "Timestamps should be within 1 second, but differ by {} seconds", diff);
    }
    
    #[test]
    fn test_proto_timestamp_to_nanos_overflow() {
        // Create a timestamp that would overflow when converted to nanos
        let proto = ProtoTimestampProto {
            seconds: i64::MAX, // Too large to fit in u64 nanoseconds
            nanos: 0,
        };
        let timestamp: ProtoTimestamp = proto.into();
        
        // Should return an error
        let result = timestamp.to_nanos();
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ClientError::ParseError(msg) => {
                    assert!(msg.contains("overflow"));
                },
                _ => panic!("Expected ParseError"),
            }
        }
    }
    
    #[test]
    fn test_proto_timestamp_try_from_header() {
        // Create a mock header with a timestamp
        let proto_time = ProtoTimestampProto {
            seconds: 1625097600,
            nanos: 0,
        };
        
        let mut header = Header::default();
        header.time = Some(proto_time.clone());
        
        // Convert header to timestamp
        let timestamp = ProtoTimestamp::try_from(header).unwrap();
        
        // Check conversion
        let nanos = timestamp.to_nanos().unwrap();
        let expected = 1625097600 * NANOS_IN_SECOND;
        assert_eq!(nanos, expected);
    }
    
    #[test]
    fn test_proto_timestamp_try_from_header_missing_time() {
        // Create a header with no timestamp
        let header = Header::default();
        
        // Conversion should fail
        let result = ProtoTimestamp::try_from(header);
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                ClientError::QueryError(msg) => {
                    assert!(msg.contains("No time"));
                },
                _ => panic!("Expected QueryError"),
            }
        }
    }
}
