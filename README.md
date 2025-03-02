# FoldDB

FoldDB is a schema-based database system that provides atomic operations, fine-grained permissions control, and version history tracking. It's built in Rust with a focus on data integrity and access control.

## Key Features

- **Schema-Based Storage**: Define and validate data structure with JSON schemas
- **Field-Level Permissions**: Fine-grained access control at the individual field level
- **Trust-Based Access**: Flexible permissions model using trust distance and explicit policies
- **Atomic Operations**: All data changes are atomic and create new versions
- **Version History**: Track and access the complete history of data changes

## Core Concepts

### Atoms & AtomRefs

- **Atoms**: Immutable data containers that store content and metadata
- **AtomRefs**: References that always point to the latest version of data
- **Version History**: Maintained through linked Atoms, allowing access to previous versions

### Permissions Model

- **Trust Distance**: Lower numbers indicate higher trust levels
- **Field-Level Control**: Permissions can be set for individual data fields
- **Access Policies**: Explicit read/write permissions using public key authentication

### Schema System

- **JSON Schemas**: Define data structure and validation rules
- **Field Definitions**: Specify data types and constraints
- **Permission Rules**: Integrate access control with schema definitions

## Technical Details

- Built in Rust for performance and safety
- Uses sled embedded database for persistent storage
- JSON-based data representation
- No external database dependencies

## Setup

1. Requirements:
   - Rust toolchain
   - Cargo package manager

2. Installation:
   ```bash
   cargo install folddb
   ```

## Usage

```bash
cargo run --bin datafold_node
```

```rust
use folddb::{FoldDB, Schema};

// Initialize database
let db = FoldDB::new("path/to/db")?;

// Load schema
let schema = Schema::from_json(schema_json)?;
db.load_schema("user_profile", schema)?;

// Write data
let data = json!({
    "name": "Alice",
    "email": "alice@example.com"
});
db.write("user_profile", data, public_key)?;

// Read data
let user = db.read("user_profile", "user123")?;
```

## Architecture

FoldDB follows a modular design with clear separation of concerns:

- **FoldDB**: Main entry point and operation coordinator
- **SchemaManager**: Handles schema validation and management
- **PermissionManager**: Controls access and trust calculations
- **Atom Storage**: Manages immutable data storage and versioning

## Technical Constraints

- All data changes create new versions (immutable data model)
- Trust distance must be a positive integer
- Schema must be loaded before data operations
- Write operations require explicit permissions
- Public key required for authentication

## Development

```bash
# Build project
cargo build

# Run tests
cargo test

# Run with example configuration
cargo run --example basic_usage
```

## Best Practices

1. **Schema Design**:
   - Define clear field-level permissions
   - Use appropriate data types
   - Consider versioning requirements

2. **Permissions**:
   - Set appropriate trust distances
   - Use explicit permissions for sensitive data
   - Review access patterns regularly

3. **Data Operations**:
   - Validate data before writing
   - Handle version history appropriately
   - Consider atomic operation boundaries

## Open Source Philosophy

FoldDB is built on the principles of open collaboration and knowledge sharing. We believe that software should be freely available for everyone to use, study, modify, and distribute. By making FoldDB open source, we aim to:

- **Foster Innovation**: Allow anyone to build upon and improve the codebase
- **Ensure Transparency**: Make the inner workings of the database visible to all users
- **Build Community**: Create a collaborative environment where ideas can flourish
- **Promote Learning**: Provide a real-world codebase for developers to study and learn from
- **Ensure Longevity**: Prevent the project from becoming abandoned or inaccessible

We are committed to maintaining FoldDB as a truly open project that anyone can contribute to, use, and modify without restriction.

## Contributing

FoldDB is an open-source project that welcomes contributions from everyone. You are free to use, modify, and distribute this software without limitation. We believe in the power of community-driven development and encourage participation from developers of all backgrounds and skill levels.

### How to Contribute

1. **Fork the Repository**: Create your own fork of the project to work on your changes
2. **Create a Branch**: Make your changes in a new branch with a descriptive name
3. **Submit a Pull Request**: Open a PR with a clear description of your changes and their purpose
4. **Code Review**: Participate in the review process and address any feedback

No contribution is too small - whether it's fixing a typo, improving documentation, adding tests, or implementing new features, all contributions are valued and appreciated.

### Development Guidelines

- Follow the existing code style and patterns for consistency
- Add tests for new functionality to ensure reliability
- Update documentation to reflect your changes
- Keep pull requests focused on a single topic for easier review
- Consider backward compatibility when making changes

### Reporting Issues

- Use the issue tracker to report bugs or suggest enhancements
- Include detailed steps to reproduce the issue
- Mention your environment and FoldDB version
- Feel free to suggest improvements or new features

### Getting Help

- Check existing documentation and issues first
- Don't hesitate to ask questions if something is unclear
- Be respectful and considerate when interacting with other community members

### Project Governance

This project is open for anyone to contribute. There are no special permissions required to make changes - simply fork the repository, make your changes, and submit a pull request. All contributions will be considered based on their technical merit and alignment with the project's goals.

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. We expect all participants to adhere to the following principles:

- Be respectful and considerate
- Be open to collaboration and different viewpoints
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

## License

FoldDB is released under the MIT License, one of the most permissive open-source licenses available.

### What This License Means For You

- **Freedom to Use**: You can use FoldDB for any purpose, including commercial applications, without any restrictions
- **Freedom to Modify**: You can modify the code to suit your needs without requiring approval
- **Freedom to Distribute**: You can distribute your modified versions to anyone
- **Freedom to Contribute**: You can contribute back to the project without transferring your rights
- **No Warranty**: The software is provided "as is" without warranty of any kind

```
MIT License

Copyright (c) 2025 FoldDB Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

This open-source license ensures that FoldDB remains freely available for everyone to use, modify, and distribute without limitation. We believe in the principles of open-source software and are committed to maintaining FoldDB as an open and accessible project for the entire community.
