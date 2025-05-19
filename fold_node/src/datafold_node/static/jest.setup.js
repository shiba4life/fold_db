// Import jest-dom matchers
require('@testing-library/jest-dom');

// Set up any global test configuration
window.ResizeObserver = jest.fn().mockImplementation(() => ({
    observe: jest.fn(),
    unobserve: jest.fn(),
    disconnect: jest.fn(),
}));