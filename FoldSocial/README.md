# FoldSocial

A simple social media application that allows a single user to add posts and see previous posts. This application uses the DataFold client to interact with the DataFold database system.

## Features

- Create new posts with author name and content
- View all posts in chronological order (newest first)
- Responsive design for desktop and mobile devices

## Technologies Used

- Node.js and Express for the server
- EJS for templating
- DataFold client for database operations
- CSS for styling

## Prerequisites

- Node.js installed

## About the DataFold Node

FoldSocial includes a simple implementation of a DataFold Node server that:
- Exposes an HTTP API on port 8080
- Stores schemas and data in memory and persists to JSON files
- Provides the same API interface as the full DataFold system
- Supports schema management, queries, and mutations

## Installation

1. Clone the repository
2. Install dependencies:

```bash
cd FoldSocial
npm install
```

## Running the Application

### Option 1: Start with DataFold Node (Recommended)

Use the provided script to start both the DataFold node and the FoldSocial app:

```bash
./start-with-node.sh
```

This script will:
1. Check if the DataFold node is running
2. Start the DataFold node if it's not running
3. Populate the database with sample posts
4. Start the FoldSocial application

### Option 2: Start Components Separately

You can start the DataFold Node server and the FoldSocial app separately:

```bash
# Start the DataFold Node server
./start-node.sh

# In another terminal, start the FoldSocial app
npm start

# For development with auto-restart
npm run dev
```

The application will be available at http://localhost:3000

## Testing Schema Loading

FoldSocial now supports schema loading from files using the updated DataFold client. To test this functionality:

```bash
# Run the schema loading test
./run-schema-loading-test.sh
```

This script will:
1. Check if the DataFold node is running
2. Load the Post schema from a file using the new schema loading API
3. Create a test post
4. Fetch and display all posts

The schema file is located at `data/post-schema.json` and can be modified to test different schema configurations.

## How It Works

1. The application connects to a DataFold node using the DataFold client
2. On startup, it ensures that the Post schema exists in the database
3. When a user submits a new post, it creates a new record in the Post schema
4. The home page displays all posts sorted by timestamp (newest first)

## Schema Structure

The Post schema has the following fields:
- `id`: Unique identifier for the post
- `content`: The content of the post
- `author`: The name of the author
- `timestamp`: When the post was created

## License

MIT
