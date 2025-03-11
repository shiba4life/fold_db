#!/bin/bash

# Start the FoldSocial application
echo "Starting FoldSocial application..."
echo "Make sure your DataFold node is running at http://localhost:8080"
echo ""

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
  echo "Installing dependencies..."
  npm install
fi

# Start the application
echo "Starting the application..."
npm start
