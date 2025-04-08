# Sample Social App for FoldClient

This is a sample social app that demonstrates how to use the FoldClient to interact with a DataFold node in a Docker-sandboxed environment. The app provides a simple social network with users, posts, and comments.

## Features

- Create and view users
- Create and view posts
- Add and view comments on posts
- Discover remote nodes
- Query data from remote nodes

## Prerequisites

- FoldClient installed and running
- A DataFold node running (local or remote)
- Docker installed
- Rust and Cargo installed

## Building the App

To build the app, run:

```bash
cd new_fold_client/examples/sample_social_app
cargo build
```

## Running the App with Docker

The app is designed to be run in the FoldClient Docker sandbox. To run it:

1. Start the FoldClient
2. Register the app with the FoldClient:

```bash
# In a separate terminal
cd new_fold_client
cargo run --bin register_app -- --name "Sample Social App" --permissions list_schemas,query,mutation,discover_nodes,query_remote
```

This will output an app ID and token.

3. Build the Docker image:

```bash
cd examples/sample_social_app
docker build -t sample-social-app .
```

4. Launch the app with the FoldClient:

```bash
# In the new_fold_client directory
cargo run --bin fold_client -- launch --app-id <APP_ID> --image sample-social-app
```

Replace `<APP_ID>` with the app ID from step 2.

## Running the App Locally (for Development)

For development purposes, you can run the app locally:

1. Start the FoldClient
2. Register the app with the FoldClient (as in step 2 above)
3. Run the app with the appropriate environment variables:

```bash
cd examples/sample_social_app
FOLD_CLIENT_APP_ID=<APP_ID> FOLD_CLIENT_APP_TOKEN=<TOKEN> FOLD_CLIENT_SOCKET_DIR=<SOCKET_DIR> cargo run
```

Replace `<APP_ID>`, `<TOKEN>`, and `<SOCKET_DIR>` with the appropriate values.

## API Endpoints

The app provides the following API endpoints:

- `GET /api/users` - List all users
- `POST /api/users` - Create a new user
- `GET /api/posts` - List all posts
- `POST /api/posts` - Create a new post
- `GET /api/comments` - List all comments
- `POST /api/comments` - Create a new comment
- `GET /api/posts/:id/comments` - Get comments for a specific post
- `GET /api/discover-nodes` - Discover remote nodes

## Data Models

### User

```json
{
  "id": "string",
  "username": "string",
  "full_name": "string",
  "bio": "string",
  "created_at": "string"
}
```

### Post

```json
{
  "id": "string",
  "title": "string",
  "content": "string",
  "author_id": "string",
  "created_at": "string"
}
```

### Comment

```json
{
  "id": "string",
  "content": "string",
  "author_id": "string",
  "post_id": "string",
  "created_at": "string"
}
```

## Docker Sandboxing

The app is designed to run in a Docker container managed by the FoldClient. The FoldClient provides:

- Process isolation through Docker containers
- Resource limits (CPU, memory, storage)
- Network isolation (optional)
- File system isolation
- IPC communication with the DataFold node through the FoldClient

For more information on the Docker sandboxing implementation, see the [DOCKER.md](../../DOCKER.md) file in the FoldClient repository.

## Security Considerations

When running the app in the FoldClient Docker sandbox, the following security considerations apply:

- The app runs in an isolated Docker container with limited privileges
- The app can only access the DataFold node through the FoldClient
- The app can only perform operations it has permission for
- The app's resource usage is limited by the Docker container

## License

This project is licensed under the MIT License - see the LICENSE file for details.
