# Integrated DataFold Node with FoldClient

This document explains the changes made to integrate the FoldClient functionality directly into the DataFold node, eliminating the need to run a separate FoldClient process.

## Changes Made

1. **Modified DataFold Node**: The DataFold node now includes the FoldClient functionality, allowing it to handle both node operations and client operations in a single process.

2. **Updated Scripts**: The scripts have been updated to use the integrated FoldClient functionality.

3. **Direct Connection**: The social app now connects directly to the DataFold node's TCP server instead of using the FoldClient IPC mechanism.

## Benefits

- **Simplified Setup**: You no longer need to run a separate FoldClient process, simplifying the setup and reducing resource usage.
- **Reduced Complexity**: The system is now simpler to understand and maintain.
- **Improved Performance**: Direct connection to the DataFold node can improve performance by eliminating the IPC overhead.

## How to Use

### Starting the Integrated Node

To start the DataFold node with integrated FoldClient functionality, run:

```bash
./test_integrated_node.sh
```

This will start the DataFold node with the integrated FoldClient functionality and wait for you to press Ctrl+C to stop it.

### Running the Social App

You can run the social app directly against the DataFold node using the updated scripts:

```bash
cd social_app
./run_social_app.sh
```

or

```bash
cd social_app
./register_and_run.sh
```

## Implementation Details

### DataFold Node Changes

The DataFold node now initializes and starts the FoldClient as part of its startup process. This allows it to handle both node operations and client operations in a single process.

### Social App Changes

The social app has been updated to connect directly to the DataFold node's TCP server instead of using the FoldClient IPC mechanism. This simplifies the code and improves performance.

## Current Limitations and Future Work

The current implementation has some limitations that need to be addressed in future work:

1. **Serialization Issues**: There are serialization issues when creating schemas and performing other operations. The error `Error: Kind(UnexpectedEof)` indicates that the connection is being closed unexpectedly, possibly due to a serialization error on the server side.

2. **Error Handling**: The error handling in the TCP communication between the social app and the DataFold node needs improvement.

3. **Schema Creation**: The schema creation functionality needs to be fixed to handle the serialization issues.

Future work should focus on:

1. Fixing the serialization issues in the TCP server implementation
2. Improving error handling in both the client and server
3. Adding more robust connection management
4. Implementing proper schema validation

## Troubleshooting

If you encounter any issues with the integrated node, try the following:

1. Make sure the DataFold node is running with the integrated FoldClient functionality.
2. Check the logs for any error messages.
3. Make sure the social app is connecting to the correct port (default: 9000).
4. If you encounter the `Error: Kind(UnexpectedEof)` error, this is a known issue with the current implementation that needs to be fixed in future work.
