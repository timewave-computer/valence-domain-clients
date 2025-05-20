//-----------------------------------------------------------------------------
// Osmosis GAMM v1beta1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// MsgSwapExactAmountIn defines a message for swapping an exact amount of tokens in
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(message, optional, tag = "3")]
    pub token_in: ::core::option::Option<Coin>,
    #[prost(string, tag = "4")]
    pub token_out_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub token_out_min_amount: ::prost::alloc::string::String,
}

/// MsgSwapExactAmountInResponse defines the response for swapping an exact amount in
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}

/// MsgJoinPool defines the message for joining a pool with an exact amount of tokens
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgJoinPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(string, tag = "3")]
    pub share_out_min_amount: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub token_in_maxs: ::prost::alloc::vec::Vec<Coin>,
}

/// MsgJoinPoolResponse defines the response for joining a pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgJoinPoolResponse {
    #[prost(string, tag = "1")]
    pub share_out_amount: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub token_in: ::prost::alloc::vec::Vec<Coin>,
}

/// Coin defines a token with a denomination and an amount
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}

/// Pool represents a liquidity pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pool {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(string, tag = "2")]
    pub pool_params: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub future_pool_governor: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub total_shares: ::prost::alloc::vec::Vec<Coin>,
    #[prost(message, repeated, tag = "5")]
    pub pool_assets: ::prost::alloc::vec::Vec<PoolAsset>,
    #[prost(string, tag = "6")]
    pub total_weight: ::prost::alloc::string::String,
}

/// PoolAsset represents a token in a pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PoolAsset {
    #[prost(string, tag = "1")]
    pub token_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}
