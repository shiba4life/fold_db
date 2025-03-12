// FoldDB Query Signer inject script
// This script is injected into web pages and provides a direct interface for web applications

// Define the FoldDB provider class
class FoldDBProvider {
  constructor() {
    this.isConnected = true;
    this.eventListeners = {};
    
    // Dispatch an event to notify that the provider is ready
    setTimeout(() => {
      this.dispatchEvent('connect', { isConnected: true });
    }, 0);
  }
  
  // Get the public key
  async getPublicKey() {
    try {
      return await window.foldDBProvider.getPublicKey();
    } catch (error) {
      console.error('Error getting public key:', error);
      throw error;
    }
  }
  
  // Sign a request
  async signRequest(payload) {
    try {
      return await window.foldDBProvider.signRequest(payload);
    } catch (error) {
      console.error('Error signing request:', error);
      throw error;
    }
  }
  
  // Add an event listener
  on(eventName, callback) {
    if (!this.eventListeners[eventName]) {
      this.eventListeners[eventName] = [];
    }
    
    this.eventListeners[eventName].push(callback);
    
    // If this is a 'connect' event and we're already connected, trigger it immediately
    if (eventName === 'connect' && this.isConnected) {
      callback({ isConnected: true });
    }
    
    return this;
  }
  
  // Remove an event listener
  off(eventName, callback) {
    if (!this.eventListeners[eventName]) {
      return this;
    }
    
    if (!callback) {
      // Remove all listeners for this event
      delete this.eventListeners[eventName];
    } else {
      // Remove the specific listener
      this.eventListeners[eventName] = this.eventListeners[eventName].filter(
        listener => listener !== callback
      );
    }
    
    return this;
  }
  
  // Dispatch an event
  dispatchEvent(eventName, data) {
    if (!this.eventListeners[eventName]) {
      return;
    }
    
    for (const callback of this.eventListeners[eventName]) {
      callback(data);
    }
  }
}

// Wait for the foldDBProvider to be injected by the content script
window.addEventListener('folddb:providerReady', () => {
  // Check if window.foldDB already exists (created by content.js)
  if (!window.foldDB) {
    // Create the FoldDB provider instance if it doesn't exist
    window.foldDB = new FoldDBProvider();
  } else {
    // Enhance the existing foldDB object with any missing functionality
    if (!window.foldDB.eventListeners) {
      window.foldDB.eventListeners = {};
    }
    
    // Add the dispatchEvent method if it doesn't exist
    if (!window.foldDB.dispatchEvent) {
      window.foldDB.dispatchEvent = function(eventName, data) {
        if (!this.eventListeners || !this.eventListeners[eventName]) {
          return;
        }
        
        for (const callback of this.eventListeners[eventName]) {
          callback(data);
        }
      };
    }
  }
  
  // Dispatch an event to notify that the provider is available
  const event = new CustomEvent('folddb:ready');
  window.dispatchEvent(event);
  
  console.log('FoldDB provider is ready');
});

// Notify the page that the inject script has loaded
console.log('FoldDB Query Signer inject script loaded');
