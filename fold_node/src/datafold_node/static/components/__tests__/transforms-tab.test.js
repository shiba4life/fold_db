const { fireEvent } = require('@testing-library/dom');
require('@testing-library/jest-dom');

// Mock modules
window.icons = {
    refresh: () => '<svg>refresh</svg>'
};

window.transformsModule = {
    loadTransforms: jest.fn()
};

// Mock utils to satisfy script though not used
window.utils = {
    displayResult: jest.fn()
};

describe('Transforms Tab Component', () => {
    let container;

    beforeEach(() => {
        const html = require('../transforms-tab.html');
        document.body.innerHTML = html;
        container = document.getElementById('transformsTab');

        const scriptContent = html.match(/<script>([\s\S]*?)<\/script>/)[1];
        eval(scriptContent);
        jest.clearAllMocks();
        document.dispatchEvent(new Event('DOMContentLoaded'));
    });

    test('initializes and loads transforms on DOMContentLoaded', () => {
        expect(window.transformsModule.loadTransforms).toHaveBeenCalledTimes(1);
        const refreshIcon = document.getElementById('refreshTransformsIcon');
        expect(refreshIcon.innerHTML).toBe('<svg>refresh</svg>');
    });

    test('refresh button triggers loadTransforms', () => {
        const btn = container.querySelector('#refreshTransformsBtn');
        fireEvent.click(btn);
        expect(window.transformsModule.loadTransforms).toHaveBeenCalled();
    });
});
