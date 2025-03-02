use crate::error::{FoldDbError, FoldDbResult};
use crate::datafold_node::app::manifest::WindowConfig;
use std::path::Path;

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
        })
    }
    
    /// Opens the window
    pub fn open(&mut self) -> FoldDbResult<()> {
        // Check if window is already open
        if self.is_open {
            return Err(FoldDbError::Config("Window is already open".to_string()));
        }
        
        // In a real implementation, this would open a browser window or WebView
        // For now, we'll just set the is_open flag
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
}
