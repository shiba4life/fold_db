// Constants
const KEY_STORAGE_KEY = 'folddb_keypair';
const PENDING_REQUESTS_KEY = 'folddb_pending_requests';

// State
let pendingRequests = [];
let publicKey = null;
// Make connectedTabs accessible to the popup
window.connectedTabs = new Set();
// Alias for backward compatibility
let connectedTabs = window.connectedTabs;

// Initialize the background script
async function initialize() {
  // Load pending requests from storage
  const storedRequests = await loadPendingRequests();
  pendingRequests = storedRequests || [];
  
  // Load keys from storage
  const keys = await loadKeys();
  if (keys && keys.publicKey) {
    publicKey = keys.publicKey;
  }
  
  console.log('FoldDB Query Signer background script initialized');
}

// Load keys from storage
async function loadKeys() {
  try {
    const result = await chrome.storage.local.get(KEY_STORAGE_KEY);
    return result[KEY_STORAGE_KEY] || null;
  } catch (error) {
    console.error('Error loading keys:', error);
    return null;
  }
}

// Load pending requests from storage
async function loadPendingRequests() {
  try {
    const result = await chrome.storage.local.get(PENDING_REQUESTS_KEY);
    return result[PENDING_REQUESTS_KEY] || [];
  } catch (error) {
    console.error('Error loading pending requests:', error);
    return [];
  }
}

// Save pending requests to storage
async function savePendingRequests() {
  try {
    await chrome.storage.local.set({ [PENDING_REQUESTS_KEY]: pendingRequests });
  } catch (error) {
    console.error('Error saving pending requests:', error);
  }
}

// Add a new signing request
async function addSigningRequest(request) {
  // Generate a unique ID for the request
  request.id = generateUniqueId();
  
  // Add the request to the pending list
  pendingRequests.push(request);
  
  // Save the updated list
  await savePendingRequests();
  
  // Notify the popup if it's open
  chrome.runtime.sendMessage({ action: 'newRequest' });
  
  // Return the request ID
  return request.id;
}

// Sign a request with the private key
async function signRequest(requestId) {
  try {
    // Find the request
    const requestIndex = pendingRequests.findIndex(req => req.id === requestId);
    if (requestIndex === -1) {
      throw new Error('Request not found');
    }
    
    const request = pendingRequests[requestIndex];
    
    // Load the keys
    const keys = await loadKeys();
    if (!keys || !keys.privateKey) {
      throw new Error('No private key available');
    }
    
    // Import the private key
    const privateKeyBuffer = base64ToArrayBuffer(keys.privateKey);
    const privateKey = await window.crypto.subtle.importKey(
      'pkcs8',
      privateKeyBuffer,
      {
        name: 'ECDSA',
        namedCurve: 'P-256',
      },
      false,
      ['sign']
    );
    
    // Create the message to sign (timestamp + payload stringified)
    const message = JSON.stringify({
      timestamp: request.timestamp,
      payload: request.payload
    });
    
    // Convert the message to an ArrayBuffer
    const encoder = new TextEncoder();
    const messageBuffer = encoder.encode(message);
    
    // Sign the message
    const signatureBuffer = await window.crypto.subtle.sign(
      {
        name: 'ECDSA',
        hash: { name: 'SHA-256' },
      },
      privateKey,
      messageBuffer
    );
    
    // Convert the signature to base64
    const signature = arrayBufferToBase64(signatureBuffer);
    
    // Update the request with the signature
    request.signature = signature;
    pendingRequests[requestIndex] = request;
    
    // Save the updated list
    await savePendingRequests();
    
    // Return the signed request
    return request;
  } catch (error) {
    console.error('Error signing request:', error);
    throw error;
  }
}

// Remove a request from the pending list
async function removeRequest(requestId) {
  // Find the request
  const requestIndex = pendingRequests.findIndex(req => req.id === requestId);
  if (requestIndex === -1) {
    return false;
  }
  
  // Remove the request
  pendingRequests.splice(requestIndex, 1);
  
  // Save the updated list
  await savePendingRequests();
  
  return true;
}

// Generate a unique ID
function generateUniqueId() {
  return Date.now().toString(36) + Math.random().toString(36).substring(2);
}

// Helper function to convert Base64 to ArrayBuffer
function base64ToArrayBuffer(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

// Helper function to convert ArrayBuffer to Base64
function arrayBufferToBase64(buffer) {
  const binary = String.fromCharCode.apply(null, new Uint8Array(buffer));
  return btoa(binary);
}

// Listen for messages from the popup or content scripts
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  (async () => {
    try {
      if (message.action === 'getPendingRequests') {
        // Return the list of pending requests
        sendResponse(pendingRequests);
      } else if (message.action === 'approveRequest') {
        // Sign and approve the request
        const signedRequest = await signRequest(message.requestId);
        
        // Find the tab that originated the request
        const tab = await chrome.tabs.get(signedRequest.tabId);
        
        // Send the signed request back to the content script
        await chrome.tabs.sendMessage(signedRequest.tabId, {
          action: 'requestSigned',
          requestId: signedRequest.id,
          signature: signedRequest.signature
        });
        
        // Remove the request from the pending list
        await removeRequest(message.requestId);
        
        sendResponse({ success: true });
      } else if (message.action === 'rejectRequest') {
        // Find the request
        const requestIndex = pendingRequests.findIndex(req => req.id === message.requestId);
        if (requestIndex !== -1) {
          const request = pendingRequests[requestIndex];
          
          // Notify the content script that the request was rejected
          await chrome.tabs.sendMessage(request.tabId, {
            action: 'requestRejected',
            requestId: request.id
          });
          
          // Remove the request from the pending list
          await removeRequest(message.requestId);
        }
        
        sendResponse({ success: true });
      } else if (message.action === 'signRequest') {
        // Add the request to the pending list
        const requestId = await addSigningRequest({
          origin: sender.origin || 'unknown',
          tabId: sender.tab.id,
          timestamp: message.request.timestamp,
          payload: message.request.payload
        });
        
        sendResponse({ success: true, requestId });
      } else if (message.action === 'keysAvailable') {
        // Update the public key
        publicKey = message.publicKey;
        sendResponse({ success: true });
      } else if (message.action === 'keysUnavailable') {
        // Clear the public key
        publicKey = null;
        sendResponse({ success: true });
      } else if (message.action === 'getPublicKey') {
        // Return the public key
        sendResponse({ publicKey });
      } else if (message.action === 'tabConnected') {
        // Add the tab to the connected tabs set
        connectedTabs.add(sender.tab.id);
        
        // Notify the popup if it's open
        chrome.runtime.sendMessage({ action: 'connectionChanged' });
        
        sendResponse({ success: true });
      } else if (message.action === 'tabDisconnected') {
        // Remove the tab from the connected tabs set
        connectedTabs.delete(sender.tab.id);
        
        // Notify the popup if it's open
        chrome.runtime.sendMessage({ action: 'connectionChanged' });
        
        sendResponse({ success: true });
      }
    } catch (error) {
      console.error('Error handling message:', error);
      sendResponse({ success: false, error: error.message });
    }
  })();
  
  // Return true to indicate async response
  return true;
});

// Listen for tab removal to clean up connected tabs
chrome.tabs.onRemoved.addListener((tabId) => {
  if (connectedTabs.has(tabId)) {
    connectedTabs.delete(tabId);
    
    // Notify the popup if it's open
    chrome.runtime.sendMessage({ action: 'connectionChanged' });
  }
});

// Initialize the background script
initialize();
