const express = require('express');
const axios = require('axios');
const fs = require('fs');
const http = require('http');

const app = express();
const port = 3000;

// Configuration based on environment variables
const config = {
  // Network-based API access
  apiHost: process.env.DATAFOLD_API_HOST || 'datafold-api',
  apiPort: process.env.DATAFOLD_API_PORT || 8080,
  
  // Unix socket-based API access
  apiSocket: process.env.DATAFOLD_API_SOCKET || null
};

// Create API client based on configuration
const createApiClient = () => {
  if (config.apiSocket) {
    console.log(`Using Unix socket communication: ${config.apiSocket}`);
    return axios.create({
      socketPath: config.apiSocket,
      baseURL: 'http://localhost',
      timeout: 5000
    });
  } else {
    console.log(`Using network communication: http://${config.apiHost}:${config.apiPort}`);
    return axios.create({
      baseURL: `http://${config.apiHost}:${config.apiPort}`,
      timeout: 5000
    });
  }
};

const apiClient = createApiClient();

// API routes
app.get('/', (req, res) => {
  res.send('Datafold Sandboxed App - Example application for Datafold sandbox environment');
});

// Query schemas from Datafold API
app.get('/schemas', async (req, res) => {
  try {
    const response = await apiClient.get('/schema');
    res.json(response.data);
  } catch (error) {
    console.error('Error querying schemas:', error.message);
    res.status(500).json({ error: error.message });
  }
});

// Execute a query against Datafold API
app.get('/query/:schema', async (req, res) => {
  try {
    const schema = req.params.schema;
    const fields = req.query.fields ? req.query.fields.split(',') : ['*'];
    
    const response = await apiClient.post('/query', {
      schema,
      fields
    });
    
    res.json(response.data);
  } catch (error) {
    console.error('Error executing query:', error.message);
    res.status(500).json({ error: error.message });
  }
});

// List nodes from Datafold API
app.get('/nodes', async (req, res) => {
  try {
    const response = await apiClient.get('/node');
    res.json(response.data);
  } catch (error) {
    console.error('Error listing nodes:', error.message);
    res.status(500).json({ error: error.message });
  }
});

// Test external network access (should fail in sandbox)
app.get('/test-external', async (req, res) => {
  try {
    const response = await axios.get('https://example.com', { timeout: 3000 });
    res.json({ 
      success: true, 
      message: 'External network access is available (this should not happen in sandbox)' 
    });
  } catch (error) {
    res.json({ 
      success: false, 
      message: 'External network access is blocked (expected in sandbox)',
      error: error.message
    });
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Sandboxed app listening on port ${port}`);
  console.log('Environment:');
  console.log(`  DATAFOLD_API_HOST: ${process.env.DATAFOLD_API_HOST || 'not set'}`);
  console.log(`  DATAFOLD_API_PORT: ${process.env.DATAFOLD_API_PORT || 'not set'}`);
  console.log(`  DATAFOLD_API_SOCKET: ${process.env.DATAFOLD_API_SOCKET || 'not set'}`);
  
  // Test API connection
  console.log('Testing API connection...');
  apiClient.get('/schema')
    .then(() => {
      console.log('Successfully connected to Datafold API');
      console.log('APP_READY: Datafold Sandboxed App is ready');
    })
    .catch(err => {
      console.error('Failed to connect to Datafold API:', err.message);
      console.log('APP_READY: Datafold Sandboxed App is ready (with API connection issues)');
    });
});
