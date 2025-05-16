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
    base = {
      # Base is an Ethereum L2 and doesn't use protocol buffers
      # These are included for configuration reference only
      mainnet = {
        rpc = "https://mainnet.base.org";
        chainId = 8453;
        explorer = "https://base.blockscout.com/";
      };
      testnet = {
        rpc = "https://sepolia.base.org";
        chainId = 84532;
        explorer = "https://sepolia-explorer.base.org";
      };
    };
  };
  
  # Base derivation for the fetch-protos script
  script = pkgs.writeShellScriptBin name ''
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Set directories using the current working directory
    PROTO_DIR="./src/proto"
    THIRD_PARTY_DIR="./third_party"
    CONFIG_DIR="./src/config"
    TMP_DIR=$(mktemp -d)
    
    # Create proto and third_party directories if they don't exist
    mkdir -p "$PROTO_DIR"
    mkdir -p "$THIRD_PARTY_DIR/gogoproto"
    mkdir -p "$THIRD_PARTY_DIR/cosmos_proto"
    mkdir -p "$THIRD_PARTY_DIR/amino"
    mkdir -p "$CONFIG_DIR"
    
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
    
    # Function to generate Base network configuration
    generate_base_config() {
      echo "Generating Base network configuration..."
      
      # Create a Base network config file
      cat > "$CONFIG_DIR/base_networks.json" << EOF
{
  "networks": {
    "mainnet": {
      "rpc": "https://mainnet.base.org",
      "chainId": 8453,
      "currencySymbol": "ETH",
      "blockExplorer": "https://base.blockscout.com/"
    },
    "sepolia": {
      "rpc": "https://sepolia.base.org",
      "chainId": 84532, 
      "currencySymbol": "ETH",
      "blockExplorer": "https://sepolia-explorer.base.org"
    }
  }
}
EOF
      
      echo "Base network configuration generated at $CONFIG_DIR/base_networks.json"
    }
    
    # Main execution
    main() {
      local chains=()
      local fetch_deps=true
      local gen_base=false
      
      # Parse command line arguments
      while [[ $# -gt 0 ]]; do
        case "$1" in
          --all)
            chains=("noble" "osmosis" "cosmos-sdk")
            gen_base=true
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
          --base)
            gen_base=true
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
      if [[ ''${#chains[@]} -eq 0 && "$gen_base" == false ]]; then
        chains=("noble" "osmosis" "cosmos-sdk")
        gen_base=true
      fi
      
      # Fetch dependencies if requested
      if [[ "$fetch_deps" == true ]]; then
        fetch_dependencies
      fi
      
      # Generate Base configuration if requested
      if [[ "$gen_base" == true ]]; then
        generate_base_config
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
      if [[ "$gen_base" == true ]]; then
        echo "Base network configuration has been generated"
      fi
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
