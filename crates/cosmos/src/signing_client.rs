use bip32::{Language, Mnemonic, Seed};
use cosmos_sdk_proto::cosmos::{
    auth::v1beta1::{
        query_client::QueryClient as AuthQueryClient, BaseAccount,
        QueryAccountRequest,
    },
    tx::v1beta1::{
        BroadcastMode as ProtoBroadcastMode,
        BroadcastTxRequest as ProtoBroadcastTxRequest,
    },
};
use cosmrs::{
    crypto::{secp256k1::SigningKey, PublicKey},
    tendermint::chain::Id as CosmrsChainId,
    tx::{BodyBuilder, Fee as CosmrsFee, SignDoc, SignerInfo},
    AccountId, Any as CosmrsAny, Coin as CosmrsCoin, Denom as CosmrsDenom,
};
use prost::Message;
use tonic::transport::Channel;

use crate::errors::CosmosError;
use crate::types::{CosmosBroadcastMode, CosmosBroadcastTxRequest, CosmosFee};
use valence_core::error::ClientError;

const DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";

/// Internal struct for signing-related information.
pub(crate) struct SigningClient {
    pub signing_key: SigningKey,
    pub address: AccountId,
    pub account_number: u64,
    pub sequence: u64,
    pub chain_id: CosmrsChainId,
    pub public_key: PublicKey,
}

// Helper to convert our CosmosFee to cosmrs::Fee
fn to_cosmrs_fee(
    fee: &CosmosFee,
    default_denom_if_empty: &CosmrsDenom,
) -> Result<CosmrsFee, ClientError> {
    let amount: Vec<CosmrsCoin> = fee
        .amount
        .iter()
        .map(|coin| {
            let denom = coin
                .denom
                .parse::<CosmrsDenom>()
                .map_err(CosmosError::CosmrsError)?;

            Ok(CosmrsCoin {
                denom,
                amount: coin.amount,
            })
        })
        .collect::<Result<Vec<_>, ClientError>>()?;

    let fee_to_use = if amount.is_empty() {
        CosmrsFee::from_amount_and_gas(
            CosmrsCoin {
                denom: default_denom_if_empty.clone(),
                amount: 0,
            },
            fee.gas_limit,
        )
    } else {
        CosmrsFee {
            amount,
            gas_limit: fee.gas_limit,
            payer: None,
            granter: None,
        }
    };
    Ok(fee_to_use)
}

impl SigningClient {
    /// Signs a transaction and returns the encoded tx bytes.
    pub(crate) async fn sign_tx(
        &mut self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
        chain_default_denom: &CosmrsDenom,
    ) -> Result<Vec<u8>, ClientError> {
        let tx_body = BodyBuilder::new()
            .msg(msg)
            .memo(memo.unwrap_or_default().to_string())
            .finish();

        let cosmrs_fee = to_cosmrs_fee(&fee, chain_default_denom)?;

        let auth_info =
            SignerInfo::single_direct(Some(self.public_key), self.sequence)
                .auth_info(cosmrs_fee);

        let sign_doc =
            SignDoc::new(&tx_body, &auth_info, &self.chain_id, self.account_number)
                .map_err(CosmosError::CosmrsError)?;

        let tx_raw = sign_doc
            .sign(&self.signing_key)
            .map_err(CosmosError::CosmrsError)?;

        tx_raw
            .to_bytes()
            .map_err(|e| ClientError::from(CosmosError::CosmrsError(e)))
    }
    pub(crate) async fn from_mnemonic(
        channel: Channel,
        mnemonic_str: &str,
        prefix: &str,
        chain_id_str: &str,
    ) -> Result<Self, ClientError> {
        let mnemonic = Mnemonic::new(mnemonic_str, Language::English)
            .map_err(CosmosError::Bip32Error)?;
        let seed: Seed = mnemonic.to_seed("");
        let signing_key = Self::derive_signing_key(seed.as_ref(), DERIVATION_PATH)?;
        let public_key = signing_key.public_key();
        let sender_account_id = public_key
            .account_id(prefix)
            .map_err(CosmosError::CosmrsError)?;
        let parsed_chain_id: CosmrsChainId = chain_id_str.parse().map_err(|e| {
            ClientError::ParseError(format!(
                "Failed to parse chain_id '{chain_id_str}': {e}"
            ))
        })?;

        let mut auth_client = AuthQueryClient::new(channel);
        let account_info_resp = auth_client
            .account(QueryAccountRequest {
                address: sender_account_id.to_string(),
            })
            .await
            .map_err(CosmosError::StatusError)?
            .into_inner();

        let base_account_any = account_info_resp.account.ok_or_else(|| {
            ClientError::QueryError(
                "failed to get account info (account field is None)".to_string(),
            )
        })?;

        let base_account = BaseAccount::decode(base_account_any.value.as_slice())
            .map_err(|e| {
                ClientError::ParseError(format!("Failed to decode BaseAccount: {e}"))
            })?;

        Ok(SigningClient {
            signing_key,
            address: sender_account_id,
            account_number: base_account.account_number,
            sequence: base_account.sequence,
            chain_id: parsed_chain_id,
            public_key,
        })
    }

    fn derive_signing_key(
        seed_bytes: &[u8],
        path: &str,
    ) -> Result<SigningKey, ClientError> {
        let derivation_path =
            path.parse::<bip32::DerivationPath>().map_err(|e| {
                ClientError::ClientError(format!(
                    "Failed to parse derivation path '{path}': {e}"
                ))
            })?;
        let xprv = bip32::XPrv::new(seed_bytes).map_err(CosmosError::Bip32Error)?;

        // Use the correct method to derive child key from path
        // The bip32 crate's API changed and the method is now 'derive_path_vec'
        let child_xprv =
            derivation_path.into_iter().try_fold(xprv, |key, index| {
                key.derive_child(index).map_err(CosmosError::Bip32Error)
            })?;
        let secret_key = child_xprv.private_key(); // This is k256::SecretKey
        SigningKey::from_slice(secret_key.to_bytes().as_slice())
            .map_err(|e| ClientError::from(CosmosError::CosmrsError(e)))
    }

    #[allow(dead_code)]
    pub(crate) async fn create_tx(
        &self,
        msg: CosmrsAny,
        fee: CosmosFee,
        memo: Option<&str>,
        chain_default_denom: &CosmrsDenom,
    ) -> Result<CosmosBroadcastTxRequest, ClientError> {
        let tx_body = BodyBuilder::new()
            .msg(msg)
            .memo(memo.unwrap_or_default().to_string())
            .finish();

        let cosmrs_fee = to_cosmrs_fee(&fee, chain_default_denom)?;

        let auth_info =
            SignerInfo::single_direct(Some(self.public_key), self.sequence)
                .auth_info(cosmrs_fee);

        let sign_doc =
            SignDoc::new(&tx_body, &auth_info, &self.chain_id, self.account_number)
                .map_err(CosmosError::CosmrsError)?;

        let tx_raw = sign_doc
            .sign(&self.signing_key)
            .map_err(CosmosError::CosmrsError)?;

        let tx_bytes = tx_raw.to_bytes().map_err(CosmosError::CosmrsError)?;

        let proto_req = ProtoBroadcastTxRequest {
            tx_bytes,
            mode: ProtoBroadcastMode::Sync.into(),
        };

        // Create a CosmosBroadcastTxRequest with the correct mode and tx_bytes
        let broadcast_mode = match ProtoBroadcastMode::try_from(proto_req.mode) {
            Ok(ProtoBroadcastMode::Sync) => CosmosBroadcastMode::Sync,
            Ok(ProtoBroadcastMode::Async) => CosmosBroadcastMode::Async,
            Ok(ProtoBroadcastMode::Block) => CosmosBroadcastMode::Block,
            _ => CosmosBroadcastMode::Sync,
        };

        // Return request with tx_bytes and mode
        Ok(CosmosBroadcastTxRequest {
            tx_bytes: proto_req.tx_bytes,
            mode: broadcast_mode,
            fee_setting: crate::types::FeeSetting::default(),
            memo: None,
        })
    }
}
