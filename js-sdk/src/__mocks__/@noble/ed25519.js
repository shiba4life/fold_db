// Mock implementation of @noble/ed25519 for Jest tests
let keyGenerationCounter = 0;

// Ensure we have proper validation of inputs
const validatePrivateKey = (privateKey) => {
  if (!privateKey || privateKey.length !== 32) {
    throw new Error('Invalid private key length');
  }
};

const validatePublicKey = (publicKey) => {
  if (!publicKey || publicKey.length !== 32) {
    throw new Error('Invalid public key length');
  }
};

const validateMessage = (message) => {
  if (!message || message.length === 0) {
    throw new Error('Invalid message');
  }
};

const validateSignature = (signature) => {
  if (!signature || signature.length !== 64) {
    throw new Error('Invalid signature length');
  }
};

const utils = {
  randomPrivateKey: jest.fn(() => {
    const key = new Uint8Array(32);
    // Generate more varied random key
    const entropy = Date.now() + Math.random() * 1000000 + keyGenerationCounter * 1000;
    for (let i = 0; i < 32; i++) {
      key[i] = Math.floor((Math.sin(entropy + i) * 10000 + Math.cos(entropy * 1.618 + i) * 10000) % 256);
      if (key[i] < 0) key[i] += 256;
    }
    keyGenerationCounter++;
    return key;
  })
};

const getPublicKeyAsync = jest.fn(async (privateKey) => {
  try {
    if (!privateKey) {
      throw new Error('Private key is null or undefined');
    }
    
    if (!(privateKey instanceof Uint8Array)) {
      throw new Error(`Private key must be Uint8Array, got ${typeof privateKey}`);
    }
    
    if (privateKey.length !== 32) {
      throw new Error(`Private key must be 32 bytes, got ${privateKey.length}`);
    }
    
    // Mock public key generation - create a unique public key based on private key
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = (privateKey[i] * 3 + 17 + keyGenerationCounter) % 256;
    }
    keyGenerationCounter++;
    return publicKey;
  } catch (error) {
    throw new Error(`getPublicKeyAsync failed: ${error.message || error.toString()}`);
  }
});

const getPublicKey = jest.fn((privateKey) => {
  try {
    validatePrivateKey(privateKey);
    
    // Synchronous version
    const publicKey = new Uint8Array(32);
    for (let i = 0; i < 32; i++) {
      publicKey[i] = (privateKey[i] * 3 + 17 + keyGenerationCounter) % 256;
    }
    keyGenerationCounter++;
    return publicKey;
  } catch (error) {
    throw new Error(`getPublicKey failed: ${error.message}`);
  }
});

const signAsync = jest.fn(async (message, privateKey) => {
  try {
    validateMessage(message);
    validatePrivateKey(privateKey);
    
    // Mock signature - create deterministic signature
    const signature = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      signature[i] = (message[0] + privateKey[0] + i) % 256;
    }
    return signature;
  } catch (error) {
    throw new Error(`signAsync failed: ${error.message}`);
  }
});

const sign = jest.fn((message, privateKey) => {
  try {
    validateMessage(message);
    validatePrivateKey(privateKey);
    
    // Synchronous version
    const signature = new Uint8Array(64);
    for (let i = 0; i < 64; i++) {
      signature[i] = (message[0] + privateKey[0] + i) % 256;
    }
    return signature;
  } catch (error) {
    throw new Error(`sign failed: ${error.message}`);
  }
});

const verifyAsync = jest.fn(async (signature, message, publicKey) => {
  try {
    validateSignature(signature);
    validateMessage(message);
    validatePublicKey(publicKey);
    
    // Mock verification - return true for valid format, false for all-zero signatures
    const isAllZeros = Array.from(signature).every(byte => byte === 0);
    return !isAllZeros;
  } catch (error) {
    // Return false for validation errors rather than throwing
    return false;
  }
});

const verify = jest.fn((signature, message, publicKey) => {
  try {
    validateSignature(signature);
    validateMessage(message);
    validatePublicKey(publicKey);
    
    // Synchronous version
    const isAllZeros = Array.from(signature).every(byte => byte === 0);
    return !isAllZeros;
  } catch (error) {
    return false;
  }
});

// CommonJS export for Jest compatibility
module.exports = {
  utils,
  getPublicKeyAsync,
  getPublicKey,
  signAsync,
  sign,
  verifyAsync,
  verify
};

// Also support ES module default export for compatibility
module.exports.default = module.exports;