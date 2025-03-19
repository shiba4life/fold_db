# FoldDB Sample Schemas, Mutations, and Queries

This directory contains the centralized repository of sample schemas, mutations, and queries for the FoldDB system. These examples are used by both the web UI and the CLI.

## Files Overview

- `user_profile_schema.json`: A UserProfile schema with various fields and permission policies
- `user_profile2_schema.json`: A UserProfile2 schema that maps fields from the UserProfile schema
- `user_profile_mutations.json`: Sample mutations for creating, updating, and deleting UserProfile records
- `user_profile_queries.json`: Sample queries for both UserProfile and UserProfile2 schemas

## Centralized Example System

These examples serve as the single source of truth for the FoldDB system. They are used by:

1. **Web UI**: The web interface loads these examples via API endpoints when users click "Load Example" buttons
2. **CLI**: The command-line interface can use these examples for testing and demonstration
3. **Documentation**: These examples are referenced in documentation to ensure consistency

This centralized approach ensures that all examples are maintained in one place, avoiding duplication and inconsistencies across different parts of the system.

## Usage in Web UI

The web UI loads these examples through API endpoints:

- `/api/examples` - Lists all available example files
- `/api/examples/{filename}` - Retrieves a specific example file

When users click "Load Example" buttons in the Schema, Query, or Mutation tabs, the UI fetches the appropriate example from these endpoints.

## Usage with CLI

You can use these examples with the FoldDB CLI:

```bash
# Load a schema from an example file
cargo run --bin fold_cli load-schema --file src/datafold_node/examples/user_profile_schema.json

# Execute a query using an example (requires extracting a single query from the array)
# Create a temporary file with a single query from user_profile_queries.json
cargo run --bin fold_cli query --schema UserProfile --json-file path/to/extracted_query.json
```

## Testing Instructions

### 1. Start the FoldDB Node

First, make sure the FoldDB node is running:

```bash
cargo run --bin datafold_node
```

### 2. Load the Schemas

Use the web API to load the schemas:

```bash
# Load UserProfile schema
curl -X POST http://localhost:3000/api/schema \
  -H "Content-Type: application/json" \
  -d @src/datafold_node/examples/user_profile_schema.json

# Load UserProfile2 schema (after loading UserProfile)
curl -X POST http://localhost:3000/api/schema \
  -H "Content-Type: application/json" \
  -d @src/datafold_node/examples/user_profile2_schema.json
```

### 3. Execute Mutations

Create sample user profiles:

```bash
# Create first user profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"johndoe\",\"email\":\"john.doe@example.com\",\"full_name\":\"John Doe\",\"bio\":\"Software developer with 10 years of experience\",\"age\":35,\"location\":\"San Francisco, CA\"}}"}'

# Create second user profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"janedoe\",\"email\":\"jane.doe@example.com\",\"full_name\":\"Jane Doe\",\"bio\":\"UX Designer passionate about user-centered design\",\"age\":28,\"location\":\"New York, NY\"}}"}'

# Create third user profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"create\",\"data\":{\"username\":\"bobsmith\",\"email\":\"bob.smith@example.com\",\"full_name\":\"Bob Smith\",\"bio\":\"Data scientist specializing in machine learning\",\"age\":42,\"location\":\"Seattle, WA\"}}"}'
```

Update existing profiles:

```bash
# Update John Doe's profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"update\",\"filter\":{\"username\":\"johndoe\"},\"data\":{\"bio\":\"Senior software engineer with expertise in distributed systems\",\"location\":\"Austin, TX\"}}"}'

# Update Jane Doe's profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"update\",\"filter\":{\"username\":\"janedoe\"},\"data\":{\"full_name\":\"Jane A. Doe\",\"age\":29}}"}'
```

### 4. Execute Queries

Query UserProfile data:

```bash
# Query all usernames, emails, and bios
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\",\"email\",\"bio\"],\"filter\":null}"}'

# Query John Doe's profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\",\"full_name\",\"location\"],\"filter\":{\"username\":\"johndoe\"}}"}'

# Query profiles with age > 30
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"query\",\"schema\":\"UserProfile\",\"fields\":[\"username\",\"full_name\",\"bio\",\"location\"],\"filter\":{\"age\":{\"gt\":30}}}"}'
```

Query UserProfile2 data (which maps from UserProfile):

```bash
# Query all mapped user data
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"query\",\"schema\":\"UserProfile2\",\"fields\":[\"user_name\",\"contact_email\",\"profile_description\"],\"filter\":null}"}'

# Query John Doe's mapped profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"query\",\"schema\":\"UserProfile2\",\"fields\":[\"user_name\",\"display_name\",\"user_location\"],\"filter\":{\"user_name\":\"johndoe\"}}"}'
```

### 5. Delete Data

Delete a user profile:

```bash
# Delete Bob Smith's profile
curl -X POST http://localhost:3000/api/execute \
  -H "Content-Type: application/json" \
  -d '{"operation": "{\"type\":\"mutation\",\"schema\":\"UserProfile\",\"operation\":\"delete\",\"filter\":{\"username\":\"bobsmith\"}}"}'
```

## Schema Field Mapping

The UserProfile2 schema demonstrates field mapping from the UserProfile schema:

| UserProfile2 Field    | Maps from UserProfile Field |
|-----------------------|----------------------------|
| user_name             | username                   |
| contact_email         | email                      |
| display_name          | full_name                  |
| profile_description   | bio                        |
| user_location         | location                   |

This mapping allows you to create different views of the same underlying data with different field names and potentially different permission policies.
