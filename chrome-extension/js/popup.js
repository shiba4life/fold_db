// Constants
const KEY_STORAGE_KEY = 'folddb_keypair';
const CONNECTION_STATUS_ELEMENT = document.getElementById('connection-status');
const PUBLIC_KEY_ELEMENT = document.getElementById('public-key');
const PUBLIC_KEY_DISPLAY = document.getElementById('public-key-display');
const NO_KEYS_MESSAGE = document.getElementById('no-keys-message');
const KEYS_EXIST_CONTROLS = document.getElementById('keys-exist-controls');
const PENDING_REQUESTS_CONTAINER = document.getElementById('pending-requests-container');
const REQUEST_LIST = document.getElementById('request-list');
const NO_REQUESTS_MESSAGE = document.getElementById('no-requests-message');

// Button elements
const GENERATE_KEY_BTN = document.getElementById('generate-key-btn');
const EXPORT_KEY_BTN = document.getElementById('export-key-btn');
const RESET_KEY_BTN = document.getElementById('reset-key-btn');

// Initialize the popup
document.addEventListener('DOMContentLoaded', async () => {
  // Check if keys exist
  const keys = await getKeys();
  updateUIBasedOnKeys(keys);
  
  // Check connection status
  checkConnectionStatus();
  
  // Check for pending requests
  checkPendingRequests();
  
  // Set up event listeners
  setupEventListeners();
});

// Set up event listeners for buttons
function setupEventListeners() {
  // Generate new keys
  GENERATE_KEY_BTN.addEventListener('click', async () => {
    try {
      const keys = await generateKeys();
      updateUIBasedOnKeys(keys);
      showNotification('Keys generated successfully!');
    } catch (error) {
      console.error('Error generating keys:', error);
      showNotification('Failed to generate keys. Please try again.', true);
    }
  });
  
  // Export public key
  EXPORT_KEY_BTN.addEventListener('click', async () => {
    try {
      const keys = await getKeys();
      if (keys && keys.publicKey) {
        // Copy to clipboard
        await navigator.clipboard.writeText(keys.publicKey);
        showNotification('Public key copied to clipboard!');
      }
    } catch (error) {
      console.error('Error exporting public key:', error);
      showNotification('Failed to export public key.', true);
    }
  });
  
  // Reset keys
  RESET_KEY_BTN.addEventListener('click', async () => {
    if (confirm('Are you sure you want to reset your keys? This action cannot be undone.')) {
      try {
        await chrome.storage.local.remove(KEY_STORAGE_KEY);
        updateUIBasedOnKeys(null);
        showNotification('Keys have been reset.');
      } catch (error) {
        console.error('Error resetting keys:', error);
        showNotification('Failed to reset keys.', true);
      }
    }
  });
}

// Generate a new key pair using the Web Crypto API
async function generateKeys() {
  try {
    // Generate a new key pair
    const keyPair = await window.crypto.subtle.generateKey(
      {
        name: 'ECDSA',
        namedCurve: 'P-256', // Use P-256 curve (similar to secp256k1 used in crypto wallets)
      },
      true, // extractable
      ['sign', 'verify'] // key usages
    );
    
    // Export the public key
    const publicKeyBuffer = await window.crypto.subtle.exportKey(
      'spki', // SubjectPublicKeyInfo format
      keyPair.publicKey
    );
    
    // Export the private key
    const privateKeyBuffer = await window.crypto.subtle.exportKey(
      'pkcs8', // PKCS #8 format
      keyPair.privateKey
    );
    
    // Convert to base64
    const publicKeyBase64 = arrayBufferToBase64(publicKeyBuffer);
    const privateKeyBase64 = arrayBufferToBase64(privateKeyBuffer);
    
    // Store the keys
    const keys = {
      publicKey: publicKeyBase64,
      privateKey: privateKeyBase64
    };
    
    await chrome.storage.local.set({ [KEY_STORAGE_KEY]: keys });
    
    return keys;
  } catch (error) {
    console.error('Error generating keys:', error);
    throw error;
  }
}

// Get the stored keys
async function getKeys() {
  try {
    const result = await chrome.storage.local.get(KEY_STORAGE_KEY);
    return result[KEY_STORAGE_KEY] || null;
  } catch (error) {
    console.error('Error getting keys:', error);
    return null;
  }
}

// Update the UI based on whether keys exist
function updateUIBasedOnKeys(keys) {
  if (keys && keys.publicKey) {
    // Keys exist
    NO_KEYS_MESSAGE.style.display = 'none';
    KEYS_EXIST_CONTROLS.style.display = 'block';
    
    // Update public key display
    const truncatedPublicKey = truncateKey(keys.publicKey);
    PUBLIC_KEY_ELEMENT.textContent = truncatedPublicKey;
    PUBLIC_KEY_DISPLAY.textContent = truncatedPublicKey;
    
    // Notify background script that keys are available
    try {
      chrome.runtime.sendMessage({ action: 'keysAvailable', publicKey: keys.publicKey }, (response) => {
        if (chrome.runtime.lastError) {
          console.warn('Error notifying background script about keys:', chrome.runtime.lastError);
        }
      });
    } catch (error) {
      console.warn('Error sending keysAvailable message:', error);
    }
  } else {
    // No keys
    NO_KEYS_MESSAGE.style.display = 'block';
    KEYS_EXIST_CONTROLS.style.display = 'none';
    PUBLIC_KEY_ELEMENT.textContent = 'Not generated';
    
    // Notify background script that keys are not available
    try {
      chrome.runtime.sendMessage({ action: 'keysUnavailable' }, (response) => {
        if (chrome.runtime.lastError) {
          console.warn('Error notifying background script about keys:', chrome.runtime.lastError);
        }
      });
    } catch (error) {
      console.warn('Error sending keysUnavailable message:', error);
    }
  }
}

// Check connection status with app_server
async function checkConnectionStatus() {
  try {
    // In Manifest V3, we can't use getBackgroundPage, so we'll use messaging instead
    // Query for tabs that might be connected
    const tabs = await chrome.tabs.query({});
    const connectedTabs = tabs.filter(tab => {
      return tab.url && (
        tab.url.includes('localhost') || 
        tab.url.includes('127.0.0.1') ||
        tab.url.includes('folddb.com')
      );
    });
    
    if (connectedTabs.length > 0) {
      CONNECTION_STATUS_ELEMENT.textContent = 'Connected';
      CONNECTION_STATUS_ELEMENT.classList.add('connected');
      CONNECTION_STATUS_ELEMENT.classList.remove('disconnected');
      
      // Add a small badge with the count
      const countBadge = document.createElement('span');
      countBadge.textContent = connectedTabs.length;
      countBadge.style.marginLeft = '5px';
      countBadge.style.backgroundColor = '#4CAF50';
      countBadge.style.color = 'white';
      countBadge.style.borderRadius = '50%';
      countBadge.style.padding = '2px 6px';
      countBadge.style.fontSize = '10px';
      
      // Clear any existing badges
      while (CONNECTION_STATUS_ELEMENT.childNodes.length > 0) {
        CONNECTION_STATUS_ELEMENT.removeChild(CONNECTION_STATUS_ELEMENT.lastChild);
      }
      
      // Add the text and badge
      CONNECTION_STATUS_ELEMENT.textContent = 'Connected';
      CONNECTION_STATUS_ELEMENT.appendChild(countBadge);
    } else {
      CONNECTION_STATUS_ELEMENT.textContent = 'Disconnected';
      CONNECTION_STATUS_ELEMENT.classList.add('disconnected');
      CONNECTION_STATUS_ELEMENT.classList.remove('connected');
    }
  } catch (error) {
    console.error('Error checking connection status:', error);
    CONNECTION_STATUS_ELEMENT.textContent = 'Unknown';
  }
}

// Check for pending signing requests
async function checkPendingRequests() {
  try {
    // Use a Promise with timeout to handle potential connection issues
    const pendingRequests = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        resolve([]); // Resolve with empty array on timeout
        console.warn('Request for pending requests timed out');
      }, 1000);
      
      chrome.runtime.sendMessage({ action: 'getPendingRequests' }, (response) => {
        clearTimeout(timeout);
        if (chrome.runtime.lastError) {
          console.warn('Error getting pending requests:', chrome.runtime.lastError);
          resolve([]); // Resolve with empty array on error
        } else {
          resolve(response || []);
        }
      });
    });
    
    if (pendingRequests && pendingRequests.length > 0) {
      PENDING_REQUESTS_CONTAINER.style.display = 'block';
      NO_REQUESTS_MESSAGE.style.display = 'none';
      
      // Clear existing requests
      REQUEST_LIST.innerHTML = '';
      
      // Add each request to the list
      pendingRequests.forEach(request => {
        const requestElement = createRequestElement(request);
        REQUEST_LIST.appendChild(requestElement);
      });
    } else {
      PENDING_REQUESTS_CONTAINER.style.display = 'block';
      NO_REQUESTS_MESSAGE.style.display = 'block';
      REQUEST_LIST.innerHTML = '';
    }
  } catch (error) {
    console.error('Error checking pending requests:', error);
    PENDING_REQUESTS_CONTAINER.style.display = 'block';
    NO_REQUESTS_MESSAGE.style.display = 'block';
    REQUEST_LIST.innerHTML = '';
  }
}

// Create an element for a pending request
function createRequestElement(request) {
  const requestElement = document.createElement('div');
  requestElement.className = 'request-item';
  
  const requestDetails = document.createElement('div');
  requestDetails.className = 'request-details';
  
  // Format the request details
  const operation = JSON.parse(request.payload.content).operation || 'Unknown';
  requestDetails.innerHTML = `
    <strong>Origin:</strong> ${request.origin}<br>
    <strong>Operation:</strong> ${operation}<br>
    <strong>Timestamp:</strong> ${new Date(request.timestamp).toLocaleString()}
  `;
  
  const requestActions = document.createElement('div');
  requestActions.className = 'request-actions';
  
  // Approve button
  const approveButton = document.createElement('button');
  approveButton.className = 'btn primary-btn action-btn';
  approveButton.textContent = 'Approve';
  approveButton.addEventListener('click', () => {
    approveRequest(request.id);
  });
  
  // Reject button
  const rejectButton = document.createElement('button');
  rejectButton.className = 'btn danger-btn action-btn';
  rejectButton.textContent = 'Reject';
  rejectButton.addEventListener('click', () => {
    rejectRequest(request.id);
  });
  
  requestActions.appendChild(approveButton);
  requestActions.appendChild(rejectButton);
  
  requestElement.appendChild(requestDetails);
  requestElement.appendChild(requestActions);
  
  return requestElement;
}

// Approve a signing request
async function approveRequest(requestId) {
  try {
    // Use a Promise with timeout to handle potential connection issues
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Request timed out'));
        console.warn('Approve request timed out');
      }, 1000);
      
      chrome.runtime.sendMessage({ 
        action: 'approveRequest', 
        requestId 
      }, (response) => {
        clearTimeout(timeout);
        if (chrome.runtime.lastError) {
          console.warn('Error approving request:', chrome.runtime.lastError);
          reject(new Error(chrome.runtime.lastError.message));
        } else {
          resolve(response);
        }
      });
    });
    
    // Refresh the pending requests
    checkPendingRequests();
    showNotification('Request approved and signed!');
  } catch (error) {
    console.error('Error approving request:', error);
    showNotification('Failed to approve request.', true);
  }
}

// Reject a signing request
async function rejectRequest(requestId) {
  try {
    // Use a Promise with timeout to handle potential connection issues
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Request timed out'));
        console.warn('Reject request timed out');
      }, 1000);
      
      chrome.runtime.sendMessage({ 
        action: 'rejectRequest', 
        requestId 
      }, (response) => {
        clearTimeout(timeout);
        if (chrome.runtime.lastError) {
          console.warn('Error rejecting request:', chrome.runtime.lastError);
          reject(new Error(chrome.runtime.lastError.message));
        } else {
          resolve(response);
        }
      });
    });
    
    // Refresh the pending requests
    checkPendingRequests();
    showNotification('Request rejected.');
  } catch (error) {
    console.error('Error rejecting request:', error);
    showNotification('Failed to reject request.', true);
  }
}

// Helper function to convert ArrayBuffer to Base64
function arrayBufferToBase64(buffer) {
  const binary = String.fromCharCode.apply(null, new Uint8Array(buffer));
  return btoa(binary);
}

// Helper function to truncate a key for display
function truncateKey(key) {
  if (!key) return '';
  return key.substring(0, 10) + '...' + key.substring(key.length - 10);
}

// Show a notification in the popup
function showNotification(message, isError = false) {
  // Create notification element if it doesn't exist
  let notification = document.getElementById('notification');
  if (!notification) {
    notification = document.createElement('div');
    notification.id = 'notification';
    notification.style.position = 'fixed';
    notification.style.bottom = '10px';
    notification.style.left = '50%';
    notification.style.transform = 'translateX(-50%)';
    notification.style.padding = '8px 16px';
    notification.style.borderRadius = '4px';
    notification.style.color = 'white';
    notification.style.fontSize = '14px';
    notification.style.zIndex = '1000';
    document.body.appendChild(notification);
  }
  
  // Set notification style based on type
  notification.style.backgroundColor = isError ? '#f44336' : '#4CAF50';
  notification.textContent = message;
  
  // Show the notification
  notification.style.display = 'block';
  
  // Hide after 3 seconds
  setTimeout(() => {
    notification.style.display = 'none';
  }, 3000);
}

// Listen for messages from the background script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'newRequest') {
    // New signing request received
    checkPendingRequests();
  } else if (message.action === 'connectionChanged') {
    // Connection status changed
    checkConnectionStatus();
  }
  
  // Return true to indicate async response
  return true;
});
