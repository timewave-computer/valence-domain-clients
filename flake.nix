# flake.nix (simplified example)
{
  # ... other inputs ...
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Specify your desired Rust toolchain
        # You can find stable, beta, nightly versions, or specific versions.
        # See https://github.com/oxalica/rust-overlay for more options.
        rustToolchain = pkgs.rust-bin.stable."latest".default.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rustfmt"
          "rust-std"
          "rust-analyzer" # For IDE support
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          packages = [ # Keep runtime packages or those primarily for user interaction here
            rustToolchain
            # If some packages are only needed for runtime or direct use, not building, they can stay here
          ];

          nativeBuildInputs = [ # Build-time dependencies that need to be on PATH for compilers/build scripts
            # rustToolchain # Often rustToolchain itself is also a nativeBuildInput or its components are.
                          # If rustToolchain is already providing cargo/rustc on PATH, this is fine.
                          # For simplicity, let's assume rustToolchain in 'packages' makes cargo/rustc available.
            pkgs.protobuf
            pkgs.pkg-config
            pkgs.openssl
            # Add any other system dependencies your project might need for building
            # e.g., pkgs.clang, pkgs.llvmPackages.libclang
          ];

          shellHook = ''
            # Ensure the protobuf bin directory is on PATH
            export PATH="${pkgs.protobuf}/bin:$PATH"
            # Explicitly set the PROTOC environment variable
            export PROTOC="${pkgs.protobuf}/bin/protoc"
            # You can add echos here for your own debugging if you enter the shell manually:
            # echo "Nix shell hook: PROTOC explicitly set to $PROTOC"
            # echo "Nix shell hook: PATH is now $PATH"
          '';
        };
      }
    );
}