# Active Context

## Current Task
Implementing separate UI and App servers for DataFold node to distinguish between management UI and 3rd party API access.

## Recent Changes
1. Created a new server architecture with two separate servers:
   - UI Server (port 8080): For management UI access
   - App Server (port 8081): For 3rd party API access with cryptographic signature verification

2. Implemented UI Server:
   - Renamed existing web_server to ui_server
   - Updated error handling with UiError and UiErrorResponse
   - Maintained all existing API endpoints

3. Implemented App Server:
   - Created new app_server module with:
     - Signature verification middleware
     - CORS support for cross-origin requests
     - Comprehensive logging system
     - Error handling specific to API requests
   - Implemented cryptographic signature verification for all requests
   - Added timestamp validation to prevent replay attacks

4. Added security features:
   - Request signing with public/private key pairs
   - Timestamp validation (5-minute window)
   - Detailed security logging
   - Permission checking based on public keys

5. Updated main binary to run both servers concurrently

## Next Steps
1. Implement actual signature verification (currently a placeholder)
2. Add comprehensive tests for the new API server
3. Create documentation for 3rd party developers
4. Implement rate limiting for API requests
5. Add more detailed permission checking for operations

## Implementation Details

### API Authentication Flow
1. Client signs request with private key
2. Request includes:
   - Public key in header
   - Signature in header
   - Timestamp in request body
   - Operation details in request body
3. Server verifies:
   - Signature is valid for the request
   - Timestamp is within 5 minutes
   - Public key has permission for the operation

### API Endpoints
- GET /api/v1/status - Get API status (no authentication)
- POST /api/v1/execute - Execute an operation (requires signature)

### Security Considerations
- All API requests must be signed
- Timestamps prevent replay attacks
- Detailed security logging for audit purposes
- Permission checking based on public keys and trust distance
