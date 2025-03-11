# Active Context

## Current Task
Separated DataFold Node's web server into two distinct servers:
1. UI Server - For the DataFold Node's own web interface
2. API Server - For other webapps to query and interact with DataFold

## Recent Changes
- Created a new ApiServer class that handles all API endpoints for external applications
- Modified the existing WebServer class to focus only on UI functionality
- Updated the main function to run both servers concurrently
- Configured the servers to run on different ports (UI on 8080, API on 8081 by default)
- Made the ports configurable via environment variables (UI_PORT and API_PORT)
- Fixed the "Missing X-Public-Key header" error by ensuring proper authentication in the API server
- Added the execute endpoint to the UI server to support mutations from the UI interface
- Fixed "Method Not Allowed" error when trying to perform mutations from the UI

## Next Steps
1. Add user authentication to support multiple users
2. Add ability to edit and delete posts
3. Implement comments on posts
4. Add image upload support for posts
5. Implement API versioning and rate limiting
6. Enhance authentication and authorization mechanisms
7. Add support for more complex schema relationships
8. Implement schema versioning and migration
9. Add more comprehensive tests for DataFold Node functionality
