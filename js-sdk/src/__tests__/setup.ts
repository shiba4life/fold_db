// Test setup for Ed25519 tests
// Mock window and location for browser environment tests
// Mock indexedDB first so we can reference it in window
const mockIndexedDB = {
  open: () => {
    const request = mockIDBRequest();
    setTimeout(() => {
      request.result = mockIDBDatabase();
      if (request.onupgradeneeded) {
        request.onupgradeneeded({ target: request } as any);
      }
      if (request.onsuccess) {
        request.onsuccess();
      }
    }, 0);
    return request;
  },
  deleteDatabase: () => {
    const request = mockIDBRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  }
};

Object.defineProperty(globalThis, 'window', {
  value: {
    location: {
      protocol: 'https:',
      hostname: 'localhost'
    },
    isSecureContext: true,
    indexedDB: mockIndexedDB
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

// Mock TextEncoder and TextDecoder for browser environment tests
if (typeof globalThis.TextEncoder === 'undefined') {
  globalThis.TextEncoder = class TextEncoder {
    encoding = 'utf-8';
    encode(input: string): Uint8Array {
      return new Uint8Array(Buffer.from(input, 'utf8'));
    }
    encodeInto(input: string, destination: Uint8Array): { read: number; written: number } {
      const encoded = this.encode(input);
      const toCopy = Math.min(encoded.length, destination.length);
      destination.set(encoded.subarray(0, toCopy));
      return { read: input.length, written: toCopy };
    }
  } as any;
}

if (typeof globalThis.TextDecoder === 'undefined') {
  globalThis.TextDecoder = class TextDecoder {
    encoding = 'utf-8';
    fatal = false;
    ignoreBOM = false;
    constructor(label?: string) {
      this.encoding = label || 'utf-8';
    }
    decode(input?: Uint8Array): string {
      if (!input) return '';
      return Buffer.from(input).toString('utf8');
    }
  } as any;
}

// Mock IndexedDB for browser storage tests
const mockIDBRequest = () => ({
  result: null as any,
  error: null,
  onsuccess: null as any,
  onerror: null as any,
  onupgradeneeded: null as any
});

const mockIDBObjectStore = () => ({
  createIndex: () => {},
  put: () => {
    const request = mockIDBRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  },
  get: () => {
    const request = mockIDBRequest();
    request.result = {
      id: 'test-key',
      encryptedPrivateKey: new ArrayBuffer(64),
      publicKey: new Uint8Array(32).buffer,
      iv: new ArrayBuffer(12),
      salt: new ArrayBuffer(16),
      metadata: {
        name: 'test-key',
        description: '',
        created: new Date().toISOString(),
        lastAccessed: new Date().toISOString(),
        tags: []
      },
      version: 1
    };
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  },
  delete: () => {
    const request = mockIDBRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  },
  clear: () => {
    const request = mockIDBRequest();
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  },
  getAll: () => {
    const request = mockIDBRequest();
    request.result = [];
    setTimeout(() => request.onsuccess && request.onsuccess(), 0);
    return request;
  }
});

const mockIDBTransaction = () => ({
  objectStore: () => mockIDBObjectStore(),
  oncomplete: null as any,
  onerror: null as any,
  onabort: null as any
});

const mockIDBDatabase = () => ({
  objectStoreNames: {
    contains: () => false
  },
  createObjectStore: () => mockIDBObjectStore(),
  transaction: () => mockIDBTransaction(),
  close: () => {},
  version: 1
});

Object.defineProperty(globalThis, 'indexedDB', {
  value: mockIndexedDB,
  writable: true,
  configurable: true
});

// Mock WebCrypto API with proper implementations
const mockSubtle = {
  importKey: async (format: any, keyData: any, algorithm: any, extractable: any, keyUsages: any) => {
    return {
      type: 'secret',
      extractable,
      algorithm,
      usages: keyUsages
    };
  },
  deriveKey: async (algorithm: any, baseKey: any, derivedKeyAlgorithm: any, extractable: any, keyUsages: any) => {
    const keyLength = derivedKeyAlgorithm.length || 256;
    const key = new Uint8Array(keyLength / 8);
    
    // Generate more unique key material based on multiple inputs
    const infoHash = algorithm.info ? Array.from(algorithm.info as Uint8Array).reduce((acc: number, val: number) => acc + val, 0) : 0;
    const saltHash = algorithm.salt ? Array.from(algorithm.salt as Uint8Array).reduce((acc: number, val: number) => acc + val, 0) : 0;
    const iterationsHash = algorithm.iterations || 0;
    
    // Create more varied entropy
    for (let i = 0; i < key.length; i++) {
      key[i] = (i * 37 + infoHash * 7 + saltHash * 11 + iterationsHash * 3 + keyLength * 13) % 256;
    }
    
    return {
      type: 'secret',
      extractable,
      algorithm: derivedKeyAlgorithm,
      usages: keyUsages,
      _keyData: key
    };
  },
  encrypt: async (algorithm: any, key: any, data: any) => {
    const dataArray = new Uint8Array(data);
    const result = new Uint8Array(dataArray.length + 16); // Add 16 bytes for tag
    
    // Use key data for consistent encryption if available
    const keyXor = key._keyData ? key._keyData[0] : 42;
    
    // Deterministic encryption based on key and data
    for (let i = 0; i < dataArray.length; i++) {
      result[i] = dataArray[i] ^ ((i + keyXor + 42) % 256);
    }
    
    // Add deterministic authentication tag
    for (let i = dataArray.length; i < result.length; i++) {
      result[i] = (i * 37 + keyXor) % 256;
    }
    
    return result.buffer;
  },
  decrypt: async (algorithm: any, key: any, data: any) => {
    const dataArray = new Uint8Array(data);
    if (dataArray.length < 16) {
      throw new Error('Invalid encrypted data');
    }
    
    const ciphertext = dataArray.slice(0, -16); // Remove tag
    const tag = dataArray.slice(-16); // Get tag
    const result = new Uint8Array(ciphertext.length);
    
    // Use key data for consistent decryption if available
    const keyXor = key._keyData ? key._keyData[0] : 42;
    
    // Verify tag (basic check)
    for (let i = 0; i < tag.length; i++) {
      const expectedTag = ((ciphertext.length + i) * 37 + keyXor) % 256;
      if (tag[i] !== expectedTag) {
        throw new Error('Authentication failed');
      }
    }
    
    // Reverse the encryption - XOR with same pattern
    for (let i = 0; i < ciphertext.length; i++) {
      result[i] = ciphertext[i] ^ ((i + keyXor + 42) % 256);
    }
    
    return result.buffer;
  },
  generateKey: async (algorithm: any, extractable: any, keyUsages: any) => {
    if (algorithm.name === 'AES-GCM') {
      const keyData = new Uint8Array(algorithm.length / 8);
      for (let i = 0; i < keyData.length; i++) {
        keyData[i] = Math.floor(Math.random() * 256);
      }
      
      return {
        type: 'secret',
        extractable,
        algorithm,
        usages: keyUsages,
        _keyData: keyData
      };
    }
    
    return {
      publicKey: {
        type: 'public',
        extractable: true,
        algorithm,
        usages: keyUsages.filter((usage: string) => ['verify'].includes(usage))
      },
      privateKey: {
        type: 'private',
        extractable,
        algorithm,
        usages: keyUsages.filter((usage: string) => ['sign'].includes(usage))
      }
    };
  },
  digest: async (algorithm: any, data: any) => {
    const input = new Uint8Array(data);
    const hashSize = algorithm === 'SHA-256' ? 32 : 20;
    const result = new Uint8Array(hashSize);
    
    // Simple hash simulation
    for (let i = 0; i < hashSize; i++) {
      let sum = 0;
      for (let j = 0; j < input.length; j++) {
        sum += input[j] * (i + j + 1);
      }
      result[i] = sum % 256;
    }
    
    return result.buffer;
  },
  deriveBits: async (algorithm: any, baseKey: any, length: any) => {
    const result = new Uint8Array(length / 8);
    
    // Generate more unique deterministic bits
    const infoHash = algorithm.info ? Array.from(algorithm.info as Uint8Array).reduce((acc: number, val: number) => acc + val, 0) : 0;
    const saltHash = algorithm.salt ? Array.from(algorithm.salt as Uint8Array).reduce((acc: number, val: number) => acc + val, 0) : 0;
    
    for (let i = 0; i < result.length; i++) {
      result[i] = (i * 41 + length * 17 + infoHash * 23 + saltHash * 19) % 256;
    }
    
    return result.buffer;
  },
  exportKey: async (format: any, key: any) => {
    if (format === 'raw' && key._keyData) {
      return key._keyData.buffer;
    }
    
    // Return mock key data with proper length based on algorithm
    const keyLength = key.algorithm?.length ? key.algorithm.length / 8 : 32;
    const mockKeyData = new Uint8Array(keyLength);
    for (let i = 0; i < keyLength; i++) {
      mockKeyData[i] = (i + 42) % 256;
    }
    return mockKeyData.buffer;
  }
};

Object.defineProperty(globalThis, 'crypto', {
  value: {
    subtle: mockSubtle,
    getRandomValues: (array: Uint8Array) => {
      // Fallback to Math.random with better entropy
      const entropy = Date.now() + Math.random() * 1000000 + performance.now();
      for (let i = 0; i < array.length; i++) {
        // Use multiple entropy sources for better randomness
        const x = Math.sin(entropy + i * 1.618033988749895) * 10000; // Golden ratio
        const y = Math.cos(entropy + i * 2.718281828459045) * 10000; // Euler's number
        array[i] = Math.floor(((x - Math.floor(x)) + (y - Math.floor(y))) * 128) % 256;
      }
      return array;
    }
  },
  writable: true,
  configurable: true
});

// Mock Response constructor for Node.js environment
if (typeof globalThis.Response === 'undefined') {
  globalThis.Response = class Response {
    public readonly body: ReadableStream | null = null;
    public readonly bodyUsed: boolean = false;
    public readonly headers: any;
    public readonly ok: boolean;
    public readonly redirected: boolean = false;
    public readonly status: number;
    public readonly statusText: string;
    public readonly type: string = 'basic';
    public readonly url: string = '';

    constructor(body?: BodyInit | null, init?: ResponseInit) {
      this.status = init?.status || 200;
      this.statusText = init?.statusText || 'OK';
      this.ok = this.status >= 200 && this.status < 300;
      this.headers = new Map(Object.entries(init?.headers || {}));
      
      if (typeof body === 'string') {
        this._bodyText = body;
      }
    }

    private _bodyText = '';

    async text(): Promise<string> {
      return this._bodyText;
    }

    async json(): Promise<any> {
      return JSON.parse(this._bodyText);
    }

    async arrayBuffer(): Promise<ArrayBuffer> {
      const encoder = new TextEncoder();
      return encoder.encode(this._bodyText).buffer;
    }

    async blob(): Promise<Blob> {
      throw new Error('Blob not implemented in test environment');
    }

    async formData(): Promise<FormData> {
      throw new Error('FormData not implemented in test environment');
    }

    clone(): Response {
      return new Response(this._bodyText, {
        status: this.status,
        statusText: this.statusText,
        headers: this.headers
      });
    }
  } as any;
}

// Mock Headers constructor
if (typeof globalThis.Headers === 'undefined') {
  globalThis.Headers = class Headers {
    private _headers = new Map<string, string>();

    constructor(init?: HeadersInit) {
      if (init) {
        if (Array.isArray(init)) {
          init.forEach(([key, value]) => this.set(key, value));
        } else if (init instanceof Headers) {
          init.forEach((value, key) => this.set(key, value));
        } else {
          Object.entries(init).forEach(([key, value]) => this.set(key, value));
        }
      }
    }

    append(name: string, value: string): void {
      const existing = this._headers.get(name.toLowerCase());
      this._headers.set(name.toLowerCase(), existing ? `${existing}, ${value}` : value);
    }

    delete(name: string): void {
      this._headers.delete(name.toLowerCase());
    }

    get(name: string): string | null {
      return this._headers.get(name.toLowerCase()) || null;
    }

    has(name: string): boolean {
      return this._headers.has(name.toLowerCase());
    }

    set(name: string, value: string): void {
      this._headers.set(name.toLowerCase(), value);
    }

    forEach(callback: (value: string, key: string, parent: Headers) => void): void {
      this._headers.forEach((value, key) => callback(value, key, this));
    }

    *[Symbol.iterator](): IterableIterator<[string, string]> {
      for (const [key, value] of this._headers) {
        yield [key, value];
      }
    }

    entries(): IterableIterator<[string, string]> {
      return this[Symbol.iterator]();
    }

    keys(): IterableIterator<string> {
      return this._headers.keys();
    }

    values(): IterableIterator<string> {
      return this._headers.values();
    }
  } as any;
}

// Import and setup server mock
import { setupServerMock } from './__mocks__/server-mock.js';

// Setup server mock for all tests
setupServerMock();
