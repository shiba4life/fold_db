# Active Context

## Current Task
Removing UI server, app server, and network layer code from the DataFold node to simplify the codebase.

## Recent Changes
1. Removed UI Server:
   - Removed web_server_compat module
   - Removed UI server initialization from the main binary

2. Removed App Server:
   - Removed app_server module and all its components
   - Removed app server initialization from the main binary

3. Removed Network Layer:
   - Removed network-related methods from DataFoldNode
   - Removed network field from DataFoldNode struct
   - Removed network-related imports

4. Updated Tests:
   - Removed references to app_server_tests, network_tests, and network_discovery_tests in unit tests
   - Removed references to app_server_tests, web_api_tests, user_profile_api_tests, and schema_mapping_tests in integration tests

5. Updated Main Binary:
   - Simplified to just load the node and wait for Ctrl+C signal
   - Removed server initialization and execution

## Next Steps
1. Implement a simpler, more focused API if needed
2. Update documentation to reflect the simplified architecture
3. Fix any remaining issues with tests
4. Consider alternative approaches for node communication if needed
5. Enhance the CLI with additional features as needed

## Implementation Details

### Core Functionality
- DataFoldNode now focuses solely on database operations
- No network communication or API endpoints
- Simplified architecture with fewer dependencies
- CLI interface for interacting with the node

### Security Considerations
- Security now relies on direct access control to the node
- No network exposure reduces attack surface
- CLI provides controlled access to node operations
