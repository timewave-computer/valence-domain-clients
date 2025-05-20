//-----------------------------------------------------------------------------
// Osmosis ConcentratedLiquidity v1beta1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// MsgCreatePosition defines the message for creating a concentrated liquidity position
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePosition {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub pool_id: u64,
    #[prost(string, tag = "3")]
    pub lower_tick: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub upper_tick: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "5")]
    pub tokens_provided: ::prost::alloc::vec::Vec<Coin>,
    #[prost(string, tag = "6")]
    pub token_min_amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub token_min_amount1: ::prost::alloc::string::String,
}

/// MsgCreatePositionResponse defines the response for creating a concentrated liquidity position
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreatePositionResponse {
    #[prost(uint64, tag = "1")]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub liquidity_created: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "5")]
    pub lower_tick: ::core::option::Option<Tick>,
    #[prost(message, optional, tag = "6")]
    pub upper_tick: ::core::option::Option<Tick>,
}

/// MsgWithdrawPosition defines the message for withdrawing a concentrated liquidity position
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawPosition {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub position_id: u64,
    #[prost(string, tag = "3")]
    pub liquidity_amount: ::prost::alloc::string::String,
}

/// MsgWithdrawPositionResponse defines the response for withdrawing a concentrated liquidity position
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawPositionResponse {
    #[prost(string, tag = "1")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount1: ::prost::alloc::string::String,
}

/// Position represents a concentrated liquidity position
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    #[prost(uint64, tag = "1")]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub pool_id: u64,
    #[prost(string, tag = "4")]
    pub lower_tick: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub upper_tick: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub liquidity: ::prost::alloc::string::String,
}

/// Tick represents a tick in a concentrated liquidity pool
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tick {
    #[prost(string, tag = "1")]
    pub tick_index: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub liquidity_net: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub liquidity_gross: ::prost::alloc::string::String,
}

/// Coin defines a token with a denomination and an amount
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
