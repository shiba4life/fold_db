#!/usr/bin/env node

/**
 * API Test Runner for Social App
 * 
 * This script runs the API tests to verify that posts are properly persisted.
 * It starts the API server, runs the tests, and then shuts down the server.
 * 
 * Usage:
 *   node run-api-tests.js [--verbose]
 */

const { spawn, fork } = require('child_process');
const path = require('path');

// Configuration
const PORT = 3002; // Match the server port
const VERBOSE = process.argv.includes('--verbose');

async function runTests() {
  console.log('Starting API test runner...');
  console.log(`Verbose mode: ${VERBOSE ? 'enabled' : 'disabled'}`);
  
  let serverProcess;
  let testProcess;
  let exitCode = 0;
  
  try {
    // Start the server with a custom port
    console.log('Starting API server...');
    serverProcess = fork(path.join(__dirname, 'server.js'), [], {
      env: { ...process.env, PORT },
      stdio: 'pipe'
    });
    
    // Log server output
    serverProcess.stdout?.on('data', (data) => {
      console.log(`[SERVER] ${data.toString().trim()}`);
    });
    
    serverProcess.stderr?.on('data', (data) => {
      console.error(`[SERVER ERROR] ${data.toString().trim()}`);
    });
    
    // Wait for server to start
    await new Promise((resolve) => setTimeout(resolve, 1000));
    
    // Run the API tests
    console.log('Running API tests...');
    
    const args = [path.join(__dirname, 'api-tests.js')];
    if (VERBOSE) {
      args.push('--verbose');
    }
    
    testProcess = spawn('node', args, {
      stdio: 'inherit',
      shell: true,
      env: { ...process.env, API_PORT: PORT }
    });
    
    // Wait for the tests to complete
    exitCode = await new Promise((resolve) => {
      testProcess.on('close', (code) => {
        resolve(code);
      });
    });
    
    if (exitCode === 0) {
      console.log('\n✅ All API tests passed');
    } else {
      console.error('\n❌ API tests failed');
    }
    
  } catch (error) {
    console.error('Error running tests:', error);
    exitCode = 1;
  } finally {
    // Clean up
    if (testProcess && !testProcess.killed) {
      testProcess.kill();
    }
    
    if (serverProcess && !serverProcess.killed) {
      serverProcess.kill();
      console.log('Server process terminated');
    }
    
    process.exit(exitCode);
  }
}

// Run the tests
runTests();
