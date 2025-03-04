#!/usr/bin/env node

const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');
const http = require('http');
const handler = require('serve-handler');

// Configuration
const PORT = 3000;
const TEST_TIMEOUT = 60000; // 60 seconds
const HEADLESS = process.argv.includes('--headless');
const SUITE = process.argv.find(arg => arg.startsWith('--suite='))?.split('=')[1] || 'all';

// Create a simple HTTP server to serve the app files
const server = http.createServer((request, response) => {
  return handler(request, response, {
    public: path.resolve(__dirname),
  });
});

async function runTests() {
  console.log('Starting test runner...');
  console.log(`Test suite: ${SUITE}`);
  console.log(`Headless mode: ${HEADLESS ? 'enabled' : 'disabled'}`);
  
  let browser;
  let exitCode = 0;
  
  try {
    // Start the server
    await new Promise((resolve) => {
      server.listen(PORT, () => {
        console.log(`Server running at http://localhost:${PORT}`);
        resolve();
      });
    });
    
    // Launch browser
    browser = await puppeteer.launch({
      headless: HEADLESS ? 'new' : false,
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
      defaultViewport: { width: 1024, height: 768 }
    });
    
    const page = await browser.newPage();
    
    // Set up console logging
    page.on('console', message => {
      const type = message.type().substr(0, 3).toUpperCase();
      const text = message.text();
      
      // Filter out noisy messages
      if (text.includes('webpack') || text.includes('DevTools')) return;
      
      console.log(`[BROWSER ${type}] ${text}`);
    });
    
    // Enable more verbose logging
    page.on('pageerror', error => {
      console.error('Page error:', error.message);
    });
    
    page.on('requestfailed', request => {
      console.error(`Request failed: ${request.url()}`);
    });
    
    // Navigate to the test harness
    await page.goto(`http://localhost:${PORT}/test-harness.html`, {
      waitUntil: 'networkidle2',
      timeout: TEST_TIMEOUT
    });
    
    console.log('Test harness loaded');
    
    // Wait for the test harness to be ready
    await page.waitForSelector('#run-all-tests', { timeout: TEST_TIMEOUT });
    
    // Run the appropriate test suite
    let testButton;
    switch (SUITE) {
      case 'navigation':
        testButton = '#run-navigation-tests';
        break;
      case 'post':
        testButton = '#run-post-tests';
        break;
      case 'profile':
        testButton = '#run-profile-tests';
        break;
      case 'friend':
        testButton = '#run-friend-tests';
        break;
      case 'all':
      default:
        testButton = '#run-all-tests';
        break;
    }
    
    console.log(`Running test suite: ${SUITE}`);
    await page.click(testButton);
    
    // Wait for tests to complete (look for the summary marker)
    await page.waitForFunction(
      () => document.querySelector('#test-summary-complete') !== null,
      { timeout: TEST_TIMEOUT }
    );
    
    // Extract test results
    const testResults = await page.evaluate(() => {
      const resultsElement = document.getElementById('test-results');
      return {
        text: resultsElement.textContent,
        html: resultsElement.innerHTML,
        failed: resultsElement.textContent.includes('Failed:') && 
                !resultsElement.textContent.includes('Failed: 0')
      };
    });
    
    console.log('\nTest Results:');
    console.log(testResults.text);
    
    // Save test results to file
    const resultsDir = path.join(__dirname, 'test-results');
    if (!fs.existsSync(resultsDir)) {
      fs.mkdirSync(resultsDir);
    }
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const resultsFile = path.join(resultsDir, `results-${SUITE}-${timestamp}.html`);
    
    fs.writeFileSync(resultsFile, `
      <!DOCTYPE html>
      <html>
      <head>
        <title>Test Results - ${SUITE} - ${timestamp}</title>
        <style>
          body { font-family: Arial, sans-serif; margin: 20px; }
          .success { color: green; }
          .failure { color: red; }
          h1 { color: #333; }
          pre { background: #f5f5f5; padding: 10px; border-radius: 5px; }
        </style>
      </head>
      <body>
        <h1>Test Results - ${SUITE} - ${timestamp}</h1>
        <div>${testResults.html}</div>
      </body>
      </html>
    `);
    
    console.log(`\nTest results saved to: ${resultsFile}`);
    
    // Take screenshot
    const screenshotFile = path.join(resultsDir, `screenshot-${SUITE}-${timestamp}.png`);
    await page.screenshot({ path: screenshotFile, fullPage: true });
    console.log(`Screenshot saved to: ${screenshotFile}`);
    
    // Set exit code based on test results
    if (testResults.failed) {
      console.log('\n❌ Tests failed');
      exitCode = 1;
    } else {
      console.log('\n✅ All tests passed');
      exitCode = 0;
    }
    
  } catch (error) {
    console.error('Error running tests:', error);
    exitCode = 1;
  } finally {
    // Clean up
    if (browser) {
      await browser.close();
    }
    
    server.close(() => {
      console.log('Server closed');
      process.exit(exitCode);
    });
  }
}

// Run the tests
runTests();
