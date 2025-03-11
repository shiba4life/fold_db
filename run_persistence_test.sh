#!/bin/bash

# Run the persistence test script
echo "Installing node-fetch dependency..."
npm install node-fetch@2

echo "Starting persistence test..."
node test_persistence.js
