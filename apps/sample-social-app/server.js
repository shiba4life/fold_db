/**
 * API Server for Social App
 * 
 * This server provides the API endpoints for the Social App.
 * It handles post creation, retrieval, and other operations using FoldDB.
 */

const http = require('http');
const path = require('path');
const url = require('url');
const handler = require('serve-handler');
const foldDBClient = require('./fold_db_client');

// Configuration
const PORT = process.env.PORT || 3002; // Use port 3002 to avoid conflicts
const SCHEMAS_DIR = path.join(__dirname, 'schemas');

// Initialize FoldDB client
(async () => {
  try {
    await foldDBClient.initialize(SCHEMAS_DIR);
    console.log('FoldDB client initialized with schemas');
  } catch (error) {
    console.error('Failed to initialize FoldDB client:', error);
    process.exit(1);
  }
})();

// API handlers
async function handleApiRequest(req, res) {
  const parsedUrl = url.parse(req.url, true);
  const pathname = parsedUrl.pathname;

  // Set CORS headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  // Handle preflight requests
  if (req.method === 'OPTIONS') {
    res.statusCode = 204;
    res.end();
    return;
  }

  // Handle API endpoints
  if (pathname === '/api/execute' && req.method === 'POST') {
    let body = '';
    req.on('data', chunk => {
      body += chunk.toString();
    });

    req.on('end', async () => {
      try {
        const requestData = JSON.parse(body);
        const operation = JSON.parse(requestData.operation);
        
        console.log('Received operation:', operation);
        
        // Handle different operation types
        if (operation.type === 'query') {
          await handleQuery(operation, res);
        } else if (operation.type === 'mutation') {
          await handleMutation(operation, res);
        } else {
          res.statusCode = 400;
          res.setHeader('Content-Type', 'application/json');
          res.end(JSON.stringify({ error: 'Invalid operation type' }));
        }
      } catch (error) {
        console.error('Error processing request:', error);
        res.statusCode = 400;
        res.setHeader('Content-Type', 'application/json');
        res.end(JSON.stringify({ error: error.message }));
      }
    });
  } else {
    // Not an API request, pass to static file handler
    return handler(req, res, {
      public: __dirname
    });
  }
}

async function handleQuery(operation, res) {
  try {
    // Validate schema
    const schema = foldDBClient.getSchema(operation.schema);
    if (!schema) {
      res.statusCode = 400;
      res.setHeader('Content-Type', 'application/json');
      res.end(JSON.stringify({ error: `Schema '${operation.schema}' not found` }));
      return;
    }
    
    // Execute query using FoldDB client
    const results = await foldDBClient.executeQuery(operation);
    
    res.statusCode = 200;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ data: { results } }));
  } catch (error) {
    console.error('Error handling query:', error);
    res.statusCode = 500;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ error: error.message }));
  }
}

async function handleMutation(operation, res) {
  try {
    // Validate schema
    const schema = foldDBClient.getSchema(operation.schema);
    if (!schema) {
      res.statusCode = 400;
      res.setHeader('Content-Type', 'application/json');
      res.end(JSON.stringify({ error: `Schema '${operation.schema}' not found` }));
      return;
    }
    
    // Validate data against schema
    if (operation.data && !foldDBClient.validateAgainstSchema(operation.schema, operation.data)) {
      res.statusCode = 400;
      res.setHeader('Content-Type', 'application/json');
      res.end(JSON.stringify({ error: 'Data does not match schema' }));
      return;
    }
    
    // Execute mutation using FoldDB client
    const result = await foldDBClient.executeMutation(operation);
    
    res.statusCode = 200;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify(result));
  } catch (error) {
    console.error('Error handling mutation:', error);
    res.statusCode = 500;
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ error: error.message }));
  }
}

// Create and start the server
const server = http.createServer(handleApiRequest);

server.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}`);
});

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('Shutting down server...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
});
