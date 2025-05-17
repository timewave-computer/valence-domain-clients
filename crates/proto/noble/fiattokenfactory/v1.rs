//-----------------------------------------------------------------------------
// Noble FiatTokenFactory v1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// MsgConfigureMinter defines the Msg/ConfigureMinter request type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConfigureMinter {
    #[prost(string, tag = "1")]
    pub from: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub minter: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub minter_allowance: ::prost::alloc::string::String,
}

/// MsgConfigureMinterResponse defines the Msg/ConfigureMinter response type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgConfigureMinterResponse {}

/// MsgMint defines the Msg/Mint request type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMint {
    #[prost(string, tag = "1")]
    pub from: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub to: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount: ::prost::alloc::string::String,
}

/// MsgMintResponse defines the Msg/Mint response type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMintResponse {}

/// MsgBurn defines the Msg/Burn request type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurn {
    #[prost(string, tag = "1")]
    pub from: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}

/// MsgBurnResponse defines the Msg/Burn response type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurnResponse {}

/// Coin defines a token with a denomination and an amount
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
