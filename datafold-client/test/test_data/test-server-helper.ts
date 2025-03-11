import { ChildProcess, spawn } from 'child_process';
import { existsSync, mkdirSync, unlinkSync, rmdirSync } from 'fs';
import { join } from 'path';
import axios from 'axios';

// Test configuration
export const NODE_SERVER_PORT = 8082; // Use a different port than the default to avoid conflicts
export const NODE_SERVER_URL = `http://localhost:${NODE_SERVER_PORT}`;
export const TEST_DATA_DIR = join(__dirname);

// Helper function to wait for the server to start
export const waitForServer = async (url: string, maxRetries = 10, retryDelay = 500): Promise<boolean> => {
  for (let i = 0; i < maxRetries; i++) {
    try {
      await axios.get(`${url}/api/schemas`);
      return true;
    } catch (error) {
      await new Promise(resolve => setTimeout(resolve, retryDelay));
    }
  }
  return false;
};

// Start the test server
export const startTestServer = async (): Promise<ChildProcess> => {
  // Create test data directory if it doesn't exist
  if (!existsSync(TEST_DATA_DIR)) {
    mkdirSync(TEST_DATA_DIR, { recursive: true });
  }

  console.log('Starting DataFold Node server for integration tests...');
  
  // Start the server
  const serverFilePath = join(TEST_DATA_DIR, 'test-server.js');
  const nodeProcess = spawn('node', [serverFilePath], {
    stdio: 'inherit',
    detached: true
  });

  // Wait for the server to start
  const serverStarted = await waitForServer(NODE_SERVER_URL);
  if (!serverStarted) {
    throw new Error('Failed to start DataFold Node server for integration tests');
  }

  return nodeProcess;
};

// Stop the test server
export const stopTestServer = (nodeProcess: ChildProcess): void => {
  if (nodeProcess && nodeProcess.pid) {
    console.log('Stopping DataFold Node server...');
    if (process.platform === 'win32') {
      // Windows
      spawn('taskkill', ['/pid', nodeProcess.pid.toString(), '/f', '/t']);
    } else {
      // Unix-like
      try {
        process.kill(nodeProcess.pid, 'SIGINT');
      } catch (error) {
        console.error('Error stopping server:', error);
      }
    }
  }
};

// Clean up test data
export const cleanupTestData = (): void => {
  try {
    if (existsSync(join(TEST_DATA_DIR, 'schemas.json'))) {
      unlinkSync(join(TEST_DATA_DIR, 'schemas.json'));
    }
    if (existsSync(join(TEST_DATA_DIR, 'data.json'))) {
      unlinkSync(join(TEST_DATA_DIR, 'data.json'));
    }
  } catch (error) {
    console.error('Error cleaning up test data:', error);
  }
};
