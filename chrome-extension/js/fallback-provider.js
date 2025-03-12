// Fallback provider
window.foldDBProvider = {
  isConnected: true,
  getPublicKey: () => Promise.resolve('MOCK_PUBLIC_KEY_FOR_TESTING'),
  signRequest: (payload) => Promise.resolve({
    timestamp: Date.now(),
    payload,
    signature: 'MOCK_SIGNATURE_FOR_TESTING'
  })
};

// Also create the window.foldDB object
window.foldDB = {
  isConnected: true,
  eventListeners: {},
  getPublicKey: function() {
    return Promise.resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
  },
  signRequest: function(payload) {
    return Promise.resolve({
      timestamp: Date.now(),
      payload,
      signature: 'MOCK_SIGNATURE_FOR_TESTING'
    });
  },
  on: function(eventName, callback) {
    if (!this.eventListeners[eventName]) {
      this.eventListeners[eventName] = [];
    }
    this.eventListeners[eventName].push(callback);
    if (eventName === 'connect') {
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
      callback(data);
    }
  }
};

// Handle custom events directly in the page
window.addEventListener('folddb:getPublicKey', function(event) {
  if (event && event.detail && typeof event.detail.resolve === 'function') {
    setTimeout(() => {
      event.detail.resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
    }, 50);
  } else {
    console.error('Invalid event detail in folddb:getPublicKey event', event);
  }
});

window.addEventListener('folddb:signRequest', function(event) {
  if (event && event.detail && typeof event.detail.resolve === 'function') {
    const payload = event.detail.payload || {};
    setTimeout(() => {
      event.detail.resolve({
        timestamp: Date.now(),
        payload: payload,
        signature: 'MOCK_SIGNATURE_FOR_TESTING'
      });
    }, 50);
  } else {
    console.error('Invalid event detail in folddb:signRequest event', event);
  }
});

// Dispatch a connect event
setTimeout(() => {
  if (window.foldDB && window.foldDB.dispatchEvent) {
    window.foldDB.dispatchEvent('connect', { isConnected: true });
  }
  // Dispatch an event to notify that the provider is available
  window.dispatchEvent(new Event('folddb:providerReady'));
  console.log('FoldDB fallback provider initialized');
}, 50);
