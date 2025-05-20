//-----------------------------------------------------------------------------
// Osmosis TokenFactory v1beta1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// MsgCreateDenom defines the message for creating a new denom
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateDenom {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub subdenom: ::prost::alloc::string::String,
}

/// MsgCreateDenomResponse defines the response for creating a new denom
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateDenomResponse {
    #[prost(string, tag = "1")]
    pub new_token_denom: ::prost::alloc::string::String,
}

/// MsgMint defines the message for minting tokenfactory tokens
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMint {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<Coin>,
    #[prost(string, tag = "3")]
    pub mint_to_address: ::prost::alloc::string::String,
}

/// MsgMintResponse defines the response for minting tokenfactory tokens
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgMintResponse {}

/// MsgBurn defines the message for burning tokenfactory tokens
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<Coin>,
    #[prost(string, tag = "3")]
    pub burn_from_address: ::prost::alloc::string::String,
}

/// MsgBurnResponse defines the response for burning tokenfactory tokens
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBurnResponse {}

/// MsgChangeAdmin defines the message for changing the admin of a tokenfactory denom
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeAdmin {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub new_admin: ::prost::alloc::string::String,
}

/// MsgChangeAdminResponse defines the response for changing the admin of a tokenfactory denom
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgChangeAdminResponse {}

/// Coin defines a token with a denomination and an amount
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
