# DataFold UI Implementation Plan

This document outlines the plan for creating a user-friendly UI for DataFold that allows loading schemas, mutations, and queries with a clean interface and Rust backend.

## Overview

We will enhance the existing DataFold system to provide a more intuitive and user-friendly interface for managing schemas, queries, and mutations. The implementation will focus on:

1. Enhancing the existing TCP server to handle HTTP requests
2. Creating a sample data management system
3. Improving the frontend UI for better usability
4. Implementing one-click loading of sample data

## 1. Backend Enhancements

### 1.1 TCP Server Enhancement

We will extend the existing `TcpServer` in `fold_node/src/datafold_node/tcp_server.rs` to:

- Detect and handle HTTP requests alongside the current TCP protocol
- Implement REST API endpoints for schemas, queries, mutations, and sample data
- Serve static files for the enhanced UI
- Maintain backward compatibility with existing TCP clients

#### HTTP Request Detection and Handling

```rust
// Pseudocode for HTTP detection in handle_connection
async fn handle_connection(mut socket: TcpStream, node: Arc<Mutex<DataFoldNode>>) -> FoldDbResult<()> {
    // Buffer for initial bytes to detect protocol
    let mut buffer = [0u8; 8];
    
    // Read initial bytes without consuming them (peek)
    match socket.peek(&mut buffer).await {
        Ok(n) if n >= 4 => {
            // Check if this looks like an HTTP request
            if &buffer[0..4] == b"GET " || &buffer[0..4] == b"POST" || 
               &buffer[0..4] == b"PUT " || &buffer[0..4] == b"DELE" {
                return handle_http_request(socket, node).await;
            }
            
            // Otherwise, handle as regular TCP request
            return handle_tcp_request(socket, node).await;
        },
        _ => {
            // Not enough data or error, try regular TCP handling
            return handle_tcp_request(socket, node).await;
        }
    }
}
```

#### REST API Endpoints

We will implement the following HTTP endpoints:

- **Schemas**
  - `GET /api/schemas` - List all schemas
  - `GET /api/schema/:name` - Get a specific schema
  - `POST /api/schema` - Create a new schema
  - `PUT /api/schema/:name` - Update a schema
  - `DELETE /api/schema/:name` - Delete a schema

- **Operations**
  - `POST /api/query` - Execute a query
  - `POST /api/mutation` - Execute a mutation

- **Samples**
  - `GET /api/samples/schemas` - List sample schemas
  - `GET /api/samples/queries` - List sample queries
  - `GET /api/samples/mutations` - List sample mutations
  - `GET /api/samples/schema/:name` - Get a specific sample schema
  - `GET /api/samples/query/:name` - Get a specific sample query
  - `GET /api/samples/mutation/:name` - Get a specific sample mutation

#### Static File Serving

```rust
// Pseudocode for static file handling
async fn handle_static_file(path: &str) -> FoldDbResult<HttpResponse> {
    let file_path = format!("src/datafold_node/static/{}", path);
    
    if let Ok(content) = tokio::fs::read(&file_path).await {
        let content_type = match Path::new(&file_path).extension().and_then(OsStr::to_str) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            _ => "application/octet-stream",
        };
        
        Ok(HttpResponse {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), content_type.to_string()),
                ("Content-Length".to_string(), content.len().to_string()),
            ],
            body: content,
        })
    } else {
        Ok(HttpResponse {
            status: 404,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: b"File not found".to_vec(),
        })
    }
}
```

### 1.2 Sample Data Management

We will create a new module in `fold_node/src/datafold_node/samples/` to organize and manage sample schemas, queries, and mutations:

```
fold_node/src/datafold_node/samples/
├── mod.rs                 # Module definition
├── schema_samples.rs      # Sample schema management
├── query_samples.rs       # Sample query management
├── mutation_samples.rs    # Sample mutation management
└── data/                  # Sample data files
    ├── schemas/           # Sample schema JSON files
    ├── queries/           # Sample query JSON files
    └── mutations/         # Sample mutation JSON files
```

#### Sample Manager Implementation

```rust
// Pseudocode for sample manager
pub struct SampleManager {
    schemas: HashMap<String, Schema>,
    queries: HashMap<String, Value>,
    mutations: HashMap<String, Value>,
}

impl SampleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            schemas: HashMap::new(),
            queries: HashMap::new(),
            mutations: HashMap::new(),
        };
        
        manager.load_samples();
        manager
    }
    
    fn load_samples(&mut self) {
        // Load sample schemas
        self.load_schema_samples();
        
        // Load sample queries
        self.load_query_samples();
        
        // Load sample mutations
        self.load_mutation_samples();
    }
    
    // Methods to get samples
    pub fn get_schema_sample(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }
    
    pub fn get_query_sample(&self, name: &str) -> Option<&Value> {
        self.queries.get(name)
    }
    
    pub fn get_mutation_sample(&self, name: &str) -> Option<&Value> {
        self.mutations.get(name)
    }
    
    // Methods to list samples
    pub fn list_schema_samples(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
    
    pub fn list_query_samples(&self) -> Vec<String> {
        self.queries.keys().cloned().collect()
    }
    
    pub fn list_mutation_samples(&self) -> Vec<String> {
        self.mutations.keys().cloned().collect()
    }
}
```

## 2. Frontend UI Improvements

We will enhance the existing UI files in `fold_node/src/datafold_node/static/` to create a more intuitive and user-friendly interface.

### 2.1 UI Structure

```
fold_node/src/datafold_node/static/
├── index.html             # Main HTML file
├── components/            # UI components
│   ├── schema-tab.html    # Enhanced schema management
│   ├── query-tab.html     # Enhanced query builder
│   ├── mutation-tab.html  # Enhanced mutation builder
│   ├── samples-tab.html   # New sample data library
│   └── results-tab.html   # Enhanced results viewer
├── css/                   # Stylesheets
│   ├── styles.css         # Base styles
│   └── modern-styles.css  # Enhanced modern styles
└── js/                    # JavaScript files
    ├── app.js             # Main application logic
    ├── schema.js          # Schema management
    ├── query.js           # Query builder
    ├── mutation.js        # Mutation builder
    ├── samples.js         # Sample data library
    ├── results.js         # Results viewer
    └── utils.js           # Utility functions
```

### 2.2 Schema Management UI

We will enhance the schema management UI to provide:

- Visual schema representation
- Form-based schema creation/editing
- One-click sample schema loading
- Import/export functionality

### 2.3 Query Builder UI

We will create an intuitive query builder that allows:

- Selecting a schema from loaded schemas
- Choosing fields to query with checkboxes
- Building filters with a visual interface
- One-click sample query loading

### 2.4 Mutation Builder UI

We will create a user-friendly mutation builder that supports:

- Selecting a schema from loaded schemas
- Choosing mutation type (create, update, delete)
- Form-based data entry with validation
- One-click sample mutation loading

### 2.5 Sample Data Library

We will implement a sample data library that provides:

- Categorized sample schemas, queries, and mutations
- Preview functionality
- One-click loading
- Description of each sample

## 3. Sample Data

We will prepare the following sample data:

### 3.1 Sample Schemas

1. **User Profile Schema**
   - Fields: username, email, full_name, bio, age, location
   - Permissions: Various read/write policies
   - Use case: User management system

2. **Product Catalog Schema**
   - Fields: product_id, name, description, price, category, inventory_count
   - Permissions: Public read, restricted write
   - Use case: E-commerce platform

3. **Blog Post Schema**
   - Fields: title, content, author, publish_date, tags, comments
   - Permissions: Public read, author-only write
   - Use case: Content management system

4. **Social Media Post Schema**
   - Fields: user_id, content, media_url, likes, comments, timestamp
   - Permissions: Friend-based read, owner-only write
   - Use case: Social media platform

5. **Financial Transaction Schema**
   - Fields: transaction_id, amount, sender, recipient, timestamp, status
   - Permissions: Strict read/write policies
   - Use case: Payment processing system

### 3.2 Sample Queries

1. **Basic User Query**
   - Schema: UserProfile
   - Fields: username, email
   - Filter: None

2. **Filtered Product Query**
   - Schema: ProductCatalog
   - Fields: name, price, inventory_count
   - Filter: category = "Electronics"

3. **Recent Blog Posts Query**
   - Schema: BlogPost
   - Fields: title, author, publish_date
   - Filter: publish_date > "2023-01-01"

4. **Popular Social Posts Query**
   - Schema: SocialMediaPost
   - Fields: user_id, content, likes
   - Filter: likes > 100

5. **Transaction History Query**
   - Schema: FinancialTransaction
   - Fields: amount, sender, recipient, timestamp
   - Filter: sender = "user123" OR recipient = "user123"

### 3.3 Sample Mutations

1. **Create User Mutation**
   - Schema: UserProfile
   - Type: Create
   - Data: Basic user information

2. **Update Product Mutation**
   - Schema: ProductCatalog
   - Type: Update
   - Data: Updated price and inventory

3. **Delete Blog Post Mutation**
   - Schema: BlogPost
   - Type: Delete
   - Data: Post identifier

4. **Create Social Post Mutation**
   - Schema: SocialMediaPost
   - Type: Create
   - Data: New post content

5. **Update Transaction Status Mutation**
   - Schema: FinancialTransaction
   - Type: Update
   - Data: New transaction status

## 4. Implementation Timeline

### Phase 1: Backend Enhancement (Weeks 1-2)

1. **Week 1: TCP Server Enhancement**
   - Implement HTTP request detection and handling
   - Create basic REST API endpoints
   - Set up static file serving

2. **Week 2: Sample Data Management**
   - Create sample data module structure
   - Implement sample loading and retrieval
   - Connect sample endpoints to TCP server

### Phase 2: Frontend Development (Weeks 3-4)

1. **Week 3: Core UI Components**
   - Enhance schema management UI
   - Implement query builder UI
   - Create mutation builder UI

2. **Week 4: Sample Library and Results Viewer**
   - Implement sample data library UI
   - Enhance results viewer
   - Add import/export functionality

### Phase 3: Integration and Testing (Week 5)

1. **Integration**
   - Connect frontend components to backend API
   - Implement error handling and loading states

2. **Testing**
   - Test with sample workflows
   - Fix bugs and optimize performance

3. **Documentation**
   - Create user guide
   - Add tooltips and help text

## 5. Conclusion

This implementation plan provides a roadmap for creating a user-friendly UI for DataFold that allows loading schemas, mutations, and queries with a clean interface and Rust backend. By enhancing the existing TCP server and improving the frontend UI, we will create a more intuitive and efficient user experience while maintaining compatibility with the existing codebase.