# Nix script for fetching protocol buffer definitions from various blockchain projects
{ pkgs ? import <nixpkgs> {} }:

let
  name = "fetch-protocol-buffers";
  
  # Define version tags for various chains
  versions = {
    noble = {
      tag = "v1.0.0";
      repo = "https://github.com/noble-assets/noble";
      protoPath = "proto";
    };
    osmosis = {
      tag = "v20.0.0";
      repo = "https://github.com/osmosis-labs/osmosis";
      protoPath = "proto";
    };
    cosmos-sdk = {
      tag = "v0.47.5";
      repo = "https://github.com/cosmos/cosmos-sdk";
      protoPath = "proto";
    };
  };
  
  # Base derivation for the fetch-protos script
  script = pkgs.writeShellScriptBin name ''
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Set directories
    SCRIPT_DIR="$( cd "$( dirname "''${BASH_SOURCE[0]}" )" && pwd )"
    PROTO_DIR="$SCRIPT_DIR/../src/proto"
    THIRD_PARTY_DIR="$SCRIPT_DIR/../third_party"
    TMP_DIR=$(mktemp -d)
    
    # Create proto and third_party directories if they don't exist
    mkdir -p "$PROTO_DIR"
    mkdir -p "$THIRD_PARTY_DIR/gogoproto"
    mkdir -p "$THIRD_PARTY_DIR/cosmos_proto"
    mkdir -p "$THIRD_PARTY_DIR/amino"
    
    # Clean up temporary directory on exit
    trap 'rm -rf "$TMP_DIR"' EXIT
    
    # Function to fetch proto files from a Git repository
    fetch_chain_protos() {
      local chain=$1
      local tag=$2
      local repo=$3
      local proto_path=$4
      local output_dir=$5
      
      echo "Fetching $chain proto files (version $tag)..."
      
      # Clone the repository at the specific tag
      git clone --depth 1 --branch "$tag" "$repo" "$TMP_DIR/$chain"
      
      # Create output directory
      mkdir -p "$output_dir/$chain"
      
      # Copy proto files
      cp -r "$TMP_DIR/$chain/$proto_path"/* "$output_dir/$chain/"
      
      echo "Finished fetching $chain proto files"
    }
    
    # Function to fetch dependencies
    fetch_dependencies() {
      echo "Fetching proto dependencies..."
      
      # Fetch gogoproto
      curl -sL https://raw.githubusercontent.com/cosmos/gogoproto/main/gogo.proto -o "$THIRD_PARTY_DIR/gogoproto/gogo.proto"
      
      # Fetch cosmos_proto
      curl -sL https://raw.githubusercontent.com/cosmos/cosmos-proto/main/cosmos.proto -o "$THIRD_PARTY_DIR/cosmos_proto/cosmos.proto"
      
      # Fetch amino.proto
      curl -sL https://raw.githubusercontent.com/cosmos/cosmos-sdk/main/proto/amino/amino.proto -o "$THIRD_PARTY_DIR/amino/amino.proto"
      
      echo "All proto dependencies fetched successfully"
    }
    
    # Main execution
    main() {
      local chains=()
      local fetch_deps=true
      
      # Parse command line arguments
      while [[ $# -gt 0 ]]; do
        case "$1" in
          --all)
            chains=("noble" "osmosis" "cosmos-sdk")
            shift
            ;;
          --noble)
            chains+=("noble")
            shift
            ;;
          --osmosis)
            chains+=("osmosis")
            shift
            ;;
          --cosmos-sdk)
            chains+=("cosmos-sdk")
            shift
            ;;
          --no-deps)
            fetch_deps=false
            shift
            ;;
          *)
            echo "Unknown option: $1"
            exit 1
            ;;
        esac
      done
      
      # Default to all chains if none specified
      if [[ ''${#chains[@]} -eq 0 ]]; then
        chains=("noble" "osmosis" "cosmos-sdk")
      fi
      
      # Fetch dependencies if requested
      if [[ "$fetch_deps" == true ]]; then
        fetch_dependencies
      fi
      
      # Fetch requested chain protos
      for chain in "''${chains[@]}"; do
        case "$chain" in
          noble)
            fetch_chain_protos "noble" "${versions.noble.tag}" "${versions.noble.repo}" "${versions.noble.protoPath}" "$PROTO_DIR"
            ;;
          osmosis)
            fetch_chain_protos "osmosis" "${versions.osmosis.tag}" "${versions.osmosis.repo}" "${versions.osmosis.protoPath}" "$PROTO_DIR"
            ;;
          cosmos-sdk)
            fetch_chain_protos "cosmos-sdk" "${versions.cosmos-sdk.tag}" "${versions.cosmos-sdk.repo}" "${versions.cosmos-sdk.protoPath}" "$PROTO_DIR"
            ;;
        esac
      done
      
      echo "Proto files have been successfully fetched to $PROTO_DIR"
      echo "Run 'cargo build' to regenerate protocol buffer code"
    }
    
    main "$@"
  '';

in pkgs.stdenv.mkDerivation {
  inherit name;
  buildInputs = [
    pkgs.git
    pkgs.curl
    script
  ];
  
  # Simple shell wrapper to make it easy to run
  phases = [ "installPhase" ];
  installPhase = ''
    mkdir -p $out/bin
    ln -s ${script}/bin/${name} $out/bin/${name}
  '';
  
  meta = {
    description = "Fetches protocol buffer definitions from blockchain projects";
    mainProgram = name;
  };
}
