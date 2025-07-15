{
  description = "Valence Domain Clients - Client implementations for interacting with Valence Protocol domains";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crate2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain configuration (from rust.nix)
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        crate2nixTools = crate2nix.packages.${system}.default;

        # Common library configuration (from lib.nix)
        commonLib = rec {
          # Common build inputs for all configurations
          buildInputs = with pkgs; [
            # Build tools
            pkg-config
            cmake
            
            # Linker optimization
            lld

            # Crypto and system libraries
            openssl
            protobuf
            
            # For cosmos gRPC/protobuf dependencies
            grpc
            
            # Database libraries (for potential use)
            sqlite
            rocksdb
            
            # Additional crypto libraries
            perl # needed for ring crate
            
            # Development tools
            git
            curl
            cacert
            
            # File watching for auto-reload
            entr
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # Linux specific dependencies
            libudev-zero
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # macOS specific dependencies
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # Common environment variables
          envVars = {
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.protobuf}/lib/pkgconfig";
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${pkgs.protobuf}/include";
            LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (with pkgs; [
              openssl protobuf grpc sqlite rocksdb
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.SystemConfiguration
            ])}";
            MACOSX_DEPLOYMENT_TARGET = "10.12";
            
            # Build optimization environment variables
            NIX_BUILD_CORES = "0"; # Use all available cores
            CARGO_NET_GIT_FETCH_WITH_CLI = "true"; # Use git CLI for better caching
            CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse"; # Use sparse protocol for faster updates
            
            # Rust compilation flags for development
            RUSTFLAGS = "-C target-cpu=native -C opt-level=2";
            
            # Enable incremental compilation for development
            CARGO_INCREMENTAL = "1";
          };

          # Common crate overrides for crate2nix
          crateOverrides = pkgs.defaultCrateOverrides // {
            # Override for crates that need OpenSSL
            openssl-sys = attrs: { 
              nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [ pkgs.openssl ];
              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              OPENSSL_DIR = "${pkgs.openssl.dev}";
              OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            };
            
            # Override for crates that need protobuf
            prost-build = attrs: {
              nativeBuildInputs = [ pkgs.protobuf ];
              PROTOC = "${pkgs.protobuf}/bin/protoc";
              PROTOC_INCLUDE = "${pkgs.protobuf}/include";
            };
            
            # Override for tonic build scripts
            tonic-build = attrs: {
              nativeBuildInputs = [ pkgs.protobuf ];
              PROTOC = "${pkgs.protobuf}/bin/protoc";
              PROTOC_INCLUDE = "${pkgs.protobuf}/include";
            };
            
            # Override for ring (crypto library)
            ring = attrs: {
              nativeBuildInputs = [ pkgs.perl ];
            } // pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
              buildInputs = [ pkgs.darwin.apple_sdk.frameworks.Security ];
            };
            
            # Override for secp256k1-sys (crypto library)
            secp256k1-sys = attrs: {
              nativeBuildInputs = [ pkgs.pkg-config ];
            };
            
            # Override for librocksdb-sys (if used)
            librocksdb-sys = attrs: {
              nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [ pkgs.rocksdb ];
              ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
            };
            
            # Override for any -sys crates that might need system libraries
            libsqlite3-sys = attrs: {
              nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [ pkgs.sqlite ];
            };
            
            # Optimize build for our root crate
            valence-domain-clients = attrs: {
              # Enable parallel builds
              NIX_BUILD_CORES = "0"; # Use all available cores
            };
          };

          # Common PATH for applications
          toolPath = "${rustToolchain}/bin:${pkgs.pkg-config}/bin:${pkgs.openssl.dev}/bin:${pkgs.protobuf}/bin:${pkgs.perl}/bin:${pkgs.entr}/bin:${pkgs.python3}/bin";

          # Helper function to create environment for scripts
          mkEnv = additionalVars: let
            allVars = envVars // additionalVars;
          in ''
            export PATH="${rustToolchain}/bin:${pkgs.pkg-config}/bin:${pkgs.openssl.dev}/bin:${pkgs.protobuf}/bin:${pkgs.perl}/bin:${pkgs.entr}/bin:${pkgs.python3}/bin:$PATH"
          '' + pkgs.lib.concatStringsSep "\n" (
            pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${toString value}\"") allVars
          );
        };
        
        # Import other modules
        devshellConfig = import ./nix/devshell.nix { 
          inherit pkgs rustToolchain commonLib; 
          lib = pkgs.lib;
          crate2nixTools = crate2nixTools;
        };
        
        # crate2nix generated project with optimized overrides for maximum granularity
        project = pkgs.callPackage ./Cargo.nix {
          # Enable release builds for better performance
          release = true;
          
          # Use default root features for better caching
          rootFeatures = [ "default" ];
          
          # Use the common crate overrides
          defaultCrateOverrides = commonLib.crateOverrides;
          
          # Additional build options for better performance and granularity
          buildRustCrateForPkgs = pkgs: pkgs.buildRustCrate;
        };

      in
      {
        # Development shell
        devShells.default = devshellConfig.shell;

        # Packages (inline definition)
        packages = {
          # Package definition - build all features by default
          default = project.rootCrate.build.override {
            features = [ "test-utils" "cosmos" "evm" "solana" ];
          };

          # Individual feature packages with optimized granular caching
          test-utils = project.rootCrate.build.override {
            features = [ "test-utils" ];
          };

          cosmos = project.rootCrate.build.override {
            features = [ "cosmos" ];
          };

          evm = project.rootCrate.build.override {
            features = [ "evm" ];
          };

          solana = project.rootCrate.build.override {
            features = [ "solana" ];
          };

          coprocessor = project.rootCrate.build.override {
            features = [ "coprocessor" ];
          };

          # All features package with maximum granularity
          all-features = project.rootCrate.build.override {
            features = [ "test-utils" "cosmos" "evm" "solana" "coprocessor" ];
          };
        };
      });
} 