# Agent Guidelines

All contributors must ensure that every new feature is accompanied by appropriate tests. Write comprehensive tests for each feature you build before submitting your changes.

1. Keep code DRY.
2. Limit file size for code.  Break out into helpers and utilities
3. Limit line length for function.  Break out into separate functions where possible.

For refactoring try to delete more lines than you add.

Find simplifications if you can.

No silent failures.

run cargo test --workspace
run npm tests in fold_node/src/datafold_node/static-react