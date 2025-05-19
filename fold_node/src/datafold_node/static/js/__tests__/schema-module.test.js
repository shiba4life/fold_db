require('@testing-library/jest-dom');

window.transformsModule = {
    loadTransforms: jest.fn()
};

window.utils = {
    isValidJSON: () => true,
    apiRequest: jest.fn().mockResolvedValue({}),
    displayResult: jest.fn()
};

document.body.innerHTML = '<textarea id="schemaInput"></textarea>';

const fs = require('fs');
const path = require('path');
const schemaPath = path.join(__dirname, '..', 'schema.js');
const schemaCode = fs.readFileSync(schemaPath, 'utf8');

// Evaluate schema.js in this context
Function(schemaCode)();

describe('schemaModule.loadSchema', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });

    test('calls transformsModule.loadTransforms after loading schema', async () => {
        document.getElementById('schemaInput').value = '{}';
        await window.schemaModule.loadSchema();
        expect(window.utils.apiRequest).toHaveBeenCalled();
        expect(window.transformsModule.loadTransforms).toHaveBeenCalled();
    });
});
