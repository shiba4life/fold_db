// Main server file for the DataFold Social App
const express = require('express');
const path = require('path');
const apiServer = require('./js/server');

// Create Express app
const app = express();

// Serve static files from the social_app directory
app.use(express.static(path.join(__dirname)));

// Serve static files from the parent directory (for shared CSS and JS)
app.use(express.static(path.join(__dirname, '..')));

// Use the API server for /api routes
app.use(apiServer);

// Serve the main HTML file for all other routes (SPA support)
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, 'index.html'));
});

// Start the server
const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`DataFold Social App running on port ${PORT}`);
  console.log(`Open http://localhost:${PORT} in your browser`);
});
