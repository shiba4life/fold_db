/**
 * FoldDB Client SDK for Web Applications
 * 
 * This SDK provides a secure way for web applications to interact with FoldDB
 * using public key authentication.
 */

class FoldDBClient {
    /**
     * Create a new FoldDB client
     * 
     * @param {Object} config - Configuration options
     * @param {string} config.nodeUrl - URL of the DataFold node
     * @param {string} config.publicKey - Public key for authentication
     * @param {string} config.privateKey - Private key for signing requests
     */
    constructor(config) {
        this.nodeUrl = config.nodeUrl;
        this.publicKey = config.publicKey;
        this.privateKey = config.privateKey;
    }

    /**
     * Sign and send a request to the FoldDB API
     * 
     * @param {Object} options - Request options
     * @param {string} options.path - API path (e.g., '/api/execute')
     * @param {string} options.method - HTTP method (GET, POST, DELETE)
     * @param {Object} [options.body] - Request body (for POST requests)
     * @returns {Promise<any>} - Response data
     */
    async sendRequest(options) {
        const { path, method, body } = options;
        
        // Generate a nonce (random string)
        const nonce = this._generateNonce();
        
        // Get current timestamp
        const timestamp = Math.floor(Date.now() / 1000);
        
        // Create the message to sign
        const message = this._createSignatureMessage(path, method, body, timestamp, nonce);
        
        // Sign the message
        const signature = await this._signMessage(message);
        
        // Prepare headers
        const headers = {
            'Content-Type': 'application/json',
            'X-Public-Key': this.publicKey,
            'X-Signature': signature,
            'X-Timestamp': timestamp.toString(),
            'X-Nonce': nonce
        };
        
        // Send the request
        const url = `${this.nodeUrl}${path}`;
        const requestOptions = {
            method,
            headers,
            body: body ? JSON.stringify(body) : undefined
        };
        
        const response = await fetch(url, requestOptions);
        
        // Handle response
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.error || 'Request failed');
        }
        
        return response.json();
    }
    
    /**
     * Query data from FoldDB
     * 
     * @param {Object} options - Query options
     * @param {string} options.schema - Schema name
     * @param {Array<string>} options.fields - Fields to retrieve
     * @param {Object} [options.where] - Query conditions
     * @returns {Promise<any>} - Query results
     */
    async query(options) {
        const { schema, fields, where } = options;
        
        // Create operation JSON
        const operation = JSON.stringify({
            type: 'Query',
            schema,
            fields,
            where: where || {}
        });
        
        return this.sendRequest({
            path: '/api/execute',
            method: 'POST',
            body: { operation }
        });
    }
    
    /**
     * Create data in FoldDB
     * 
     * @param {Object} options - Create options
     * @param {string} options.schema - Schema name
     * @param {Object} options.data - Data to create
     * @returns {Promise<any>} - Created data
     */
    async create(options) {
        const { schema, data } = options;
        
        // Create operation JSON
        const operation = JSON.stringify({
            type: 'Mutation',
            operation: 'Create',
            schema,
            data
        });
        
        return this.sendRequest({
            path: '/api/execute',
            method: 'POST',
            body: { operation }
        });
    }
    
    /**
     * Update data in FoldDB
     * 
     * @param {Object} options - Update options
     * @param {string} options.schema - Schema name
     * @param {string} options.id - ID of the record to update
     * @param {Object} options.data - Data to update
     * @returns {Promise<any>} - Updated data
     */
    async update(options) {
        const { schema, id, data } = options;
        
        // Create operation JSON
        const operation = JSON.stringify({
            type: 'Mutation',
            operation: 'Update',
            schema,
            id,
            data
        });
        
        return this.sendRequest({
            path: '/api/execute',
            method: 'POST',
            body: { operation }
        });
    }
    
    /**
     * Delete data from FoldDB
     * 
     * @param {Object} options - Delete options
     * @param {string} options.schema - Schema name
     * @param {string} options.id - ID of the record to delete
     * @returns {Promise<any>} - Deletion result
     */
    async delete(options) {
        const { schema, id } = options;
        
        // Create operation JSON
        const operation = JSON.stringify({
            type: 'Mutation',
            operation: 'Delete',
            schema,
            id
        });
        
        return this.sendRequest({
            path: '/api/execute',
            method: 'POST',
            body: { operation }
        });
    }
    
    /**
     * Register a new public key with FoldDB
     * 
     * @param {Object} options - Registration options
     * @param {string} options.publicKey - Public key to register
     * @param {number} options.trustLevel - Trust level to assign
     * @param {string} [options.adminToken] - Admin token for higher trust levels
     * @returns {Promise<any>} - Registration result
     */
    async registerKey(options) {
        const { publicKey, trustLevel, adminToken } = options;
        
        return this.sendRequest({
            path: '/api/auth/register',
            method: 'POST',
            body: {
                public_key: publicKey,
                trust_level: trustLevel,
                admin_token: adminToken
            }
        });
    }
    
    /**
     * Generate a random nonce
     * 
     * @returns {string} - Random nonce
     * @private
     */
    _generateNonce() {
        const array = new Uint8Array(16);
        window.crypto.getRandomValues(array);
        return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
    }
    
    /**
     * Create a message to sign
     * 
     * @param {string} path - Request path
     * @param {string} method - HTTP method
     * @param {Object} body - Request body
     * @param {number} timestamp - Request timestamp
     * @param {string} nonce - Request nonce
     * @returns {string} - Message to sign
     * @private
     */
    _createSignatureMessage(path, method, body, timestamp, nonce) {
        return JSON.stringify({
            path,
            method,
            body: body ? JSON.stringify(body) : '',
            timestamp,
            nonce
        });
    }
    
    /**
     * Sign a message using the private key
     * 
     * @param {string} message - Message to sign
     * @returns {Promise<string>} - Signature
     * @private
     */
    async _signMessage(message) {
        // In a real implementation, this would use the Web Crypto API
        // to sign the message with the private key
        
        // For now, we'll just return a placeholder
        // This should be replaced with actual signing logic
        
        // Example implementation using Web Crypto API:
        // 1. Import the private key
        // 2. Sign the message
        // 3. Return the signature as a base64 string
        
        // Placeholder implementation
        return `signed-${this.publicKey}-${message.length}`;
    }
}

// Example usage:
/*
const client = new FoldDBClient({
    nodeUrl: 'http://localhost:3000',
    publicKey: 'your-public-key',
    privateKey: 'your-private-key'
});

// Query data
client.query({
    schema: 'user-profile',
    fields: ['name', 'email'],
    where: { id: '123' }
})
.then(result => console.log(result))
.catch(error => console.error(error));

// Create data
client.create({
    schema: 'user-profile',
    data: {
        name: 'John Doe',
        email: 'john@example.com'
    }
})
.then(result => console.log(result))
.catch(error => console.error(error));
*/
