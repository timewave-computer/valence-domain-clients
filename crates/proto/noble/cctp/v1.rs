//-----------------------------------------------------------------------------
// Noble CCTP v1 Protocol Buffers 
//-----------------------------------------------------------------------------

/// CCTP Message type for cross-chain transfers
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Message {
    #[prost(bytes = "vec", tag = "1")]
    pub message_body: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub source_domain: u32,
    #[prost(string, tag = "3")]
    pub destination_domain: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub receiver_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "5")]
    pub message_type: ::prost::alloc::string::String,
}

/// MsgDepositForBurn defines the Msg/DepositForBurn request type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositForBurn {
    #[prost(string, tag = "1")]
    pub from: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub amount: u64,
    #[prost(string, tag = "3")]
    pub destination_domain: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub mint_recipient: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub burn_token: ::prost::alloc::string::String,
}

/// MsgDepositForBurnResponse defines the Msg/DepositForBurn response type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositForBurnResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub message_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub nonce: u64,
}
