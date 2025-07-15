#!/usr/bin/env bash
# Test all build paths for valence-domain-clients
# This script tests both nix builds and cargo builds with all feature combinations

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
TOTAL=0

# Function to print status
print_status() {
    local status=$1
    local message=$2
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}[PASS]${NC}: $message"
        PASSED=$((PASSED + 1))
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}[FAIL]${NC}: $message"
        FAILED=$((FAILED + 1))
    elif [ "$status" = "INFO" ]; then
        echo -e "${BLUE}[INFO]${NC}: $message"
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}[WARN]${NC}: $message"
    fi
    TOTAL=$((TOTAL + 1))
}

# Function to run command and check result
run_test() {
    local description=$1
    local command=$2
    
    echo -e "${BLUE}Testing:${NC} $description"
    echo -e "${YELLOW}Command:${NC} $command"
    
    if eval "$command" >/dev/null 2>&1; then
        print_status "PASS" "$description"
    else
        print_status "FAIL" "$description"
        echo -e "${RED}Command failed:${NC} $command"
    fi
    echo ""
}

# Function to print section header
print_section() {
    echo -e "\n${BLUE}===========================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}===========================================${NC}\n"
}

# Main test execution
main() {
    print_section "VALENCE DOMAIN CLIENTS - BUILD PATH TESTING"
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -f "flake.nix" ]; then
        print_status "FAIL" "Not in project root directory (missing Cargo.toml or flake.nix)"
        exit 1
    fi
    
    print_status "INFO" "Starting comprehensive build testing"
    print_status "INFO" "Project: valence-domain-clients"
    print_status "INFO" "Working directory: $(pwd)"
    
    # ===========================================
    # NIX BUILD TESTS
    # ===========================================
    print_section "NIX BUILD TESTS"
    
    # Test nix flake commands first
    run_test "Nix flake validation" "nix flake check"
    
    # Test individual nix package builds
    run_test "Nix build: test-utils package" "nix build .#test-utils --no-link"
    run_test "Nix build: cosmos package" "nix build .#cosmos --no-link"
    run_test "Nix build: evm package" "nix build .#evm --no-link"
    run_test "Nix build: solana package" "nix build .#solana --no-link"
    run_test "Nix build: coprocessor package" "nix build .#coprocessor --no-link"
    run_test "Nix build: all-features package" "nix build .#all-features --no-link"
    run_test "Nix build: default package" "nix build .#default --no-link"
    
    # ===========================================
    # CARGO BUILD TESTS (IN NIX SHELL)
    # ===========================================
    print_section "CARGO BUILD TESTS (IN NIX SHELL)"
    
    # Test cargo builds within nix develop environment
    run_test "Cargo build: default features" "nix develop --command cargo build --release"
    run_test "Cargo build: test-utils feature" "nix develop --command cargo build --release --features test-utils"
    run_test "Cargo build: cosmos feature" "nix develop --command cargo build --release --features cosmos"
    run_test "Cargo build: evm feature" "nix develop --command cargo build --release --features evm"
    run_test "Cargo build: solana feature" "nix develop --command cargo build --release --features solana"
    run_test "Cargo build: coprocessor feature" "nix develop --command cargo build --release --features coprocessor"
    
    # Test feature combinations
    run_test "Cargo build: cosmos + evm features" "nix develop --command cargo build --release --features 'cosmos,evm'"
    run_test "Cargo build: solana + evm features" "nix develop --command cargo build --release --features 'solana,evm'"
    run_test "Cargo build: all features" "nix develop --command cargo build --release --features 'test-utils,cosmos,evm,solana,coprocessor'"
    
    # ===========================================
    # CARGO CHECK TESTS (FASTER)
    # ===========================================
    print_section "CARGO CHECK TESTS (SYNTAX & TYPE CHECKING)"
    
    run_test "Cargo check: default features" "nix develop --command cargo check"
    run_test "Cargo check: cosmos feature" "nix develop --command cargo check --features cosmos"
    run_test "Cargo check: evm feature" "nix develop --command cargo check --features evm"
    run_test "Cargo check: solana feature" "nix develop --command cargo check --features solana"
    run_test "Cargo check: coprocessor feature" "nix develop --command cargo check --features coprocessor"
    run_test "Cargo check: all features" "nix develop --command cargo check --features 'test-utils,cosmos,evm,solana,coprocessor'"
    
    # ===========================================
    # TEST EXECUTION
    # ===========================================
    print_section "CARGO TEST EXECUTION"
    
    # Run tests for each feature that have meaningful unit tests
    run_test "Cargo test: solana feature (mnemonic test)" "nix develop --command cargo test --release --features solana test_mnemonic_keypair_derivation"
    run_test "Cargo test: cosmos feature (route test)" "nix develop --command cargo test --release --features cosmos test_route_query"
    run_test "Cargo test: coprocessor feature (unit tests)" "nix develop --command cargo test --release --features coprocessor --lib -- --skip client_get_witnesses_works --skip client_prove_works"
    
    # ===========================================
    # SUMMARY
    # ===========================================
    print_section "BUILD TEST SUMMARY"
    
    if [ $FAILED -eq 0 ]; then
        print_status "PASS" "All build paths completed successfully!"
        echo -e "${GREEN}SUCCESS: $PASSED/$TOTAL tests passed${NC}"
        echo -e "${GREEN}All nix builds working${NC}"
        echo -e "${GREEN}All cargo builds working${NC}"
        echo -e "${GREEN}All feature combinations working${NC}"
        echo -e "${GREEN}Key tests passing${NC}"
        echo ""
        echo -e "${BLUE}The valence-domain-clients project is ready for development!${NC}"
    else
        print_status "FAIL" "Some build paths failed"
        echo -e "${RED}FAILURE: $FAILED/$TOTAL tests failed${NC}"
        echo -e "${YELLOW}Please check the failed commands above${NC}"
        exit 1
    fi
}

# Run main function
main "$@" 