# Secure Web App Integration with FoldDB

This directory contains examples demonstrating how web applications can securely interact with FoldDB via DataFold. These examples showcase various security features and best practices for ensuring data security, privacy, and compliance with sovereignty and access control requirements.

## Security Features Implemented

1. **Public Key Authentication**
   - Web apps authenticate users using public-key cryptography
   - Each user has a public/private key pair registered with FoldDB
   - API requests are signed with the private key
   - Prevents unauthorized access by ensuring only authenticated users can interact with DataFold

2. **Field-Level Access Control**
   - Fine-grained permissions based on trust level
   - Sensitive fields are automatically redacted for users with insufficient trust
   - Permissions depend on trust distance, user identity, and explicit access policies

3. **Sandboxed Execution for Third-Party Integrations**
   - External services and AI models run within a secure sandbox
   - Prevents data leaks by restricting execution to a controlled environment
   - Enforces data residency by ensuring data remains within a trusted jurisdiction

4. **Payment-Based Access Control**
   - Data access is metered via the Lightning Network
   - Users or apps must pay micropayments for data retrieval
   - Prevents mass data scraping or unauthorized bulk access
   - Ensures data access is auditable and rate-limited

5. **Secure WebSocket Communication**
   - Real-time updates via authenticated WebSockets
   - Only authorized users receive live data streams
   - Prevents eavesdropping using end-to-end encrypted channels

## Example Files

1. **[secure-web-app.html](./secure-web-app.html)**
   - Demonstrates basic public key authentication
   - Shows field-level access control based on trust distance
   - Provides a simple UI for interacting with FoldDB

2. **[payment-based-access.html](./payment-based-access.html)**
   - Demonstrates payment-based access control using Lightning Network
   - Shows different data access tiers based on payment amount
   - Simulates invoice generation and payment flow

3. **[secure-websocket.html](./secure-websocket.html)**
   - Demonstrates secure WebSocket communication for real-time updates
   - Shows authentication and subscription to specific schemas
   - Provides a UI for viewing and filtering real-time events

4. **[../../../sandbox/examples/secure_sandbox_example.js](../../../sandbox/examples/secure_sandbox_example.js)**
   - Demonstrates sandboxed execution for third-party integrations
   - Shows how to run code securely without exposing raw data
   - Includes examples for sentiment analysis and data aggregation

5. **[../js/folddb-client.js](../js/folddb-client.js)**
   - Client SDK for web applications to interact with FoldDB
   - Handles authentication, signing, and communication
   - Provides a simple API for querying and mutating data

## How to Use These Examples

1. Start your DataFold node:
   ```bash
   cargo run --bin datafold_node
   ```

2. Open the example HTML files in your browser:
   ```bash
   open src/datafold_node/static/examples/secure-web-app.html
   ```

3. Generate a key pair and connect to your DataFold node
4. Explore the different security features

## Implementation Details

### Public Key Authentication

```javascript
// Sign and send a request
async function sendRequest(options) {
    const { path, method, body } = options;
    
    // Generate a nonce (random string)
    const nonce = generateNonce();
    
    // Get current timestamp
    const timestamp = Math.floor(Date.now() / 1000);
    
    // Create the message to sign
    const message = createSignatureMessage(path, method, body, timestamp, nonce);
    
    // Sign the message with the private key
    const signature = await signMessage(message, privateKey);
    
    // Prepare headers
    const headers = {
        'Content-Type': 'application/json',
        'X-Public-Key': publicKey,
        'X-Signature': signature,
        'X-Timestamp': timestamp.toString(),
        'X-Nonce': nonce
    };
    
    // Send the request
    const response = await fetch(url, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined
    });
    
    return response.json();
}
```

### Field-Level Access Control

FoldDB automatically filters fields based on the user's trust level:

```javascript
// User with full access (Trust Level 1)
{
  "username": "Alice",
  "email": "alice@example.com",
  "phone": "+1234567890",
  "payment_info": {
    "card_type": "Visa",
    "last_four": "1234"
  }
}

// User with limited access (Trust Level 5)
{
  "username": "Alice",
  "email": "REDACTED",
  "phone": "REDACTED",
  "payment_info": "REDACTED"
}
```

### Sandboxed Execution

```javascript
// Example of a sentiment analysis function that runs in the sandbox
async function analyzeSentiment(context, input) {
    // Get the message to analyze
    const { message, userId } = input;
    
    // Analyze sentiment (simplified example)
    const positiveWords = ['good', 'great', 'excellent', 'happy'];
    const negativeWords = ['bad', 'terrible', 'awful', 'sad'];
    
    const words = message.toLowerCase().split(/\W+/);
    
    let positiveCount = 0;
    let negativeCount = 0;
    
    for (const word of words) {
        if (positiveWords.includes(word)) {
            positiveCount++;
        } else if (negativeWords.includes(word)) {
            negativeCount++;
        }
    }
    
    // Calculate sentiment score
    const totalSentimentWords = positiveCount + negativeCount;
    let sentiment = 'neutral';
    let score = 0;
    
    if (totalSentimentWords > 0) {
        score = (positiveCount - negativeCount) / totalSentimentWords;
        sentiment = score > 0.3 ? 'positive' : (score < -0.3 ? 'negative' : 'neutral');
    }
    
    // Return only the analysis result, not the original message
    return {
        sentiment,
        score,
        confidence: totalSentimentWords > 5 ? 'high' : 'low',
    };
}
```

### Payment-Based Access

```javascript
// Request data access with payment
async function fetchPaidData(query, paymentAmount) {
    // Request an invoice
    const invoice = await requestInvoice(paymentAmount);
    
    // Pay the invoice
    await processPayment(invoice);
    
    // Fetch data with payment verification
    return fetch(`/api/execute?invoice=${invoice.id}`, {
        method: 'POST',
        headers: getAuthHeaders(),
        body: JSON.stringify(query)
    });
}
```

### Secure WebSocket Communication

```javascript
// Connect to WebSocket
function connectWebSocket() {
    const socket = new WebSocket(wsUrl);
    
    socket.onopen = () => {
        // Authenticate the WebSocket connection
        authenticateWebSocket(socket, publicKey, privateKey);
    };
    
    socket.onmessage = (event) => {
        const message = JSON.parse(event.data);
        
        // Verify trust level before processing
        if (validateTrust(message)) {
            updateUI(message);
        }
    };
}

// Authenticate WebSocket connection
function authenticateWebSocket(socket, publicKey, privateKey) {
    const timestamp = Math.floor(Date.now() / 1000);
    const nonce = generateNonce();
    
    // Sign authentication message
    const signature = signMessage({
        public_key: publicKey,
        timestamp,
        nonce
    }, privateKey);
    
    // Send authentication message
    socket.send(JSON.stringify({
        type: 'authenticate',
        public_key: publicKey,
        signature,
        timestamp,
        nonce
    }));
}
```

## Security Best Practices

1. **Never expose private keys in client-side code**
   - Use a secure key management system
   - Consider using a hardware security module (HSM) for key storage

2. **Implement proper error handling**
   - Don't expose sensitive information in error messages
   - Log security-related errors for monitoring

3. **Use HTTPS for all communications**
   - Encrypt data in transit
   - Prevent man-in-the-middle attacks

4. **Implement rate limiting**
   - Prevent brute force attacks
   - Limit the number of requests per user

5. **Regularly rotate keys**
   - Implement key rotation policies
   - Revoke compromised keys immediately

6. **Validate all input**
   - Sanitize user input to prevent injection attacks
   - Validate data types and formats

7. **Implement proper logging**
   - Log security-relevant events
   - Monitor for suspicious activity

## Further Reading

- [FoldDB Technical Whitepaper](../../cline_docs/FoldDB_Technical_Whitepaper.md)
- [DataFold Node Documentation](../../datafold_node/node.md)
- [Sandbox Documentation](../../../SANDBOX.md)
- [Sandbox API Documentation](../../../SANDBOX_API.md)
