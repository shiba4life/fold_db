# Git Hooks for DataFold Project

This directory contains Git hooks for the DataFold project to ensure code quality and prevent issues from being committed.

## Available Hooks

### Pre-commit Hook

The pre-commit hook runs before each commit and performs the following checks:

1. Runs `cargo clippy` to check for linting issues
2. Runs main project unit tests
3. Runs main project integration tests
4. Runs network tests
5. Runs SDK unit tests
6. Runs SDK integration tests
7. Runs real integration tests
8. Runs all tests across the entire workspace with `cargo test --workspace`
9. Runs `npm test` for the React UI when JavaScript or TypeScript files are
   committed and a test script is defined

If any of these checks fail, the commit will be aborted.

## Installation

To install the hooks, run the installation script from the project root:

```bash
./install-hooks.sh
```

This will copy the hooks to the `.git/hooks` directory and make them executable.

## Manual Installation

If you prefer to install the hooks manually:

1. Copy the `pre-commit` file to `.git/hooks/`
2. Make it executable:

```bash
cp pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

## Skipping Hooks

In rare cases, you may need to bypass the pre-commit hook. You can do this by using the `--no-verify` flag with your commit:

```bash
git commit --no-verify -m "Your commit message"
```

However, this should be used sparingly and only when absolutely necessary.

## Troubleshooting

If you encounter issues with the hooks:

1. Ensure the hooks are executable (`chmod +x .git/hooks/pre-commit`)
2. Check that you're running the commands from the project root
3. Verify that all dependencies are installed
4. Make sure your Rust toolchain is up to date
