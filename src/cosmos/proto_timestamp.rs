use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::Header;

pub struct ProtoTimestamp(cosmos_sdk_proto::Timestamp);

pub const NANOS_IN_SECOND: u64 = 1_000_000_000;

impl ProtoTimestamp {
    pub fn extend_by_seconds(&mut self, seconds: u64) -> anyhow::Result<()> {
        let seconds = i64::try_from(seconds)?;

        match self.0.seconds.checked_add(seconds) {
            Some(val) => {
                self.0.seconds = val;
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "proto timestamp extend by seconds overflow"
            )),
        }
    }

    pub fn to_nanos(&self) -> anyhow::Result<u64> {
        let current_seconds = u64::try_from(self.0.seconds)?;
        let current_nanos = u64::try_from(self.0.nanos)?;

        current_seconds
            .checked_mul(NANOS_IN_SECOND)
            .ok_or_else(|| anyhow::anyhow!("failed to convert seconds to nanos"))?
            .checked_add(current_nanos)
            .ok_or_else(|| anyhow::anyhow!("failed to add current nanos"))
    }
}

impl From<cosmos_sdk_proto::Timestamp> for ProtoTimestamp {
    fn from(ts: cosmos_sdk_proto::Timestamp) -> Self {
        ProtoTimestamp(ts)
    }
}

impl TryFrom<Header> for ProtoTimestamp {
    type Error = anyhow::Error;

    fn try_from(value: Header) -> anyhow::Result<Self> {
        let proto_time = value
            .time
            .ok_or_else(|| anyhow::anyhow!("No time in block header"))?
            .into();

        Ok(proto_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::{cosmos::base::tendermint::v1beta1::Header, Timestamp};

    #[test]
    fn test_from_timestamp_happy() {
        let timestamp = Timestamp {
            seconds: 1234567890,
            nanos: 123456789,
        };

        let proto_timestamp = ProtoTimestamp::from(timestamp);

        assert_eq!(proto_timestamp.0.seconds, timestamp.seconds);
        assert_eq!(proto_timestamp.0.nanos, timestamp.nanos);
    }

    #[test]
    fn test_extend_by_seconds_happy() {
        let timestamp = Timestamp {
            seconds: 1000,
            nanos: 500,
        };

        let mut proto_timestamp = ProtoTimestamp::from(timestamp);
        proto_timestamp.extend_by_seconds(500).unwrap();

        assert_eq!(proto_timestamp.0.seconds, 1500);
        assert_eq!(proto_timestamp.0.nanos, 500);
    }

    #[test]
    #[should_panic(expected = "proto timestamp extend by seconds overflow")]
    fn test_extend_by_seconds_overflow() {
        let timestamp = Timestamp {
            seconds: i64::MAX - 10,
            nanos: 500,
        };

        let mut proto_timestamp = ProtoTimestamp::from(timestamp);

        proto_timestamp.extend_by_seconds(20).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_extend_by_seconds_conversion_error() {
        let timestamp = Timestamp {
            seconds: 1000,
            nanos: 500,
        };

        let mut proto_timestamp = ProtoTimestamp::from(timestamp);
        proto_timestamp.extend_by_seconds(u64::MAX).unwrap();
    }

    #[test]
    fn test_to_nanos_happy() {
        let timestamp = Timestamp {
            seconds: 5,
            nanos: 500_000_000,
        };

        let proto_timestamp = ProtoTimestamp::from(timestamp);
        let result = proto_timestamp.to_nanos().unwrap();

        assert_eq!(result, 5_500_000_000);
    }

    #[test]
    #[should_panic]
    fn test_to_nanos_seconds_overflow() {
        let timestamp = Timestamp {
            seconds: i64::MAX,
            nanos: 500_000_000,
        };

        let proto_timestamp = ProtoTimestamp::from(timestamp);
        proto_timestamp.to_nanos().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_to_nanos_addition_overflow() {
        let timestamp = Timestamp {
            seconds: u64::MAX as i64,
            nanos: 500_000_000,
        };

        let proto_timestamp = ProtoTimestamp::from(timestamp);
        proto_timestamp.to_nanos().unwrap();
    }

    #[test]
    fn test_try_from_header_happy() {
        let timestamp = Timestamp {
            seconds: 1000,
            nanos: 500,
        };

        let header = Header {
            time: Some(timestamp.clone()),
            ..Default::default()
        };

        let proto_timestamp = ProtoTimestamp::try_from(header).unwrap();

        assert_eq!(proto_timestamp.0.seconds, timestamp.seconds);
        assert_eq!(proto_timestamp.0.nanos, timestamp.nanos);
    }

    #[test]
    #[should_panic(expected = "No time in block header")]
    fn test_try_from_header_missing_time() {
        let header = Header {
            time: None,
            ..Default::default()
        };

        ProtoTimestamp::try_from(header).unwrap();
    }
}
