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

    fn try_from(value: Header) -> anyhow::Result<Self, Self::Error> {
        let proto_time = value
            .time
            .ok_or_else(|| anyhow::anyhow!("No time in block header"))?
            .into();

        Ok(proto_time)
    }
}
