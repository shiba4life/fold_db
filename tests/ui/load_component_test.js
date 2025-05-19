const fs = require('fs');
const path = require('path');
const assert = require('assert');

class Element {
  constructor(tag) {
    this.tag = tag;
    this.innerHTML = '';
    this.id = '';
    this.children = [];
  }
  appendChild(c) {
    this.children.push(c);
    if (c.tag === 'script' && c.textContent) {
      Function(c.textContent)();
    }
  }
  removeChild(c) { this.children = this.children.filter(x => x !== c); }
  querySelectorAll(selector) {
    if (selector === 'script') {
      const regex = /<script[^>]*>([\s\S]*?)<\/script>/gi;
      const matches = [...this.innerHTML.matchAll(regex)];
      return matches.map(m => ({ textContent: m[1], src: '' }));
    }
    return [];
  }
}
class Document {
  constructor() {
    this.elements = { container: new Element('div') };
    this.body = new Element('body');
  }
  getElementById(id) { return this.elements[id]; }
  createElement(tag) { return new Element(tag); }
  dispatchEvent() {}
}

global.document = new Document();
global.window = { icons: { refresh: () => '[refresh]' } };

const utilsPath = path.join(__dirname, '..', '..', 'fold_node', 'src', 'datafold_node', 'static', 'js', 'utils.js');
const utilsCode = fs.readFileSync(utilsPath, 'utf8');

// Evaluate utils.js in this context
Function(utilsCode)();

// Mock fetch
global.fetch = async () => ({
  text: async () => '<div id="inner">Content</div><script>window.testVar=42;</script>'
});

(async () => {
  await window.utils.loadHtmlIntoContainer('dummy', 'container');
  assert.strictEqual(document.getElementById('container').innerHTML.includes('Content'), true);
  assert.strictEqual(window.testVar, 42);
  console.log('loadHtmlIntoContainer test passed');
})();
