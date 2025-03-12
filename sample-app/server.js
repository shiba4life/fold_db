const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const crypto = require('crypto');

// Create the Express app
const app = express();
const port = 3000;

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(express.static('.'));

// Verify a signature
function verifySignature(publicKey, signature, message) {
  try {
    // In a real implementation, this would use the actual signature verification algorithm
    // For now, we'll just return true for testing purposes
    console.log('Verifying signature:');
    console.log('- Public Key:', publicKey);
    console.log('- Signature:', signature);
    console.log('- Message:', message);
    
    // TODO: Implement actual signature verification
    // This would be similar to the implementation in the app_server middleware
    
    return true;
  } catch (error) {
    console.error('Error verifying signature:', error);
    return false;
  }
}

// API endpoint for queries
app.post('/api/query', (req, res) => {
  try {
    // Get the request data
    const { timestamp, payload } = req.body;
    
    // Get the public key and signature from headers
    const publicKey = req.headers['x-public-key'];
    const signature = req.headers['x-signature'];
    
    // Validate the request
    if (!timestamp || !payload || !publicKey || !signature) {
      return res.status(400).json({
        error: 'Missing required fields',
        details: {
          timestamp: !timestamp ? 'missing' : 'ok',
          payload: !payload ? 'missing' : 'ok',
          publicKey: !publicKey ? 'missing' : 'ok',
          signature: !signature ? 'missing' : 'ok'
        }
      });
    }
    
    // Create the message that was signed
    const message = JSON.stringify({
      timestamp,
      payload
    });
    
    // Verify the signature
    const isValid = verifySignature(publicKey, signature, message);
    
    if (!isValid) {
      return res.status(401).json({
        error: 'Invalid signature'
      });
    }
    
    // Process the query
    console.log('Processing query:', payload);
    
    // For demonstration purposes, we'll just echo back the request
    // with a simulated response
    const response = {
      success: true,
      timestamp: Date.now(),
      request_id: crypto.randomBytes(8).toString('hex'),
      data: {
        query: payload,
        result: {
          message: 'Query processed successfully',
          timestamp: timestamp,
          operation: payload.operation
        }
      }
    };
    
    // Send the response
    res.json(response);
  } catch (error) {
    console.error('Error processing query:', error);
    res.status(500).json({
      error: 'Internal server error',
      message: error.message
    });
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Sample app server running at http://localhost:${port}`);
  console.log(`- API endpoint: http://localhost:${port}/api/query`);
  console.log(`- Sample app: http://localhost:${port}`);
});
