# Social App

A sample social media application that demonstrates integration with FoldDB.

## Features

- Post creation and display
- User profiles
- Friends list
- Post likes
- Comments

## Getting Started

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the server:
   ```bash
   node server.js
   ```

3. Open the app in your browser:
   ```bash
   open http://localhost:3002
   ```

## Testing

The app includes several types of tests to ensure proper functionality:

### API Tests

Tests the API endpoints directly:

```bash
node run-api-tests.js [--verbose]
```

### FoldDB Integration Tests

Tests the integration with FoldDB:

```bash
node test-folddb.js [--verbose]
```

### UI Tests

Tests the UI components using the test harness:

```bash
./run-tests.sh [--headless] [--suite=SUITE]
```

Where `SUITE` can be one of: `navigation`, `post`, `profile`, `friend`, or `all`.

### End-to-End Integration Tests

There are two types of integration tests:

#### 1. With Simulated FoldDB Client

Tests the entire flow from UI to API to the simulated FoldDB client:

```bash
npm run test:integration [-- --verbose] [-- --headless]
```

#### 2. With Real DataFold Node

Tests the entire flow from UI to API to a real DataFold node:

```bash
npm run test:integration:datafold [-- --verbose] [-- --headless]
```

These comprehensive test suites:
1. Start the necessary services (server and DataFold node for the second test)
2. Launch a browser to interact with the UI
3. Create and interact with posts
4. Verify data is properly persisted
5. Test navigation between views
6. Clean up test data

## Architecture

The app follows a client-server architecture:

```
+----------------+      +----------------+      +----------------+
|                |      |                |      |                |
|  Social App UI |----->| FoldDB Client  |----->|    FoldDB     |
|                |      |                |      |                |
+----------------+      +----------------+      +----------------+
```

1. The Social App UI makes API requests to the server
2. The server uses the FoldDB Client to interact with FoldDB
3. FoldDB handles data storage, validation, and retrieval

For more details on the FoldDB integration, see [FOLDDB_INTEGRATION.md](./FOLDDB_INTEGRATION.md).

## Development

For information on the testing infrastructure, see [TESTING.md](./TESTING.md).
