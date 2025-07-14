# Development shell configuration for valence-domain-clients
{ pkgs, lib, rustToolchain, crate2nixTools, commonLib }:

{
  # Development shell configuration
  shell = pkgs.mkShell {
    buildInputs = [ rustToolchain crate2nixTools ] ++ commonLib.buildInputs;
    
    name = "valence-domain-clients-dev";
    
    shellHook = ''
      # Set up environment variables
      ${commonLib.mkEnv {}}
      
      echo "Valence Domain Clients development environment loaded!"
      echo "Rust version: $(rustc --version)"
      echo "Available features: test-utils, cosmos, evm, solana"
      echo "Build cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo '1')"
      echo ""

      echo "Nix build commands (optimized for crate2nix):"
      echo "  crate2nix generate             # Generate Cargo.nix from Cargo.toml"
      echo "  nix build                      # Build all features (default)"
      echo "  nix build .#test-utils         # Build with test-utils feature only"
      echo "  nix build .#cosmos             # Build with cosmos feature only"
      echo "  nix build .#evm                # Build with evm feature only"
      echo "  nix build .#solana             # Build with solana feature only"
      echo "  nix build .#all-features       # Build with all features (same as default)"
      echo "  # Compose features manually with .override { features = [...]; }"
      echo ""
      echo "Development commands:"
      echo "  cargo build --release          # Fast incremental build"
      echo "  cargo test                     # Run tests"
    '';
  };
} 