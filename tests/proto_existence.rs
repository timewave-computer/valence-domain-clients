//-----------------------------------------------------------------------------
// Protocol Module Existence Test
//-----------------------------------------------------------------------------

// This test is only to verify imports work, so we suppress unused imports warnings
#[allow(unused_imports)]
mod proto_imports {
    // Import types from the proto module to ensure it exists and is accessible
    use valence_domain_clients::proto::ProtoError;
    use valence_domain_clients::proto::ProtoEncodable;
    use valence_domain_clients::proto::ProtoDecodable;

    // Try to import the noble modules to verify they exist
    use valence_domain_clients::proto::noble::tokenfactory;
    use valence_domain_clients::proto::noble::cctp;
    use valence_domain_clients::proto::noble::fiattokenfactory;

    // Try to import the osmosis modules to verify they exist
    use valence_domain_clients::proto::osmosis::concentratedliquidity;
    use valence_domain_clients::proto::osmosis::gamm;
    use valence_domain_clients::proto::osmosis::poolmanager;
    use valence_domain_clients::proto::osmosis::superfluid;
    use valence_domain_clients::proto::osmosis::tokenfactory as osmosis_tokenfactory;
}

#[test]
fn test_proto_module_exists() {
    // If we got this far, the proto module exists and can be imported
    // This test just verifies compilation, it doesn't need to do anything
    println!("Proto module exists and can be imported");
} 