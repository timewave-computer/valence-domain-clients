use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxResponse;

#[derive(Debug)]
pub struct TransactionResponse {
    pub hash: String,
    pub success: bool,
    pub block_height: u64,
    pub gas_used: u64,
}

impl TryFrom<TxResponse> for TransactionResponse {
    type Error = anyhow::Error;

    fn try_from(value: TxResponse) -> anyhow::Result<Self> {
        Ok(Self {
            hash: value.txhash,
            success: value.code == 0, // 0 is success
            block_height: u64::try_from(value.height)?,
            gas_used: u64::try_from(value.gas_used)?,
        })
    }
}

impl TryFrom<Option<TxResponse>> for TransactionResponse {
    type Error = anyhow::Error;

    fn try_from(value: Option<TxResponse>) -> anyhow::Result<Self> {
        match value {
            Some(tx) => Self::try_from(tx),
            None => Err(anyhow::anyhow!("failed to find tx_response".to_string(),)),
        }
    }
}
