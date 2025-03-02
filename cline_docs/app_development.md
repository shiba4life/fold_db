# Datafold App Development Guide

## Table of Contents
1. [App Architecture Implementation](#app-architecture-implementation)
2. [Building a Sample Social App](#building-a-sample-social-app)
3. [Cross-App Communication](#cross-app-communication)

## App Architecture Implementation

The App-based approach treats extensions as full-fledged applications with their own windows and lifecycle.

### 1. App Registry System

```rust
// App manifest type definition
struct AppManifest {
    name: String,
    version: String,
    description: String,
    entry: String,
    schemas: Vec<String>,
    window: WindowConfig,
    permissions: AppPermissions,
    apis: ApiRequirements,
    payments: Option<PaymentConfig>,
}

struct WindowConfig {
    default_size: Size,
    min_size: Size,
    title: String,
    icon: Option<String>,
    resizable: bool,
}

// App registration in Datafold
impl DataFoldNode {
    pub fn register_app(&mut self, manifest: AppManifest) -> Result<(), Error> {
        // Validate manifest
        // Load schemas
        // Configure window
        // Setup permissions
        // Register API access
        // Setup payment handlers
    }
}
```

### 2. Schema Registration

Schema registration for apps defines the data structures they can access and manipulate:

```json
{
  "name": "social-app",
  "schemas": {
    "user-profile": {
      "fields": {
        "username": {"type": "string", "unique": true},
        "bio": {"type": "string"},
        "friends": {"type": "array", "items": {"type": "ref", "schema": "user-profile"}},
        "posts": {"type": "array", "items": {"type": "ref", "schema": "social-post"}}
      }
    },
    "social-post": {
      "fields": {
        "content": {"type": "string"},
        "author": {"type": "ref", "schema": "user-profile"},
        "likes": {"type": "array", "items": {"type": "ref", "schema": "user-profile"}},
        "comments": {"type": "array", "items": {"type": "ref", "schema": "post-comment"}}
      }
    }
  }
}
```

### 3. Window Management

```javascript
// App window management
class AppWindow {
    constructor(config) {
        this.window = window.open('', config.title, this.getWindowFeatures(config));
        this.setupMessageHandling();
        this.initializeAPIs(config.apis);
    }

    getWindowFeatures(config) {
        return `width=${config.defaultSize.width},height=${config.defaultSize.height},` +
               `resizable=${config.resizable},menubar=no,toolbar=no`;
    }

    setupMessageHandling() {
        this.messagePort = new MessageChannel();
        this.window.postMessage('init-message-port', '*', [this.messagePort.port2]);
        
        this.messagePort.port1.onmessage = (event) => {
            // Handle messages from the app
            this.handleAppMessage(event.data);
        };
    }

    initializeAPIs(apis) {
        // Create API proxies for the app
        const apiProxies = {};
        
        for (const api of apis) {
            apiProxies[api] = this.createAPIProxy(api);
        }
        
        // Inject API proxies into the app window
        this.window.postMessage({
            type: 'init-apis',
            apis: apiProxies
        }, '*');
    }
}
```

### 4. API Access

```rust
// API access control
struct ApiRequirements {
    required: Vec<String>,
    optional: Vec<String>,
}

// API proxy implementation
impl ApiManager {
    pub fn create_app_api_context(&self, app_name: &str, apis: &ApiRequirements) -> Result<ApiContext, Error> {
        let mut context = ApiContext::new(app_name);
        
        // Add required APIs
        for api in &apis.required {
            if !self.is_api_available(api) {
                return Err(Error::ApiNotAvailable(api.clone()));
            }
            context.add_api(api, self.get_api_proxy(api)?);
        }
        
        // Add optional APIs if available
        for api in &apis.optional {
            if self.is_api_available(api) {
                context.add_api(api, self.get_api_proxy(api)?);
            }
        }
        
        Ok(context)
    }
}
```

### 5. Resource Management

```rust
// Resource allocation for apps
struct AppResourceManager {
    allocations: HashMap<String, ResourceAllocation>,
    limits: HashMap<String, ResourceLimits>,
}

struct ResourceAllocation {
    memory: usize,
    cpu: f64,
    storage: usize,
    bandwidth: usize,
}

impl AppResourceManager {
    pub fn allocate_resources(&mut self, app_name: &str, resources: ResourceAllocation) -> Result<(), Error> {
        // Check against system capacity
        // Check against app limits
        // Allocate resources
        // Monitor usage
    }
    
    pub fn release_resources(&mut self, app_name: &str) -> Result<(), Error> {
        // Release allocated resources
        // Update system capacity
    }
}
```

## Building a Sample Social App

### 1. App Structure

```
social-app/
├── manifest.json
├── index.html
├── schemas/
│   ├── user-profile.json
│   ├── post.json
│   └── comment.json
├── src/
│   ├── app.js
│   ├── components/
│   │   ├── Feed.js
│   │   ├── Profile.js
│   │   └── Comments.js
│   ├── styles/
│   │   └── main.css
│   └── utils/
│       └── api.js
└── assets/
    └── images/
```

### 2. Implementation Steps

1. **Create App Manifest**

```json
{
  "name": "social-app",
  "version": "1.0.0",
  "description": "Social networking app for Datafold",
  "entry": "/apps/social-app/index.html",
  "schemas": [
    "user-profile",
    "post",
    "comment"
  ],
  "window": {
    "defaultSize": { "width": 800, "height": 600 },
    "minSize": { "width": 400, "height": 300 },
    "title": "Social App",
    "resizable": true
  },
  "permissions": {
    "required": [
      "read:profiles",
      "write:posts"
    ]
  },
  "apis": {
    "required": ["data", "users"],
    "optional": ["notifications"]
  }
}
```

2. **Implement Core Features**

```javascript
// social-app/src/app.js
class SocialApp {
    constructor() {
        // Initialize app when APIs are available
        window.addEventListener('message', (event) => {
            if (event.data.type === 'init-apis') {
                this.apis = event.data.apis;
                this.initialize();
            }
        });
    }

    initialize() {
        this.setupRouter();
        this.setupEventListeners();
        this.renderInitialView();
    }

    async createPost(content) {
        const post = await this.apis.data.execute({
            schema: 'post',
            operation: 'create',
            data: { content }
        });
        
        this.refreshFeed();
        
        // Notify other apps
        window.parent.postMessage({
            type: 'app-event',
            event: 'post:created',
            data: post
        }, '*');
    }

    async loadFeed() {
        return await this.apis.data.execute({
            schema: 'post',
            operation: 'query',
            filter: { order: 'timestamp:desc' }
        });
    }
}
```

3. **UI Implementation**

```html
<!-- social-app/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Social App</title>
    <link rel="stylesheet" href="src/styles/main.css">
</head>
<body>
    <div id="app">
        <header>
            <h1>Social App</h1>
            <nav>
                <button id="feed-btn">Feed</button>
                <button id="profile-btn">Profile</button>
                <button id="friends-btn">Friends</button>
            </nav>
        </header>
        
        <main id="content">
            <!-- App content will be rendered here -->
        </main>
    </div>
    
    <script src="src/app.js"></script>
    <script src="src/components/Feed.js"></script>
    <script src="src/components/Profile.js"></script>
    <script src="src/components/Comments.js"></script>
    <script>
        // Initialize app
        const app = new SocialApp();
    </script>
</body>
</html>
```

4. **API Integration**

```javascript
// social-app/src/utils/api.js
class ApiClient {
    constructor(apis) {
        this.apis = apis;
    }
    
    async getProfile(userId) {
        return await this.apis.users.getProfile(userId);
    }
    
    async updateProfile(profile) {
        return await this.apis.data.execute({
            schema: 'user-profile',
            operation: 'update',
            id: profile.id,
            data: profile
        });
    }
    
    async getFriendSuggestions() {
        return await this.apis.users.getSuggestions();
    }
}
```

### 3. App Registration

```rust
impl DataFoldNode {
    pub fn load_social_app(&mut self) -> Result<(), Error> {
        // Load manifest
        let manifest = load_manifest("social-app/manifest.json")?;
        
        // Register app
        self.register_app(manifest)?;
        
        // Allocate resources
        self.app_resource_manager.allocate_resources(
            "social-app",
            ResourceAllocation {
                memory: 50 * 1024 * 1024, // 50 MB
                cpu: 25.0,                // 25% of one CPU core
                storage: 5 * 1024 * 1024, // 5 MB
                bandwidth: 512 * 1024,    // 512 KB/s
            }
        )?;
        
        Ok(())
    }
}
```

## Cross-App Communication

### 1. Message System

```rust
// Message type definition
struct AppMessage {
    source: String,
    target: Option<String>, // None for broadcast
    message_type: String,
    payload: Value,
    timestamp: DateTime<Utc>,
}

// Message router implementation
struct MessageRouter {
    handlers: HashMap<String, Vec<Box<dyn Fn(AppMessage) + Send + Sync>>>,
}

impl MessageRouter {
    pub fn register_handler(
        &mut self,
        app_name: &str,
        message_type: &str,
        handler: Box<dyn Fn(AppMessage) + Send + Sync>
    ) {
        self.handlers
            .entry(format!("{}:{}", app_name, message_type))
            .or_default()
            .push(handler);
    }

    pub fn route_message(&self, message: AppMessage) {
        // Route to specific app if target is specified
        if let Some(target) = &message.target {
            let key = format!("{}:{}", target, message.message_type);
            if let Some(handlers) = self.handlers.get(&key) {
                for handler in handlers {
                    handler(message.clone());
                }
            }
        } else {
            // Broadcast to all apps that handle this message type
            for (key, handlers) in &self.handlers {
                if key.ends_with(&format!(":{}", message.message_type)) {
                    for handler in handlers {
                        handler(message.clone());
                    }
                }
            }
        }
    }
}
```

### 2. App-to-App Communication

```javascript
// App A: Sending a message
class AppA {
    sendMessage(targetApp, messageType, data) {
        window.parent.postMessage({
            type: 'app-message',
            target: targetApp,
            messageType: messageType,
            data: data
        }, '*');
    }
    
    broadcastMessage(messageType, data) {
        window.parent.postMessage({
            type: 'app-message',
            messageType: messageType,
            data: data
        }, '*');
    }
}

// App B: Receiving messages
class AppB {
    constructor() {
        window.addEventListener('message', this.handleMessage.bind(this));
    }
    
    handleMessage(event) {
        if (event.data.type === 'app-message') {
            const { messageType, data } = event.data;
            
            switch (messageType) {
                case 'content:shared':
                    this.handleSharedContent(data);
                    break;
                case 'user:action':
                    this.handleUserAction(data);
                    break;
            }
        }
    }
}
```

### 3. Shared Services

```javascript
// Shared service definition
class SharedService {
    constructor(name, methods) {
        this.name = name;
        this.methods = methods;
        
        // Register service with the system
        window.parent.postMessage({
            type: 'register-service',
            serviceName: name,
            methods: Object.keys(methods)
        }, '*');
        
        // Handle service calls
        window.addEventListener('message', (event) => {
            if (event.data.type === 'service-call' && 
                event.data.service === name) {
                this.handleServiceCall(event.data);
            }
        });
    }
    
    handleServiceCall(message) {
        const { method, args, callId } = message;
        
        if (this.methods[method]) {
            Promise.resolve(this.methods[method](...args))
                .then(result => {
                    window.parent.postMessage({
                        type: 'service-response',
                        callId: callId,
                        result: result
                    }, '*');
                })
                .catch(error => {
                    window.parent.postMessage({
                        type: 'service-error',
                        callId: callId,
                        error: error.message
                    }, '*');
                });
        } else {
            window.parent.postMessage({
                type: 'service-error',
                callId: callId,
                error: `Method ${method} not found`
            }, '*');
        }
    }
}

// Service usage from another app
class ServiceConsumer {
    constructor() {
        this.nextCallId = 1;
        this.pendingCalls = new Map();
        
        window.addEventListener('message', (event) => {
            if (event.data.type === 'service-response' || 
                event.data.type === 'service-error') {
                this.handleServiceResponse(event.data);
            }
        });
    }
    
    callService(service, method, ...args) {
        return new Promise((resolve, reject) => {
            const callId = this.nextCallId++;
            
            this.pendingCalls.set(callId, { resolve, reject });
            
            window.parent.postMessage({
                type: 'service-call',
                service: service,
                method: method,
                args: args,
                callId: callId
            }, '*');
        });
    }
    
    handleServiceResponse(message) {
        const { callId, result, error } = message;
        
        if (this.pendingCalls.has(callId)) {
            const { resolve, reject } = this.pendingCalls.get(callId);
            this.pendingCalls.delete(callId);
            
            if (message.type === 'service-response') {
                resolve(result);
            } else {
                reject(new Error(error));
            }
        }
    }
}
```

### 4. Security Considerations

- Messages are validated against app permissions
- Service calls are authenticated and authorized
- Apps run in separate windows with controlled communication
- Resource limits are enforced per app
- Data access is controlled through API proxies

### 5. Example: Cross-App Workflow

```javascript
// Social app sharing content with analytics app
class SocialApp {
    async createPost(content) {
        const post = await this.apis.data.execute({
            schema: 'post',
            operation: 'create',
            data: { content }
        });

        // Broadcast event for other apps
        window.parent.postMessage({
            type: 'app-event',
            event: 'content:created',
            data: {
                type: 'post',
                id: post.id,
                timestamp: new Date()
            }
        }, '*');
    }
}

class AnalyticsApp {
    constructor() {
        window.addEventListener('message', (event) => {
            if (event.data.type === 'app-event' && 
                event.data.event === 'content:created') {
                this.trackContent(event.data.data);
            }
        });
    }

    async trackContent(data) {
        await this.apis.data.execute({
            schema: 'analytics',
            operation: 'create',
            data: {
                contentId: data.id,
                contentType: data.type,
                timestamp: data.timestamp
            }
        });
        
        this.updateAnalyticsDashboard();
    }
}
```

## App Architecture Benefits

### UI Experience
- Full window control with complete UI freedom
- Application-based approach rather than component-based
- Independent styling without conflicts
- Full window real estate for rich interfaces
- Native-like window experience

### Development Experience
- Standard web app development workflow
- Any framework works natively without adapters
- Standard testing tools and methodologies
- Full developer tools access
- Isolated state management

### Resource Management
- Independent memory allocation
- Cleaner resource boundaries
- Flexible resource allocation based on app needs
- Dedicated CPU allocation
- Configurable storage limits

### Communication
- Message passing and service-based architecture
- API-based data access with clear boundaries
- Clearer security boundaries
- Direct window messaging
- Rich inter-app communication options

### Security
- Process isolation through separate windows
- Cleaner security model
- Natural browser sandbox
- Standard origin-based security
- Simpler permission model

### When to Use Apps
- When you need full control over the user experience
- For complex, full-featured applications
- When independent windows provide a better UX
- When resource isolation is important
- When extensions need their own navigation and routing
