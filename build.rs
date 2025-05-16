//-----------------------------------------------------------------------------
// Main Build Script
//-----------------------------------------------------------------------------
//
// This script tells cargo to rebuild when proto files change,
// and handles other project-wide build requirements.

fn main() {
    // Tell cargo to rebuild if any proto files change
    println!("cargo:rerun-if-changed=crates/proto");
    
    // Using manual stubs for protos
    println!("cargo:warning=Using manually defined protocol buffer stubs");
} 