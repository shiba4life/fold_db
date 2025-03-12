// Elements
const statusIndicator = document.getElementById('status-indicator');
const statusText = document.getElementById('status-text');
const publicKeyContainer = document.getElementById('public-key-container');
const publicKeyElement = document.getElementById('public-key');
const operationTypeSelect = document.getElementById('operation-type');
const queryContentTextarea = document.getElementById('query-content');
const signButton = document.getElementById('sign-button');
const signedResultContainer = document.getElementById('signed-result-container');
const signedResultElement = document.getElementById('signed-result');
const serverUrlInput = document.getElementById('server-url');
const sendButton = document.getElementById('send-button');
const serverResultContainer = document.getElementById('server-result-container');
const serverResultElement = document.getElementById('server-result');

// State
let foldDBProvider = null;
let signedQuery = null;

// Initialize the app
function initApp() {
  console.log('Initializing app, checking for FoldDB provider...');
  
  // Create a fallback provider if needed
  function createFallbackProvider() {
    console.log('Creating fallback provider for testing');
    
    // Create a mock provider for testing
    window.foldDB = {
      isConnected: true,
      eventListeners: {},
      
      getPublicKey: async function() {
        return "MOCK_PUBLIC_KEY_FOR_TESTING";
      },
      
      signRequest: async function(payload) {
        return {
          timestamp: Date.now(),
          payload,
          signature: "MOCK_SIGNATURE_FOR_TESTING"
        };
      },
      
      on: function(eventName, callback) {
        if (!this.eventListeners[eventName]) {
          this.eventListeners[eventName] = [];
        }
        
        this.eventListeners[eventName].push(callback);
        
        // If this is a 'connect' event, trigger it immediately
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
    
    return window.foldDB;
  }
  
  // Check if the FoldDB provider is available
  if (window.foldDB) {
    console.log('FoldDB provider found immediately');
    foldDBProvider = window.foldDB;
    updateConnectionStatus(true);
    
    // Get the public key
    getPublicKey();
    
    // Enable the sign button
    signButton.disabled = false;
    
    // Listen for connection events
    foldDBProvider.on('connect', (data) => {
      console.log('Connect event received:', data);
      updateConnectionStatus(data.isConnected);
    });
  } else {
    console.log('FoldDB provider not found, waiting for it to be injected...');
    
    // Provider not available, check if it's being injected
    window.addEventListener('folddb:ready', () => {
      console.log('Received folddb:ready event');
      if (window.foldDB) {
        foldDBProvider = window.foldDB;
        updateConnectionStatus(true);
        
        // Get the public key
        getPublicKey();
        
        // Enable the sign button
        signButton.disabled = false;
        
        // Listen for connection events
        foldDBProvider.on('connect', (data) => {
          console.log('Connect event received:', data);
          updateConnectionStatus(data.isConnected);
        });
      } else {
        console.log('folddb:ready event received but window.foldDB is still not available');
      }
    });
    
    // Check again after a delay in case the event was missed
    setTimeout(() => {
      console.log('Checking again for FoldDB provider...');
      if (!foldDBProvider && window.foldDB) {
        console.log('FoldDB provider found after delay');
        foldDBProvider = window.foldDB;
        updateConnectionStatus(true);
        
        // Get the public key
        getPublicKey();
        
        // Enable the sign button
        signButton.disabled = false;
        
        // Listen for connection events
        foldDBProvider.on('connect', (data) => {
          console.log('Connect event received:', data);
          updateConnectionStatus(data.isConnected);
        });
      } else if (!foldDBProvider) {
        console.log('FoldDB provider still not found. Creating fallback provider for testing.');
        
        // Create a fallback provider for testing
        foldDBProvider = createFallbackProvider();
        updateConnectionStatus(true);
        
        // Get the public key
        getPublicKey();
        
        // Enable the sign button
        signButton.disabled = false;
        
        // Add a notice that we're using a fallback provider
        const statusContainer = document.querySelector('.status');
        if (statusContainer) {
          const fallbackNotice = document.createElement('div');
          fallbackNotice.textContent = 'Using fallback provider for testing (extension not detected)';
          fallbackNotice.style.marginTop = '10px';
          fallbackNotice.style.color = '#ff9800';
          fallbackNotice.style.fontWeight = 'bold';
          statusContainer.appendChild(fallbackNotice);
          
          // Add a button to reload the page
          const reloadButton = document.createElement('button');
          reloadButton.textContent = 'Reload Page';
          reloadButton.className = 'btn primary-btn';
          reloadButton.style.marginTop = '10px';
          reloadButton.addEventListener('click', () => {
            window.location.reload();
          });
          statusContainer.appendChild(reloadButton);
        } else {
          console.warn('Status container not found, could not add fallback notice');
        }
      }
    }, 2000);
  }
  
  // Set up event listeners
  signButton.addEventListener('click', signQuery);
  sendButton.addEventListener('click', sendQuery);
}

// Update the connection status UI
function updateConnectionStatus(isConnected) {
  if (isConnected) {
    statusIndicator.classList.remove('status-disconnected');
    statusIndicator.classList.add('status-connected');
    statusText.textContent = 'Connected';
  } else {
    statusIndicator.classList.remove('status-connected');
    statusIndicator.classList.add('status-disconnected');
    statusText.textContent = 'Disconnected';
    
    // Disable buttons
    signButton.disabled = true;
    sendButton.disabled = true;
  }
}

// Get the public key from the provider
async function getPublicKey() {
  try {
    if (!foldDBProvider) {
      throw new Error('FoldDB provider not available');
    }
    
    const publicKey = await foldDBProvider.getPublicKey();
    
    // Display the public key
    publicKeyContainer.style.display = 'block';
    publicKeyElement.textContent = publicKey;
    
    return publicKey;
  } catch (error) {
    console.error('Error getting public key:', error);
    publicKeyContainer.style.display = 'block';
    publicKeyElement.textContent = `Error: ${error.message}`;
    publicKeyElement.classList.add('error');
    
    return null;
  }
}

// Sign a query using the provider
async function signQuery() {
  try {
    // Clear previous results
    signedResultContainer.style.display = 'none';
    serverResultContainer.style.display = 'none';
    sendButton.disabled = true;
    
    if (!foldDBProvider) {
      throw new Error('FoldDB provider not available');
    }
    
    // Get the query content
    let queryContent;
    try {
      queryContent = JSON.parse(queryContentTextarea.value);
    } catch (error) {
      throw new Error(`Invalid JSON: ${error.message}`);
    }
    
    // Create the payload
    const payload = {
      operation: operationTypeSelect.value,
      content: JSON.stringify(queryContent)
    };
    
    // Sign the request
    signedQuery = await foldDBProvider.signRequest(payload);
    
    // Display the signed query
    signedResultContainer.style.display = 'block';
    signedResultElement.textContent = JSON.stringify(signedQuery, null, 2);
    signedResultElement.classList.remove('error');
    
    // Enable the send button
    sendButton.disabled = false;
  } catch (error) {
    console.error('Error signing query:', error);
    signedResultContainer.style.display = 'block';
    signedResultElement.textContent = `Error: ${error.message}`;
    signedResultElement.classList.add('error');
    
    // Disable the send button
    sendButton.disabled = true;
  }
}

// Send a signed query to the app server
async function sendQuery() {
  try {
    // Clear previous results
    serverResultContainer.style.display = 'none';
    
    if (!signedQuery) {
      throw new Error('No signed query available');
    }
    
    // Get the server URL
    const serverUrl = serverUrlInput.value.trim();
    if (!serverUrl) {
      throw new Error('Server URL is required');
    }
    
    // Send the request
    const response = await fetch(serverUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-public-key': await foldDBProvider.getPublicKey(),
        'x-signature': signedQuery.signature
      },
      body: JSON.stringify({
        timestamp: signedQuery.timestamp,
        payload: signedQuery.payload
      })
    });
    
    // Parse the response
    const result = await response.json();
    
    // Display the result
    serverResultContainer.style.display = 'block';
    serverResultElement.textContent = JSON.stringify(result, null, 2);
    serverResultElement.classList.remove('error');
    
    if (!response.ok) {
      serverResultElement.classList.add('error');
    } else {
      serverResultElement.classList.add('success');
    }
  } catch (error) {
    console.error('Error sending query:', error);
    serverResultContainer.style.display = 'block';
    serverResultElement.textContent = `Error: ${error.message}`;
    serverResultElement.classList.add('error');
  }
}

// Initialize the app when the DOM is loaded
document.addEventListener('DOMContentLoaded', initApp);

// Also check for the provider when the window loads
// (in case the DOM loaded event was missed)
window.addEventListener('load', () => {
  if (!foldDBProvider) {
    initApp();
  }
});
