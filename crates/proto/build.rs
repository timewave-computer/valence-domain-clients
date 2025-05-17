//-----------------------------------------------------------------------------
// Protocol Buffer Build Script
//-----------------------------------------------------------------------------
//
// This script tells cargo to rebuild when proto files change
// and handles protocol buffer code generation.

fn main() {
    // Tell cargo to rebuild if any proto files change
    println!("cargo:rerun-if-changed=../../proto");

    // Using manual stubs for protos
    println!("cargo:warning=Using manually defined protocol buffer stubs");

    // Add protobuf build logic here if needed
    // tonic_build::compile_protos("../../proto/*.proto").unwrap();

    // Everything is currently defined manually
}
