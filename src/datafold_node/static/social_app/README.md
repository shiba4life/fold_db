# DataFold Social App

A sample social application built on the DataFold distributed database system. This application demonstrates how to build a decentralized social network using the DataFold SDK and sandboxed application architecture.

## Architecture

This sample app follows the architecture described in the DataFold Social App Architecture document:

- **Frontend**: HTML, CSS, and JavaScript UI for the social app
- **Backend**: Express.js server that simulates the DataFold node API
- **DataFold SDK**: Client library for interacting with the DataFold network

In a production environment, the backend would use the actual DataFold SDK to communicate with the DataFold network, but for this sample, we're using an in-memory data store to simulate the network.

## Features

- User profiles with bio and full name
- Creating and viewing posts
- Adding comments to posts
- Connecting to different nodes in the network

## Running the App

### Prerequisites

- Node.js (v14 or later)
- npm (v6 or later)

### Installation

1. Navigate to the social app directory:

```bash
cd src/datafold_node/static/social_app
```

2. Install dependencies:

```bash
npm install
```

### Starting the Server

Start the Express.js server:

```bash
npm start
```

For development with auto-reload:

```bash
npm run dev
```

### Accessing the App

Once the server is running, open your browser and navigate to:

```
http://localhost:3000
```

You can log in with any username and node ID. For testing, you can use:

- Username: `alice` or `bob` (pre-populated users)
- Node ID: `node1` (any string will work)

This will connect you to the simulated DataFold node and allow you to interact with the social app.

## API Endpoints

The following API endpoints are available:

- `POST /api/connect`: Connect to a DataFold node
- `GET /api/posts`: Get all posts
- `POST /api/posts`: Create a new post
- `GET /api/profile/:username`: Get a user's profile
- `PUT /api/profile`: Update a user's profile
- `GET /api/comments/:postId`: Get comments for a post
- `POST /api/comments`: Add a comment to a post

## Integration with DataFold

In a real implementation, this app would use the DataFold SDK to:

1. Create a sandboxed container for the app
2. Establish secure communication with the local DataFold node
3. Query and mutate data through the node
4. Discover and communicate with remote nodes in the network

The current implementation simulates these features with an in-memory data store.

## Security Considerations

The full DataFold Social App architecture includes:

- Complete network isolation for the app container
- Fine-grained permissions for data access
- Cryptographic authentication for all requests
- Trust distance limitations for remote node access

These security features would be implemented in a production version of the app.

## Next Steps

Future enhancements to this sample app could include:

1. Integration with the actual DataFold SDK
2. Implementation of the sandboxed container architecture
3. Support for more complex social features (likes, shares, etc.)
4. Enhanced privacy controls for users
5. Support for multimedia content
