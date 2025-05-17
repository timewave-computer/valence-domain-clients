#!/bin/bash
#-----------------------------------------------------------------------------
# Workspace Migration Helper Script
#-----------------------------------------------------------------------------
# This script helps with migrating the codebase to a workspace structure
# by updating import paths in source files.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to display step information
step() {
  echo -e "${BLUE}==> $1${NC}"
}

# Function to display success
success() {
  echo -e "${GREEN}✓ $1${NC}"
}

# Function to display warning
warning() {
  echo -e "${YELLOW}! $1${NC}"
}

# Function to display error
error() {
  echo -e "${RED}✗ $1${NC}" >&2
}

#-----------------------------------------------------------------------------
# Check prerequisites
#-----------------------------------------------------------------------------
step "Checking prerequisites"

if ! command -v cargo &> /dev/null; then
  error "cargo is not installed"
  exit 1
fi

if ! command -v grep &> /dev/null || ! command -v sed &> /dev/null; then
  error "grep and sed are required"
  exit 1
fi

success "Prerequisites checked"

#-----------------------------------------------------------------------------
# Update import paths in all Rust files
#-----------------------------------------------------------------------------
step "Updating import paths in Rust files"

# Process EVM crate files
find crates/evm/src -name "*.rs" | while read -r file; do
  echo "Processing $file"
  
  # Replace crate::core:: with valence_core::
  sed -i.bak 's/crate::core::/valence_core::/g' "$file" && rm -f "${file}.bak"
  
  # Replace crate::evm:: with crate:: for internal files
  sed -i.bak 's/crate::evm::/crate::/g' "$file" && rm -f "${file}.bak"
  
  # Replace crate::proto:: with valence_proto::
  sed -i.bak 's/crate::proto::/valence_proto::/g' "$file" && rm -f "${file}.bak"
done

# Process Cosmos crate files
find crates/cosmos/src -name "*.rs" | while read -r file; do
  echo "Processing $file"
  
  # Replace crate::core:: with valence_core::
  sed -i.bak 's/crate::core::/valence_core::/g' "$file" && rm -f "${file}.bak"
  
  # Replace crate::cosmos:: with crate:: for internal files
  sed -i.bak 's/crate::cosmos::/crate::/g' "$file" && rm -f "${file}.bak"
  
  # Replace crate::proto:: with valence_proto::
  sed -i.bak 's/crate::proto::/valence_proto::/g' "$file" && rm -f "${file}.bak"
done

success "Updated import paths in source files"

#-----------------------------------------------------------------------------
# Check for remaining instances that need manual updating
#-----------------------------------------------------------------------------
step "Checking for remaining instances that need manual updating"

# Look for remaining crate:: prefixes that might need updating
count_remaining=$(grep -r "crate::" --include="*.rs" crates/ | wc -l)
if [ "$count_remaining" -gt 0 ]; then
  warning "Found $count_remaining lines with crate:: that might need manual review"
  grep -r "crate::" --include="*.rs" crates/ | head -n 10
  
  if [ "$count_remaining" -gt 10 ]; then
    echo "... and $(($count_remaining - 10)) more"
  fi
fi

#-----------------------------------------------------------------------------
# Run cargo check to see if there are any compilation issues
#-----------------------------------------------------------------------------
step "Running cargo check to find compilation issues"
echo "This may take a moment..."

set +e
cargo check > /tmp/cargo_check_output.txt 2>&1
check_status=$?
set -e

if [ $check_status -eq 0 ]; then
  success "Cargo check passed successfully!"
else
  error "Cargo check failed. See error details below:"
  cat /tmp/cargo_check_output.txt
fi

#-----------------------------------------------------------------------------
# Final advice
#-----------------------------------------------------------------------------
step "Migration helper completed"
echo ""
echo "Next steps:"
echo "1. Review the remaining 'crate::' references that might need updating"
echo "2. Fix any compilation errors shown by cargo check"
echo "3. Refer to WORKSPACE_MIGRATION.md for more detailed guidance"
echo ""
success "Done" 