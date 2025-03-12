// Content script for FoldDB Query Signer
console.log('FoldDB Query Signer content script loaded');

// Keep track of pending requests
const pendingRequests = new Map();

// Create a unique ID for this content script instance
const contentScriptId = Date.now().toString(36) + Math.random().toString(36).substring(2);

// Notify the background script that this tab is connected
chrome.runtime.sendMessage({ action: 'tabConnected' });

// Listen for messages from the background script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'requestSigned') {
    // Find the pending request
    const pendingRequest = pendingRequests.get(message.requestId);
    if (pendingRequest) {
      // Resolve the promise with the signature
      pendingRequest.resolve(message.signature);
      
      // Remove the request from the pending list
      pendingRequests.delete(message.requestId);
    }
  } else if (message.action === 'requestRejected') {
    // Find the pending request
    const pendingRequest = pendingRequests.get(message.requestId);
    if (pendingRequest) {
      // Reject the promise
      pendingRequest.reject(new Error('Request rejected by user'));
      
      // Remove the request from the pending list
      pendingRequests.delete(message.requestId);
    }
  }
  
  // Return true to indicate async response
  return true;
});

// Function to request a signature from the background script
async function requestSignature(request) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      { action: 'signRequest', request },
      (response) => {
        if (response && response.success) {
          // Store the promise callbacks
          pendingRequests.set(response.requestId, { resolve, reject });
        } else {
          reject(new Error('Failed to request signature'));
        }
      }
    );
  });
}

// Function to get the public key from the background script
async function getPublicKey() {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      { action: 'getPublicKey' },
      (response) => {
        if (response && response.publicKey) {
          resolve(response.publicKey);
        } else {
          reject(new Error('No public key available'));
        }
      }
    );
  });
}

// Create the provider object to inject into the page
const foldDBProvider = {
  isConnected: true,
  
  // Get the public key
  getPublicKey: async () => {
    try {
      return await getPublicKey();
    } catch (error) {
      console.error('Error getting public key:', error);
      throw error;
    }
  },
  
  // Sign a request
  signRequest: async (payload) => {
    try {
      // Create the request object
      const request = {
        timestamp: Date.now(),
        payload
      };
      
      // Request a signature from the background script
      const signature = await requestSignature(request);
      
      // Return the signed request
      return {
        timestamp: request.timestamp,
        payload,
        signature
      };
    } catch (error) {
      console.error('Error signing request:', error);
      throw error;
    }
  }
};

// Inject the provider object into the page
function injectProvider() {
  // First, inject the provider object
  const providerScript = document.createElement('script');
  providerScript.textContent = `
    // FoldDB Query Signer provider
    window.foldDBProvider = ${JSON.stringify({
      isConnected: true
    })};
    
    // Define custom methods that will communicate with the content script
    Object.defineProperties(window.foldDBProvider, {
      getPublicKey: {
        value: () => {
          return new Promise((resolve, reject) => {
            window.dispatchEvent(new CustomEvent('folddb:getPublicKey', {
              detail: { resolve, reject }
            }));
          });
        }
      },
      signRequest: {
        value: (payload) => {
          return new Promise((resolve, reject) => {
            window.dispatchEvent(new CustomEvent('folddb:signRequest', {
              detail: { payload, resolve, reject }
            }));
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
          window.dispatchEvent(new CustomEvent('folddb:getPublicKey', {
            detail: { resolve, reject }
          }));
        });
      },
      signRequest: function(payload) {
        return new Promise((resolve, reject) => {
          window.dispatchEvent(new CustomEvent('folddb:signRequest', {
            detail: { payload, resolve, reject }
          }));
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
          callback(data);
        }
      }
    };
    
    // Immediately dispatch a connect event
    setTimeout(() => {
      if (window.foldDB && window.foldDB.dispatchEvent) {
        window.foldDB.dispatchEvent('connect', { isConnected: true });
      }
    }, 50);
    
    console.log('FoldDB Query Signer provider injected');
  `;
  
  // Append the provider script to the document
  (document.head || document.documentElement).appendChild(providerScript);
  
  // Remove the script element after injection
  providerScript.remove();
  
  // Now, inject the inject.js script
  const injectScript = document.createElement('script');
  injectScript.src = chrome.runtime.getURL('js/inject.js');
  injectScript.onload = function() {
    // Dispatch an event to notify the page that the provider is available
    window.dispatchEvent(new Event('folddb:providerReady'));
    console.log('FoldDB inject script loaded');
    this.remove();
  };
  
  // Append the inject script to the document
  (document.head || document.documentElement).appendChild(injectScript);
}

// Listen for events from the injected provider
window.addEventListener('folddb:getPublicKey', async (event) => {
  try {
    const publicKey = await getPublicKey();
    event.detail.resolve(publicKey);
  } catch (error) {
    event.detail.reject(error);
  }
});

window.addEventListener('folddb:signRequest', async (event) => {
  try {
    // Create the request object
    const request = {
      timestamp: Date.now(),
      payload: event.detail.payload
    };
    
    // Request a signature from the background script
    const signature = await requestSignature(request);
    
    // Resolve the promise with the signed request
    event.detail.resolve({
      timestamp: request.timestamp,
      payload: event.detail.payload,
      signature
    });
  } catch (error) {
    event.detail.reject(error);
  }
});

// Function to check if the provider is working
function checkProvider() {
  try {
    // Create a script to check if window.foldDB exists and is working
    const checkScript = document.createElement('script');
    checkScript.textContent = `
      console.log('Checking FoldDB provider status...');
      if (window.foldDB) {
        console.log('FoldDB provider exists:', window.foldDB);
        
        // Dispatch a connect event to ensure the app knows we're connected
        if (window.foldDB.dispatchEvent) {
          console.log('Dispatching connect event');
          window.foldDB.dispatchEvent('connect', { isConnected: true });
        } else {
          console.log('dispatchEvent method not found on foldDB');
        }
      } else {
        console.log('FoldDB provider does not exist');
      }
    `;
    
    // Append the script to the document
    (document.head || document.documentElement).appendChild(checkScript);
    
    // Remove the script element after execution
    checkScript.remove();
  } catch (error) {
    console.error('Error checking provider:', error);
  }
}

// Inject the provider when the content script loads
injectProvider();

// Check the provider status after a short delay
setTimeout(checkProvider, 500);

// And check again after a longer delay in case of timing issues
setTimeout(checkProvider, 2000);

// Clean up when the content script is unloaded
window.addEventListener('beforeunload', () => {
  // Notify the background script that this tab is disconnected
  chrome.runtime.sendMessage({ action: 'tabDisconnected' });
});
