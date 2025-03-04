# Social App Testing Guide

This document describes the testing infrastructure for the Social App.

## Overview

The testing system is designed to efficiently test the Social App without consuming excessive context window space. It consists of:

1. **Test Hooks in the App**: Data-testid attributes added to key UI elements
2. **Test Harness**: A standalone HTML page that loads the app in an iframe and runs tests
3. **Test Runner**: A Node.js script that automates test execution using Puppeteer
4. **Test Suites**: Focused test collections for different app features

## Test Suites

The tests are organized into focused suites:

1. **Navigation Tests**: Tests for view switching (Feed, Profile, Friends)
2. **Post Tests**: Tests for post creation and display
3. **Profile Tests**: Tests for profile information display
4. **Friend Tests**: Tests for friends list and suggestions

## Running Tests

### Manual Testing with the Test Harness

1. Open `test-harness.html` in a browser
2. Use the buttons at the top to run specific test suites or all tests
3. View test results in the status panel

### Automated Testing with the Test Runner

Prerequisites:
- Node.js 14 or higher
- npm or yarn

Setup:
```bash
# Install dependencies
npm install
```

Run tests:
```bash
# Run all tests
npm test

# Run in headless mode (for CI/CD)
npm run test:headless

# Run specific test suites
npm run test:navigation
npm run test:post
npm run test:profile
npm run test:friend

# Run a single test case
npm run test:single -- --test="Should switch to profile view"
```

Test results are saved to the `test-results` directory, including:
- HTML report with detailed test results
- Screenshot of the test harness after test completion

## Single Test Runner

For debugging specific issues, you can use the single test runner to run just one test case:

```bash
# Run a specific test case
npm run test:single -- --test="Should switch to profile view"

# Run in headless mode
npm run test:single -- --test="Should switch to profile view" --headless
```

Available test cases:

**Navigation Tests**
- "Initial view should be feed"
- "Should switch to profile view"
- "Should switch to friends view"
- "Should switch back to feed view"

**Post Tests**
- "Should display posts in feed"
- "Should create a new post"

**Profile Tests**
- "Should display profile information"
- "Should display user posts in profile"

**Friend Tests**
- "Should display friends list"
- "Should display friend suggestions"

## How It Works

1. The test harness loads the app in an iframe
2. It initializes the app with test-specific mock data
3. Tests interact with the app through the iframe using DOM selectors
4. Each test performs actions and assertions on the app's state
5. Test results are displayed in real-time

## Adding New Tests

To add new tests:

1. Add data-testid attributes to new UI elements in `index.html`
2. Add test cases to the appropriate test suite in `test-harness.html`
3. Update test data if needed

Example test case:
```javascript
await runner.runTest('Should do something', async () => {
    await runner.clickElement('[data-testid="some-button"]');
    await runner.assertElementVisible('[data-testid="some-element"]');
    await runner.assertElementText('[data-testid="some-text"]', 'Expected Text');
});
```

## Test Utilities

The TestRunner class provides several helper methods:

- `assertElementVisible(selector)`: Checks if an element is visible
- `assertElementHidden(selector)`: Checks if an element is hidden
- `assertElementExists(selector)`: Checks if an element exists
- `assertElementText(selector, expectedText)`: Checks element text
- `clickElement(selector)`: Clicks an element
- `typeInElement(selector, text)`: Types text into an input element

## Benefits of This Approach

1. **Efficiency**: Tests run in isolation without consuming API context
2. **Focus**: Each test suite targets specific functionality
3. **Speed**: Tests run quickly with minimal setup
4. **Reliability**: Test hooks make selectors more reliable
5. **Visibility**: Clear test results with pass/fail indicators
6. **Automation**: Can be integrated into CI/CD pipelines
