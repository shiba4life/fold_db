#!/bin/bash

# Generate code coverage using cargo-llvm-cov
# Installs cargo-llvm-cov if not already available and runs coverage

set -e

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov
fi

# Run coverage for the entire workspace and output HTML report
cargo llvm-cov --workspace --html

