const { fireEvent } = require('@testing-library/dom');
require('@testing-library/jest-dom');

describe('Samples Tab Preview Modal', () => {
    let modal;
    beforeEach(() => {
        const html = require('../samples-tab.html');
        document.body.innerHTML = html;
        modal = document.getElementById('samplePreviewModal');
        const scriptContent = html.match(/<script>([\s\S]*?)<\/script>/)[1];
        eval(scriptContent);
    });

    test('close button hides the modal', () => {
        modal.style.display = 'block';
        const closeButton = modal.querySelector('.close-modal');
        fireEvent.click(closeButton);
        expect(modal.style.display).toBe('none');
    });

    test('clicking outside hides the modal', () => {
        modal.style.display = 'block';
        fireEvent.click(modal);
        expect(modal.style.display).toBe('none');
    });
});
