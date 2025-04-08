# Social App with Docker-based FoldClient

This is a social app that uses the Docker-based FoldClient to communicate with a DataFold node. The app demonstrates how to use the FoldClient IPC mechanism to interact with the DataFold node from within a Docker container.

## Features

- User creation and querying
- Post creation and querying
- Comment creation and querying
- Remote node discovery and querying

## Requirements

- Rust and Cargo
- Docker
- DataFold node
- FoldClient

## Running the App

There are two ways to run the app:

### Option 1: All-in-One Script

You can run the app using the provided all-in-one script:

```bash
./run_social_app.sh
```

This script will:

1. Start a DataFold node
2. Start the FoldClient
3. Register the social app with the FoldClient
4. Build and run the social app
5. Clean up processes when done

### Option 2: Register and Run (if Node and FoldClient are already running)

If you already have a DataFold node and FoldClient running, you can use:

```bash
./register_and_run.sh
```

This script will:

1. Check if Docker, DataFold node, and FoldClient are running
2. Register the social app with the FoldClient
3. Build and run the social app

## How It Works

The social app uses the FoldClient IPC client to communicate with the DataFold node through the FoldClient. The FoldClient provides a secure and isolated environment for the app to run in, using Docker containers for sandboxing.

The app performs the following operations:

1. Connects to the FoldClient using the provided app ID and token
2. Lists available schemas
3. Creates schemas if they don't exist
4. Creates a test user
5. Queries the user
6. Creates a post
7. Queries all posts
8. Adds a comment to the post
9. Queries comments for the post
10. Discovers remote nodes
11. Queries remote nodes if available

## Docker Sandboxing

The FoldClient uses Docker to sandbox the social app, providing:

- Process isolation
- Network isolation
- File system isolation
- Resource limits
- IPC communication

This ensures that the app can only access the DataFold node through the FoldClient and cannot directly access the node or other system resources.

## Security Considerations

While Docker provides strong isolation, it's important to be aware of potential security issues:

- Container escape vulnerabilities
- Resource exhaustion
- IPC security

The FoldClient mitigates these risks through various security measures, but it's important to keep Docker and the host system up to date and follow security best practices.

## License

This project is licensed under the MIT License.
