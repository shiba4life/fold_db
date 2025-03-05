#!/usr/bin/env node

/**
 * Integration Test Runner for Social App with Real DataFold Node
 * 
 * This script runs the integration tests to verify the entire flow from UI to API to a real DataFold node.
 * It ensures all dependencies are installed before running the tests.
 * 
 * Usage:
 *   node run-integration-tests-with-datafold.js [--verbose] [--headless]
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const VERBOSE = process.argv.includes('--verbose');
const HEADLESS = process.argv.includes('--headless');

// Colors for output
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const BLUE = '\x1b[34m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

// Ensure test results directory exists
const testResultsDir = path.join(__dirname, 'test-results');
if (!fs.existsSync(testResultsDir)) {
  fs.mkdirSync(testResultsDir, { recursive: true });
}

async function checkDependencies() {
  console.log(`${BLUE}Checking dependencies...${RESET}`);
  
  // Check if puppeteer is installed
  try {
    require.resolve('puppeteer');
    console.log(`${GREEN}✓ Puppeteer is installed${RESET}`);
  } catch (error) {
    console.log(`${YELLOW}Installing puppeteer...${RESET}`);
    
    return new Promise((resolve, reject) => {
      const install = spawn('npm', ['install', 'puppeteer', '--save-dev'], {
        stdio: 'inherit',
        shell: true
      });
      
      install.on('close', (code) => {
        if (code === 0) {
          console.log(`${GREEN}✓ Puppeteer installed successfully${RESET}`);
          resolve();
        } else {
          console.error(`${RED}✗ Failed to install puppeteer${RESET}`);
          reject(new Error('Failed to install puppeteer'));
        }
      });
    });
  }
  
  // Check if node-fetch is installed
  try {
    require.resolve('node-fetch');
    console.log(`${GREEN}✓ node-fetch is installed${RESET}`);
  } catch (error) {
    console.log(`${YELLOW}Installing node-fetch...${RESET}`);
    
    return new Promise((resolve, reject) => {
      const install = spawn('npm', ['install', 'node-fetch@2', '--save-dev'], {
        stdio: 'inherit',
        shell: true
      });
      
      install.on('close', (code) => {
        if (code === 0) {
          console.log(`${GREEN}✓ node-fetch installed successfully${RESET}`);
          resolve();
        } else {
          console.error(`${RED}✗ Failed to install node-fetch${RESET}`);
          reject(new Error('Failed to install node-fetch'));
        }
      });
    });
  }
  
  // Check if Rust is installed
  console.log(`${BLUE}Checking if Rust is installed...${RESET}`);
  return new Promise((resolve) => {
    const rustCheck = spawn('cargo', ['--version'], {
      stdio: 'pipe',
      shell: true
    });
    
    rustCheck.on('close', (code) => {
      if (code === 0) {
        console.log(`${GREEN}✓ Rust is installed${RESET}`);
        resolve();
      } else {
        console.error(`${RED}✗ Rust is not installed${RESET}`);
        console.log(`${YELLOW}Please install Rust from https://rustup.rs/ to run tests with the real DataFold node${RESET}`);
        process.exit(1);
      }
    });
  });
}

async function runTests() {
  console.log(`${BLUE}=========================================${RESET}`);
  console.log(`${BLUE}  Social App Integration Tests with DataFold Node  ${RESET}`);
  console.log(`${BLUE}=========================================${RESET}`);
  
  // Check and install dependencies if needed
  await checkDependencies();
  
  // Build command arguments
  const args = [path.join(__dirname, 'integration-tests-with-datafold.js')];
  if (VERBOSE) {
    args.push('--verbose');
  }
  if (HEADLESS) {
    args.push('--headless');
  }
  
  console.log(`${BLUE}Running integration tests with DataFold node...${RESET}`);
  console.log(`Command: node ${args.join(' ')}`);
  console.log(`${BLUE}=========================================${RESET}`);
  
  // Run the tests
  return new Promise((resolve) => {
    const testProcess = spawn('node', args, {
      stdio: 'inherit',
      shell: true
    });
    
    testProcess.on('close', (code) => {
      console.log(`${BLUE}=========================================${RESET}`);
      
      if (code === 0) {
        console.log(`${GREEN}Integration tests with DataFold node completed successfully!${RESET}`);
      } else {
        console.error(`${RED}Integration tests with DataFold node failed with exit code ${code}${RESET}`);
      }
      
      resolve(code);
    });
  });
}

// Run the tests
runTests().then((exitCode) => {
  process.exit(exitCode);
}).catch((error) => {
  console.error(`${RED}Error running tests:${RESET}`, error);
  process.exit(1);
});
