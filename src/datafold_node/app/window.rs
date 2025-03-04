use crate::error::{FoldDbError, FoldDbResult};
use crate::datafold_node::app::manifest::WindowConfig;
use crate::datafold_node::app::api::ApiContext;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Manages app windows
#[derive(Debug, Clone)]
pub struct AppWindow {
    /// Window ID
    pub id: String,
    
    /// Window title
    pub title: String,
    
    /// Window URL
    pub url: String,
    
    /// Whether the window is open
    pub is_open: bool,
    
    /// API context for the app
    pub api_context: Option<ApiContext>,
}

impl AppWindow {
    /// Creates a new app window
    pub fn new(config: &WindowConfig, entry_path: &str) -> FoldDbResult<Self> {
        // Validate entry path
        if !Path::new(entry_path).exists() {
            return Err(FoldDbError::Config(format!("Entry path '{}' does not exist", entry_path)));
        }
        
        // Generate window ID
        let id = format!("app-window-{}", uuid::Uuid::new_v4());
        
        // Create window
        Ok(Self {
            id,
            title: config.title.clone(),
            url: entry_path.to_string(),
            is_open: false,
            api_context: None,
        })
    }
    
    /// Opens the window
    pub fn open(&mut self) -> FoldDbResult<()> {
        // Check if window is already open
        if self.is_open {
            return Err(FoldDbError::Config("Window is already open".to_string()));
        }
        
        // In a real implementation, this would open a browser window or WebView
        // For now, we'll simulate it by injecting the necessary JavaScript into the app's HTML
        self.inject_api_initialization()?;
        
        // Set the window as open
        self.is_open = true;
        
        Ok(())
    }
    
    /// Closes the window
    pub fn close(&self) -> FoldDbResult<()> {
        // Check if window is open
        if !self.is_open {
            return Err(FoldDbError::Config("Window is not open".to_string()));
        }
        
        // In a real implementation, this would close the browser window or WebView
        // For now, we'll just return Ok
        
        Ok(())
    }
    
    /// Injects API initialization code into the app's HTML
    fn inject_api_initialization(&self) -> FoldDbResult<()> {
        // In a real implementation, this would inject JavaScript into the app's window
        // For now, we'll simulate it by modifying the app's HTML to include our initialization code
        
        // Create a mock API object that will be passed to the app
        let mock_apis = r#"
        {
            "data": {
                "execute": async function(params) {
                    console.log("Mock API call: data.execute", params);
                    
                    // Simulate API response based on operation type
                    if (params.type === "query" || params.operation === "query") {
                        // For queries, return mock data
                        if (params.schema === "post" || params.schema === "social-post") {
                            return {
                                results: [
                                    {
                                        id: "1",
                                        content: "Just launched my new app on Datafold!",
                                        author: { id: "1", username: "alice" },
                                        timestamp: new Date(Date.now() - 3600000).toISOString(),
                                        likes: [{ id: "2", username: "bob" }],
                                        comments: []
                                    },
                                    {
                                        id: "2",
                                        content: "Exploring the new Datafold app system. It's amazing!",
                                        author: { id: "2", username: "bob" },
                                        timestamp: new Date(Date.now() - 7200000).toISOString(),
                                        likes: [{ id: "1", username: "alice" }, { id: "3", username: "charlie" }],
                                        comments: [{ id: "1", author: { id: "1", username: "alice" }, content: "Totally agree!" }]
                                    }
                                ]
                            };
                        } else if (params.schema === "user-profile") {
                            return {
                                results: [
                                    {
                                        id: "1",
                                        username: "alice",
                                        bio: "Software developer and Datafold enthusiast"
                                    }
                                ]
                            };
                        }
                    } else if (params.type === "mutation" || params.operation === "create" || params.mutation_type === "create") {
                        // For mutations, return success with the created object
                        return {
                            id: Date.now().toString(),
                            ...params.data
                        };
                    }
                    
                    return { results: [] };
                }
            },
            "schema": {
                "list": async function() {
                    console.log("Mock API call: schema.list");
                    return ["post", "user-profile", "comment"];
                },
                "get": async function(name) {
                    console.log("Mock API call: schema.get", name);
                    return { name, fields: {} };
                }
            },
            "network": {
                "getNodes": async function() {
                    console.log("Mock API call: network.getNodes");
                    return [];
                }
            }
        }
        "#;
        
        // Simulate sending the init-apis message to the app
        // In a real implementation, this would be done through window.postMessage
        // For now, we'll add a script tag to the app's HTML that initializes the APIs
        
        // Read the app's HTML
        let html_path = Path::new(&self.url);
        let html_content = std::fs::read_to_string(html_path)
            .map_err(|e| FoldDbError::Config(format!("Failed to read app HTML: {}", e)))?;
        
        // Check if we've already injected the APIs
        if html_content.contains("// DataFold API Initialization") {
            return Ok(());
        }
        
        // Create the initialization script
        let init_script = format!(r#"
        <script>
        // DataFold API Initialization
        (function() {{
            // Initialize mock APIs
            window.apis = {};
            
            // Define the initialization function
            function initializeApis() {{
                console.log('Initializing DataFold APIs');
                
                // Directly set the APIs on the window object
                window.apis = {mock_apis};
                
                // Simulate receiving the init-apis message
                const event = new MessageEvent('message', {{
                    data: {{
                        type: 'init-apis',
                        apis: {mock_apis}
                    }}
                }});
                
                // Dispatch the event to initialize the app
                window.dispatchEvent(event);
                
                console.log('DataFold APIs initialized');
                
                // Check if the app was initialized
                setTimeout(function() {{
                    if (typeof window.initializeApp === 'function') {{
                        console.log('App has initializeApp function');
                    }} else {{
                        console.log('App does not have initializeApp function');
                    }}
                    
                    // Override the app's functions to add debugging
                    if (typeof window.showFeed === 'function') {{
                        const originalShowFeed = window.showFeed;
                        window.showFeed = function() {{
                            console.log('Overridden showFeed called');
                            return originalShowFeed.apply(this, arguments);
                        }};
                    }}
                    
                    if (typeof window.showProfile === 'function') {{
                        const originalShowProfile = window.showProfile;
                        window.showProfile = function() {{
                            console.log('Overridden showProfile called');
                            return originalShowProfile.apply(this, arguments);
                        }};
                    }}
                    
                    if (typeof window.showFriends === 'function') {{
                        const originalShowFriends = window.showFriends;
                        window.showFriends = function() {{
                            console.log('Overridden showFriends called');
                            return originalShowFriends.apply(this, arguments);
                        }};
                    }}
                    
                    if (typeof window.createPost === 'function') {{
                        const originalCreatePost = window.createPost;
                        window.createPost = function() {{
                            console.log('Overridden createPost called');
                            return originalCreatePost.apply(this, arguments);
                        }};
                    }}
                    
                    // Force the app to load the feed
                    if (typeof window.loadFeed === 'function') {{
                        console.log('Calling loadFeed directly');
                        window.loadFeed();
                    }}
                }}, 1000);
            }}
            
            // Call the initialization function immediately
            initializeApis();
            
            // Also wait for DOMContentLoaded to ensure the app's code is loaded
            document.addEventListener('DOMContentLoaded', function() {{
                console.log('DOMContentLoaded event fired');
                
                // Call the initialization function again to make sure it runs after the app's code is loaded
                initializeApis();
                
                // Add event listeners directly to the buttons
                setTimeout(function() {{
                    console.log('Adding event listeners directly to buttons');
                    
                    const feedBtn = document.getElementById('feed-btn');
                    if (feedBtn) {{
                        feedBtn.addEventListener('click', function() {{
                            console.log('Feed button clicked (direct)');
                            if (typeof window.showFeed === 'function') {{
                                window.showFeed();
                            }}
                        }});
                    }}
                    
                    const profileBtn = document.getElementById('profile-btn');
                    if (profileBtn) {{
                        profileBtn.addEventListener('click', function() {{
                            console.log('Profile button clicked (direct)');
                            if (typeof window.showProfile === 'function') {{
                                window.showProfile();
                            }}
                        }});
                    }}
                    
                    const friendsBtn = document.getElementById('friends-btn');
                    if (friendsBtn) {{
                        friendsBtn.addEventListener('click', function() {{
                            console.log('Friends button clicked (direct)');
                            if (typeof window.showFriends === 'function') {{
                                window.showFriends();
                            }}
                        }});
                    }}
                    
                    const postBtn = document.getElementById('post-btn');
                    if (postBtn) {{
                        postBtn.addEventListener('click', function() {{
                            console.log('Post button clicked (direct)');
                            if (typeof window.createPost === 'function') {{
                                window.createPost();
                            }}
                        }});
                    }}
                }}, 2000);
            }});
        }})();
        </script>
        "#, mock_apis);
        
        // Inject the initialization script into the HTML
        let modified_html = if let Some(head_end) = html_content.find("</head>") {
            // Insert before </head>
            let (before, after) = html_content.split_at(head_end);
            format!("{}{}{}", before, init_script, after)
        } else {
            // If no </head>, insert at the beginning
            format!("{}{}", init_script, html_content)
        };
        
        // Write the modified HTML back to the file
        std::fs::write(html_path, modified_html)
            .map_err(|e| FoldDbError::Config(format!("Failed to write modified app HTML: {}", e)))?;
        
        Ok(())
    }
    
    /// Gets the window features string for window.open()
    pub fn get_window_features(config: &WindowConfig) -> String {
        let mut features = Vec::new();
        
        // Add window size
        features.push(format!("width={}", config.default_size.width));
        features.push(format!("height={}", config.default_size.height));
        
        // Add resizable
        features.push(format!("resizable={}", config.resizable));
        
        // Add other features
        features.push("menubar=no".to_string());
        features.push("toolbar=no".to_string());
        
        // Join features
        features.join(",")
    }
    
    /// Creates JavaScript code to initialize the window
    pub fn create_init_script(&self) -> String {
        format!(r#"
            // Initialize message handling
            window.addEventListener('message', function(event) {{
                // Handle messages from the app
                if (event.data.type === 'init-message-port') {{
                    // Store message port
                    window.messagePort = event.ports[0];
                    
                    // Set up message handling
                    window.messagePort.onmessage = function(event) {{
                        // Handle messages from the app
                        handleAppMessage(event.data);
                    }};
                }}
            }});
            
            // Function to handle messages from the app
            function handleAppMessage(message) {{
                console.log('Received message from app:', message);
                
                // Handle different message types
                switch (message.type) {{
                    case 'api-call':
                        // Handle API call
                        handleApiCall(message);
                        break;
                    case 'app-event':
                        // Handle app event
                        handleAppEvent(message);
                        break;
                    default:
                        console.warn('Unknown message type:', message.type);
                }}
            }}
            
            // Function to handle API calls
            function handleApiCall(message) {{
                // Get API and method
                const api = message.api;
                const method = message.method;
                const args = message.args;
                const callId = message.callId;
                
                    // Check if API exists
                    if (!window.apis || !window.apis[api]) {{
                        // Send error response
                        window.messagePort.postMessage({{
                            type: 'api-response',
                            callId: callId,
                            error: 'API \'' + api + '\' not found'
                        }});
                        return;
                    }}
                    
                    // Check if method exists
                    if (!window.apis[api][method]) {{
                        // Send error response
                        window.messagePort.postMessage({{
                            type: 'api-response',
                            callId: callId,
                            error: 'Method \'' + method + '\' not found on API \'' + api + '\''
                        }});
                        return;
                    }}
                
                try {{
                    // Call method
                    const result = window.apis[api][method](...args);
                    
                    // Send response
                    window.messagePort.postMessage({{
                        type: 'api-response',
                        callId: callId,
                        result: result
                    }});
                }} catch (error) {{
                    // Send error response
                    window.messagePort.postMessage({{
                        type: 'api-response',
                        callId: callId,
                        error: error.message
                    }});
                }}
            }}
            
            // Function to handle app events
            function handleAppEvent(message) {{
                // Get event type and data
                const eventType = message.event;
                const eventData = message.data;
                
                // Dispatch event
                const event = new CustomEvent(eventType, {{ detail: eventData }});
                window.dispatchEvent(event);
            }}
        "#)
    }
    
    /// Creates JavaScript code to initialize APIs
    pub fn create_api_init_script(apis: &[String]) -> String {
        let api_list = apis.join("', '");
        
        format!(r#"
            // Initialize APIs
            window.apis = {{}};
            
            // Add API proxies
            const apiNames = ['{}'];
            
            for (const apiName of apiNames) {{
                window.apis[apiName] = createApiProxy(apiName);
            }}
            
            // Function to create API proxy
            function createApiProxy(apiName) {{
                return new Proxy({{}}, {{
                    get(target, method) {{
                        if (typeof method !== 'string' || method === 'then') {{
                            return undefined;
                        }}
                        
                        return (...args) => {{
                            return callApi(apiName, method, args);
                        }};
                    }}
                }});
            }}
            
            // Function to call API
            function callApi(api, method, args) {{
                return new Promise((resolve, reject) => {{
                    // Generate call ID
                    const callId = Date.now().toString() + Math.random().toString().substr(2);
                    
                    // Create response handler
                    const responseHandler = (event) => {{
                        const message = event.data;
                        
                        // Check if this is the response for our call
                        if (message.type === 'api-response' && message.callId === callId) {{
                            // Remove event listener
                            window.messagePort.removeEventListener('message', responseHandler);
                            
                            // Check for error
                            if (message.error) {{
                                reject(new Error(message.error));
                            }} else {{
                                resolve(message.result);
                            }}
                        }}
                    }};
                    
                    // Add response handler
                    window.messagePort.addEventListener('message', responseHandler);
                    
                    // Send API call
                    window.messagePort.postMessage({{
                        type: 'api-call',
                        api: api,
                        method: method,
                        args: args,
                        callId: callId
                    }});
                }});
            }}
        "#, api_list)
    }
    
    /// Opens the app in a browser
    pub fn open_in_browser(&self) -> FoldDbResult<()> {
        // Get the full URL
        let url = format!("http://127.0.0.1:8080/{}", self.url);
        
        // Open the URL in the default browser
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/c", "start", &url])
                .spawn()
                .map_err(|e| FoldDbError::Config(format!("Failed to open browser: {}", e)))?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&url)
                .spawn()
                .map_err(|e| FoldDbError::Config(format!("Failed to open browser: {}", e)))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()
                .map_err(|e| FoldDbError::Config(format!("Failed to open browser: {}", e)))?;
        }
        
        // Give the browser time to open
        thread::sleep(Duration::from_millis(500));
        
        Ok(())
    }
}
