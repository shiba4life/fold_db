// FoldDB Query Signer provider initialization
console.log('FoldDB Query Signer provider script loaded');

// Initialize the provider object
window.foldDBProvider = {
  isConnected: true
};

// Define custom methods that will communicate with the content script
Object.defineProperties(window.foldDBProvider, {
  getPublicKey: {
    value: () => {
      return new Promise((resolve, reject) => {
        try {
          // Create a custom event with properly structured detail object
          const event = new CustomEvent('folddb:getPublicKey', {
            detail: { resolve, reject }
          });
          window.dispatchEvent(event);
        } catch (error) {
          console.error('Error dispatching getPublicKey event:', error);
          // Fallback to mock data
          resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
        }
      });
    }
  },
  signRequest: {
    value: (payload) => {
      return new Promise((resolve, reject) => {
        try {
          // Create a custom event with properly structured detail object
          const event = new CustomEvent('folddb:signRequest', {
            detail: { payload, resolve, reject }
          });
          window.dispatchEvent(event);
        } catch (error) {
          console.error('Error dispatching signRequest event:', error);
          // Fallback to mock data
          resolve({
            timestamp: Date.now(),
            payload,
            signature: 'MOCK_SIGNATURE_FOR_TESTING'
          });
        }
      });
    }
  }
});

// Also create the window.foldDB object directly for immediate access
// This ensures the app can detect it right away without waiting for inject.js
window.foldDB = {
  isConnected: true,
  eventListeners: {},
  getPublicKey: function() {
    return new Promise((resolve, reject) => {
      try {
        // Create a custom event with properly structured detail object
        const event = new CustomEvent('folddb:getPublicKey', {
          detail: { resolve, reject }
        });
        window.dispatchEvent(event);
      } catch (error) {
        console.error('Error dispatching getPublicKey event:', error);
        // Fallback to mock data
        resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
      }
    });
  },
  signRequest: function(payload) {
    return new Promise((resolve, reject) => {
      try {
        // Create a custom event with properly structured detail object
        const event = new CustomEvent('folddb:signRequest', {
          detail: { payload, resolve, reject }
        });
        window.dispatchEvent(event);
      } catch (error) {
        console.error('Error dispatching signRequest event:', error);
        // Fallback to mock data
        resolve({
          timestamp: Date.now(),
          payload,
          signature: 'MOCK_SIGNATURE_FOR_TESTING'
        });
      }
    });
  },
  on: function(eventName, callback) {
    if (!this.eventListeners[eventName]) {
      this.eventListeners[eventName] = [];
    }
    
    this.eventListeners[eventName].push(callback);
    
    // If this is a 'connect' event and we're already connected, trigger it immediately
    if (eventName === 'connect' && this.isConnected) {
      setTimeout(() => callback({ isConnected: true }), 0);
    }
    
    return this;
  },
  off: function(eventName, callback) {
    if (!this.eventListeners[eventName]) {
      return this;
    }
    
    if (!callback) {
      delete this.eventListeners[eventName];
    } else {
      this.eventListeners[eventName] = this.eventListeners[eventName].filter(
        listener => listener !== callback
      );
    }
    
    return this;
  },
  dispatchEvent: function(eventName, data) {
    if (!this.eventListeners[eventName]) {
      return;
    }
    
    for (const callback of this.eventListeners[eventName]) {
      try {
        callback(data);
      } catch (error) {
        console.error(`Error in ${eventName} event handler:`, error);
      }
    }
  }
};

// Immediately dispatch a connect event
setTimeout(() => {
  if (window.foldDB && window.foldDB.dispatchEvent) {
    try {
      window.foldDB.dispatchEvent('connect', { isConnected: true });
    } catch (error) {
      console.error('Error dispatching connect event:', error);
    }
  }
}, 50);

console.log('FoldDB Query Signer provider initialized');
