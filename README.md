# FoldDB Query Signer Chrome Extension

A MetaMask-style Chrome extension for signing queries to the FoldDB app_server. This extension allows web applications to securely sign requests using a private key stored in the browser, similar to how cryptocurrency wallets like MetaMask work.

## Features

- Generate and securely store ECDSA key pairs
- Sign queries and mutations for FoldDB app_server
- MetaMask-style approval UI for signing requests
- Web3-like provider API for web applications
- Public key export for server verification

## Extension Structure

```
chrome-extension/
├── manifest.json        # Extension manifest
├── css/
│   └── popup.css        # Styles for the popup UI
├── html/
│   └── popup.html       # Popup UI HTML
├── images/
│   ├── icon16.png       # Extension icons
│   ├── icon48.png
│   └── icon128.png
└── js/
    ├── background.js    # Background script
    ├── content.js       # Content script injected into pages
    ├── inject.js        # Script injected into page context
    └── popup.js         # Popup UI logic
```

## Sample App

A sample web application is included to demonstrate how to use the extension:

```
sample-app/
├── index.html           # Sample app UI
├── app.js               # Sample app logic
├── server.js            # Sample server for testing
└── package.json         # Dependencies
```

## Installation

### Chrome Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in the top-right corner)
3. Click "Load unpacked" and select the `chrome-extension` directory
4. The extension should now appear in your browser toolbar

### Sample App

1. Navigate to the `sample-app` directory
2. Install dependencies:
   ```
   npm install
   ```
3. Start the server:
   ```
   npm start
   ```
4. Open your browser to `http://localhost:3000`

## Usage

### For Users

1. Click the FoldDB Query Signer icon in your browser toolbar
2. Generate a new key pair by clicking "Generate New Keys"
3. When visiting a compatible website, the extension will prompt you to approve signing requests
4. Review the request details and click "Approve" or "Reject"

### For Developers

To integrate with the FoldDB Query Signer in your web application:

1. Check if the provider is available:

```javascript
if (window.foldDB) {
  // Provider is available
  const provider = window.foldDB;
  
  // Listen for connection events
  provider.on('connect', (data) => {
    console.log('Connected:', data.isConnected);
  });
}
```

2. Get the user's public key:

```javascript
const publicKey = await window.foldDB.getPublicKey();
```

3. Sign a request:

```javascript
const payload = {
  operation: 'query', // or 'mutation'
  content: JSON.stringify({
    operation: 'findOne',
    collection: 'users',
    filter: { username: 'testuser' }
  })
};

const signedRequest = await window.foldDB.signRequest(payload);
// signedRequest contains: timestamp, payload, and signature
```

4. Send the signed request to the server:

```javascript
const response = await fetch('http://your-server.com/api/query', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-public-key': publicKey,
    'x-signature': signedRequest.signature
  },
  body: JSON.stringify({
    timestamp: signedRequest.timestamp,
    payload: signedRequest.payload
  })
});
```

## Server-Side Verification

On the server side, you need to verify the signature:

1. Extract the public key and signature from the headers
2. Recreate the message that was signed (timestamp + payload)
3. Verify the signature using the public key

Example (pseudocode):
```javascript
function verifySignature(publicKey, signature, message) {
  // Use a crypto library to verify the ECDSA signature
  return cryptoLib.verify(publicKey, signature, message);
}

// In your request handler:
const publicKey = req.headers['x-public-key'];
const signature = req.headers['x-signature'];
const message = JSON.stringify({
  timestamp: req.body.timestamp,
  payload: req.body.payload
});

if (verifySignature(publicKey, signature, message)) {
  // Signature is valid, process the request
} else {
  // Invalid signature, reject the request
}
```

## Implementation in FoldDB

To implement this in the actual FoldDB app_server:

1. Update the `verify_signature` function in `src/datafold_node/app_server/middleware/signature.rs`
2. Implement proper signature verification using the secp256k1 curve
3. Extract the public key and signature from request headers
4. Verify the signature against the request body

## License

MIT
