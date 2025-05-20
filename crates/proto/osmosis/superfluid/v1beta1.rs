//-----------------------------------------------------------------------------
// Osmosis Superfluid v1beta1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// MsgLockAndSuperfluidDelegate defines the message for locking and superfluid delegating
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockAndSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub coins: ::core::option::Option<Coin>,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
}

/// MsgLockAndSuperfluidDelegateResponse defines the response for locking and superfluid delegating
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgLockAndSuperfluidDelegateResponse {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

/// MsgSuperfluidDelegate defines the message for superfluid delegating synthetic lockup
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
}

/// MsgSuperfluidDelegateResponse defines the response for superfluid delegating synthetic lockup
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidDelegateResponse {}

/// MsgSuperfluidUndelegate defines the message for superfluid undelegating
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub lock_id: u64,
}

/// MsgSuperfluidUndelegateResponse defines the response for superfluid undelegating
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSuperfluidUndelegateResponse {}

/// MsgUnPoolWhitelistedPool defines the message for unbonding whitelisted pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnPoolWhitelistedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
}

/// MsgUnPoolWhitelistedPoolResponse defines the response for unbonding whitelisted pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnPoolWhitelistedPoolResponse {}

/// Coin defines a token with a denomination and an amount
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
