// Content script for FoldDB Query Signer
console.log('FoldDB Query Signer content script loaded');

// Keep track of pending requests
const pendingRequests = new Map();

// Create a unique ID for this content script instance
const contentScriptId = Date.now().toString(36) + Math.random().toString(36).substring(2);

// Check if we're in a Chrome extension context
const isExtensionContext = typeof chrome !== 'undefined' && chrome.runtime && chrome.runtime.sendMessage;

// Flag to track if we've already shown the warning
let extensionContextWarningShown = false;

// Helper function to safely check extension context
function checkExtensionContext() {
  const isValid = typeof chrome !== 'undefined' && chrome.runtime && chrome.runtime.sendMessage;
  if (!isValid && !extensionContextWarningShown) {
    console.warn('Not running in extension context, chrome.runtime.sendMessage is not available');
    extensionContextWarningShown = true;
  }
  return isValid;
}

// Notify the background script that this tab is connected
if (checkExtensionContext()) {
  try {
    chrome.runtime.sendMessage({ action: 'tabConnected' }, (response) => {
      if (chrome.runtime.lastError) {
        console.warn('Error notifying background script about tab connection:', chrome.runtime.lastError);
      }
    });
  } catch (error) {
    console.warn('Error sending tabConnected message:', error);
  }
}

// Listen for messages from the background script
if (checkExtensionContext()) {
  try {
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
  } catch (error) {
    console.warn('Error setting up message listener:', error);
  }
}

// Function to request a signature from the background script
async function requestSignature(request) {
  return new Promise((resolve, reject) => {
    if (!checkExtensionContext()) {
      // If not in extension context, return a mock signature
      console.warn('Not running in extension context, returning mock signature');
      setTimeout(() => {
        resolve('MOCK_SIGNATURE_FOR_TESTING');
      }, 500);
      return;
    }
    
    try {
      const timeout = setTimeout(() => {
        reject(new Error('Request timed out'));
        console.warn('Sign request timed out');
      }, 5000);
      
      chrome.runtime.sendMessage(
        { action: 'signRequest', request },
        (response) => {
          clearTimeout(timeout);
          if (chrome.runtime.lastError) {
            console.warn('Error requesting signature:', chrome.runtime.lastError);
            reject(new Error(chrome.runtime.lastError.message));
          } else if (response && response.success) {
            // Store the promise callbacks
            pendingRequests.set(response.requestId, { resolve, reject });
          } else {
            reject(new Error('Failed to request signature'));
          }
        }
      );
    } catch (error) {
      console.error('Error sending signRequest message:', error);
      // Return a mock signature as fallback
      setTimeout(() => {
        resolve('MOCK_SIGNATURE_FOR_TESTING');
      }, 500);
    }
  });
}

// Function to get the public key from the background script
async function getPublicKey() {
  return new Promise((resolve, reject) => {
    if (!checkExtensionContext()) {
      // If not in extension context, return a mock public key
      console.warn('Not running in extension context, returning mock public key');
      setTimeout(() => {
        resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
      }, 500);
      return;
    }
    
    try {
      const timeout = setTimeout(() => {
        reject(new Error('Request timed out'));
        console.warn('Get public key request timed out');
      }, 5000);
      
      chrome.runtime.sendMessage(
        { action: 'getPublicKey' },
        (response) => {
          clearTimeout(timeout);
          if (chrome.runtime.lastError) {
            console.warn('Error getting public key:', chrome.runtime.lastError);
            reject(new Error(chrome.runtime.lastError.message));
          } else if (response && response.publicKey) {
            resolve(response.publicKey);
          } else {
            reject(new Error('No public key available'));
          }
        }
      );
    } catch (error) {
      console.error('Error sending getPublicKey message:', error);
      // Return a mock public key as fallback
      setTimeout(() => {
        resolve('MOCK_PUBLIC_KEY_FOR_TESTING');
      }, 500);
    }
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
      return 'MOCK_PUBLIC_KEY_FOR_TESTING';
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
      // Return a mock signed request as fallback
      return {
        timestamp: Date.now(),
        payload,
        signature: 'MOCK_SIGNATURE_FOR_TESTING'
      };
    }
  }
};

// Inject the provider object into the page
function injectProvider() {
  try {
    // First try to inject the fallback provider
    const fallbackScript = document.createElement('script');
    
    if (checkExtensionContext()) {
      try {
        fallbackScript.src = chrome.runtime.getURL('js/fallback-provider.js');
      } catch (error) {
        console.error('Error getting URL for fallback-provider.js:', error);
        return;
      }
    } else {
      // If not in extension context, try to use a relative path
      fallbackScript.src = 'chrome-extension/js/fallback-provider.js';
      console.warn('Not running in extension context, using relative path for fallback provider');
    }
    
    fallbackScript.onload = function() {
      console.log('Fallback provider script loaded');
      this.remove();
    };
    
    fallbackScript.onerror = function() {
      console.error('Failed to load fallback provider script');
      this.remove();
    };
    
    (document.head || document.documentElement).appendChild(fallbackScript);
    
    // If we're in extension context, also try to inject the extension scripts
    if (checkExtensionContext()) {
      try {
        // First, inject the provider.js script
        const providerScript = document.createElement('script');
        providerScript.src = chrome.runtime.getURL('js/provider.js');
        
        providerScript.onload = function() {
          console.log('Provider script loaded');
          
          // Wait a short time to ensure the provider object is fully initialized
          setTimeout(() => {
            try {
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
            } catch (error) {
              console.error('Error injecting inject.js script:', error);
            }
          }, 100); // Small delay to ensure provider is ready
          
          this.remove();
        };
        
        // Append the provider script to the document
        (document.head || document.documentElement).appendChild(providerScript);
      } catch (error) {
        console.error('Error injecting extension scripts:', error);
      }
    }
  } catch (error) {
    console.error('Error injecting provider:', error);
  }
}

// Helper function to safely check if an event has valid detail with resolve/reject functions
function hasValidEventDetail(event) {
  return event && 
         event.detail && 
         typeof event.detail === 'object' && 
         typeof event.detail.resolve === 'function';
}

// Listen for events from the injected provider
window.addEventListener('folddb:getPublicKey', async (event) => {
  try {
    // Check if the event detail is valid
    if (!hasValidEventDetail(event)) {
      console.error('Invalid event detail in folddb:getPublicKey event', event);
      return;
    }
    
    const publicKey = await getPublicKey();
    event.detail.resolve(publicKey);
  } catch (error) {
    if (hasValidEventDetail(event) && typeof event.detail.reject === 'function') {
      event.detail.reject(error);
    } else {
      console.error('Error in folddb:getPublicKey event and cannot reject:', error);
    }
  }
});

window.addEventListener('folddb:signRequest', async (event) => {
  try {
    // Check if the event detail is valid
    if (!hasValidEventDetail(event)) {
      console.error('Invalid event detail in folddb:signRequest event', event);
      return;
    }
    
    // Create the request object
    const request = {
      timestamp: Date.now(),
      payload: event.detail.payload || {}
    };
    
    // Request a signature from the background script
    const signature = await requestSignature(request);
    
    // Resolve the promise with the signed request
    event.detail.resolve({
      timestamp: request.timestamp,
      payload: event.detail.payload || {},
      signature
    });
  } catch (error) {
    if (hasValidEventDetail(event) && typeof event.detail.reject === 'function') {
      event.detail.reject(error);
    } else {
      console.error('Error in folddb:signRequest event and cannot reject:', error);
    }
  }
});

// Function to check if the provider is working
function checkProvider() {
  try {
    if (!checkExtensionContext()) {
      console.warn('Not running in extension context, cannot check provider');
      return;
    }
    
    // Create a script to check if window.foldDB exists and is working
    try {
      const checkScript = document.createElement('script');
      checkScript.src = chrome.runtime.getURL('js/check-provider.js');
      
      // Append the script to the document
      (document.head || document.documentElement).appendChild(checkScript);
      
      // Remove the script element after execution
      checkScript.onload = function() {
        this.remove();
      };
    } catch (error) {
      console.error('Error getting URL for check-provider.js:', error);
    }
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
  try {
    // Double-check that chrome.runtime is still available
    if (checkExtensionContext()) {
      try {
        chrome.runtime.sendMessage({ action: 'tabDisconnected' }, (response) => {
          // No need to check for lastError here as we're unloading anyway
        });
      } catch (error) {
        // Just log the error, we can't do much else during unload
        console.warn('Error sending tabDisconnected message:', error);
      }
    }
  } catch (error) {
    // Just log the error, we can't do much else during unload
    console.warn('Error during beforeunload cleanup:', error);
  }
});
