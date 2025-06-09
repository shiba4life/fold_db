// Test setup for Ed25519 tests
// Mock window and location for browser environment tests
Object.defineProperty(globalThis, 'window', {
  value: {
    location: {
      protocol: 'https:',
      hostname: 'localhost'
    },
    isSecureContext: true
  },
  writable: true,
  configurable: true
});

// Mock navigator
Object.defineProperty(globalThis, 'navigator', {
  value: {
    userAgent: 'Mozilla/5.0 (Test Environment)'
  },
  writable: true,
  configurable: true
});

// Mock btoa and atob for base64 operations
if (typeof globalThis.btoa === 'undefined') {
  globalThis.btoa = (str: string) => {
    // Simple base64 encode for testing - manual implementation
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    let result = '';
    let i = 0;
    
    while (i < str.length) {
      const a = str.charCodeAt(i++);
      const b = i < str.length ? str.charCodeAt(i++) : 0;
      const c = i < str.length ? str.charCodeAt(i++) : 0;
      
      const bitmap = (a << 16) | (b << 8) | c;
      
      result += chars.charAt((bitmap >> 18) & 63);
      result += chars.charAt((bitmap >> 12) & 63);
      result += chars.charAt((bitmap >> 6) & 63);
      result += chars.charAt(bitmap & 63);
    }
    
    return result;
  };
}

if (typeof globalThis.atob === 'undefined') {
  globalThis.atob = (str: string) => {
    // Simple base64 decode for testing - manual implementation
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    let result = '';
    
    for (let i = 0; i < str.length; i += 4) {
      const encoded1 = chars.indexOf(str[i]);
      const encoded2 = chars.indexOf(str[i + 1]);
      const encoded3 = chars.indexOf(str[i + 2]);
      const encoded4 = chars.indexOf(str[i + 3]);
      
      const bitmap = (encoded1 << 18) | (encoded2 << 12) | (encoded3 << 6) | encoded4;
      
      result += String.fromCharCode((bitmap >> 16) & 255);
      if (encoded3 !== 64) result += String.fromCharCode((bitmap >> 8) & 255);
      if (encoded4 !== 64) result += String.fromCharCode(bitmap & 255);
    }
    
    return result;
  };
}