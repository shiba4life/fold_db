#!/bin/bash

# Script to install git hooks for the project

set -e  # Exit immediately if a command exits with a non-zero status

echo "Installing git hooks..."

# Check if .git directory exists
if [ ! -d ".git" ]; then
    echo "No .git directory found. Are you in the root of the git repository?"
    echo "If this is not a git repository yet, initialize it with 'git init' first."
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
cp pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

echo "Pre-commit hook installed successfully!"
echo "The hook will run all tests before each commit."
