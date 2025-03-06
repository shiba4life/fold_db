# Datafold Sandboxed App Example

This is an example application that demonstrates how to run a third-party application in the Datafold sandbox environment and interact with the Datafold API.

## Overview

This example application is a simple Node.js Express server that provides endpoints for interacting with the Datafold API. It demonstrates:

1. How to connect to the Datafold API from within a sandboxed container
2. How to query schemas from the Datafold API
3. How to execute queries against the Datafold API
4. How to list nodes from the Datafold API
5. How to test that external network access is blocked (as expected in the sandbox)

## Building the Application

To build the Docker image for this application:

```bash
cd examples/sandboxed-app
docker build -t datafold-sandboxed-app .
```

## Running in the Sandbox Environment

### Prerequisites

1. Make sure the Datafold sandbox environment is set up and running:

```bash
cd /path/to/datafold
./setup_sandbox.sh
```

### Network-based Communication

To run the application with network-based access to the Datafold API:

```bash
docker run --rm -p 3000:3000 \
  --network=datafold_internal_network \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  --env DATAFOLD_API_HOST=datafold-api \
  --env DATAFOLD_API_PORT=8080 \
  datafold-sandboxed-app
```

### Unix Socket Communication

For maximum isolation, you can use Unix socket communication:

```bash
docker run --rm -p 3000:3000 \
  --network=none \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  -v /var/run/datafold.sock:/datafold.sock \
  --env DATAFOLD_API_SOCKET=/datafold.sock \
  datafold-sandboxed-app
```

## Testing the Application

Once the application is running, you can access it at http://localhost:3000.

The following endpoints are available:

- `GET /` - Home page
- `GET /schemas` - List all schemas from the Datafold API
- `GET /query/:schema?fields=field1,field2` - Execute a query against the specified schema
- `GET /nodes` - List all nodes from the Datafold API
- `GET /test-external` - Test that external network access is blocked

## Expected Behavior

1. The application should be able to connect to the Datafold API
2. The application should be able to query schemas, execute queries, and list nodes
3. The application should NOT be able to access the external internet (the `/test-external` endpoint should fail)

## Troubleshooting

If the application cannot connect to the Datafold API:

1. Make sure the Datafold API container is running
2. Check that the application is on the correct network (`datafold_internal_network`)
3. Verify that the environment variables are set correctly

For Unix socket communication:

1. Make sure the socket exists at `/var/run/datafold.sock`
2. Check that the socket has the correct permissions
3. Verify that the volume mount is correct in the Docker run command
