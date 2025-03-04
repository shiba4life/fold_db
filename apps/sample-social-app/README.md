# Sample Social App

A demonstration social networking application for Datafold.

## Overview

This application provides a simple social networking interface with features like:
- User profiles
- Post creation and viewing
- Friend management
- Feed display

## Testing Infrastructure

This project includes a comprehensive testing infrastructure designed to efficiently test the app without consuming excessive context window space.

### Key Components

#### 1. Test Hooks in the App
Added data-testid attributes to key UI elements in index.html, making selectors more reliable and easier to maintain.

#### 2. Test Harness (test-harness.html)
A standalone HTML page that:
- Loads the app in an iframe
- Initializes it with test-specific mock data
- Provides a UI to run specific test suites
- Displays test results in real-time

#### 3. Focused Test Suites
Tests are organized into focused suites:
- Navigation Tests: View switching (Feed, Profile, Friends)
- Post Tests: Post creation and display
- Profile Tests: Profile information display
- Friend Tests: Friends list and suggestions

#### 4. Automated Test Runner (test-runner.js)
A Node.js script that:
- Runs tests using Puppeteer
- Supports headless mode for CI/CD
- Generates HTML reports and screenshots
- Can run specific test suites

#### 5. Single Test Runner (run-single-test.js)
A script for running individual test cases:
- Useful for debugging specific issues
- Provides detailed error messages
- Runs faster than the full test suite

#### 6. Shell Script and npm Integration
Convenience scripts:
- run-tests.sh for command-line execution
- npm scripts for different test scenarios
- GitHub Actions workflow for CI/CD integration

### Benefits of This Approach

1. **Efficiency**: Tests run in isolation without consuming API context window
   - Uses mock data instead of real API calls
   - Focused test suites target specific functionality
   - Single test runner for debugging specific issues

2. **Speed**: Tests run quickly with minimal setup
   - No need to initialize the full database
   - Reduced API call overhead
   - Faster feedback loop during development

3. **Reliability**: Test hooks make selectors more reliable
   - data-testid attributes for stable element selection
   - Isolated test environment prevents cross-test interference
   - Consistent test data for predictable results

4. **Visibility**: Clear test results with pass/fail indicators
   - Real-time test status in the UI
   - Detailed error messages for failed tests
   - HTML reports and screenshots for documentation

## Getting Started

### Installation

```bash
# Install dependencies
npm install
```

### Running the App

```bash
# Serve the app locally
npm run serve
```

### Running Tests

```bash
# Run all tests
npm test

# Run specific test suites
npm run test:navigation
npm run test:post
npm run test:profile
npm run test:friend

# Run a single test case
npm run test:single -- --test="Should switch to profile view"

# Run in headless mode (for CI/CD)
npm run test:headless
```

### Using the Shell Script

```bash
# Make the script executable (if not already)
chmod +x run-tests.sh

# Run all tests
./run-tests.sh

# Run in headless mode
./run-tests.sh --headless

# Run a specific suite
./run-tests.sh --suite=navigation
```

## Troubleshooting

If you encounter issues with the test runner:

1. **404 Errors**: These are normal for some static assets and can be ignored
2. **Timeout Errors**: Try increasing the TEST_TIMEOUT value in test-runner.js
3. **Element Not Found**: Check that the data-testid attributes match between tests and HTML

## Additional Documentation

For more detailed information, see:
- [TESTING.md](./TESTING.md) - Detailed testing documentation
- [GitHub Actions Workflow](./.github/workflows/tests.yml) - CI/CD configuration
