import React, { useState, useEffect } from 'react';
import { Routes, Route, NavLink, useNavigate } from 'react-router-dom';

// Import pages
import Dashboard from './pages/Dashboard';
import PrivateKeyManager from './pages/PrivateKeyManager';
import NodeConnection from './pages/NodeConnection';
import SandboxedApps from './pages/SandboxedApps';
import Settings from './pages/Settings';
import Logs from './pages/Logs';

// Import components
import Sidebar from './components/Sidebar';

const App = () => {
  const [logs, setLogs] = useState([]);
  const [isClientRunning, setIsClientRunning] = useState(false);
  const [privateKey, setPrivateKey] = useState(null);
  const [nodeConfig, setNodeConfig] = useState({
    node_tcp_address: ['127.0.0.1', 9000],
    app_socket_dir: '',
    app_data_dir: '',
    allow_network_access: false,
    allow_filesystem_access: false,
    max_memory_mb: 1024,
    max_cpu_percent: 50
  });
  const [registeredApps, setRegisteredApps] = useState([]);
  const [runningApps, setRunningApps] = useState([]);
  
  const navigate = useNavigate();

  // Debug function to check the current state
  const debugState = () => {
    console.log('Current state:', {
      privateKey,
      isClientRunning,
      nodeConfig,
      registeredApps,
      runningApps
    });
  };

  // Load private key directly
  useEffect(() => {
    const loadPrivateKey = async () => {
      console.log(`[${new Date().toISOString()}] App: Attempting to load private key directly`);
      console.log('App: window.api available:', !!window.api);
      
      if (window.api) {
        try {
          console.log(`[${new Date().toISOString()}] App: Directly getting private key`);
          const privateKeyData = await window.api.getPrivateKey();
          console.log(`[${new Date().toISOString()}] App: Got private key:`, privateKeyData);
          
          if (privateKeyData && privateKeyData.path && privateKeyData.content) {
            console.log(`[${new Date().toISOString()}] App: Setting private key state directly`);
            console.log('App: Private key data structure:', JSON.stringify({
              hasPath: !!privateKeyData.path,
              pathLength: privateKeyData.path ? privateKeyData.path.length : 0,
              hasContent: !!privateKeyData.content,
              contentLength: privateKeyData.content ? privateKeyData.content.length : 0
            }));
            
            setPrivateKey(privateKeyData);
            
            setLogs(prevLogs => [...prevLogs, { 
              type: 'info', 
              message: `Private key loaded from storage: ${privateKeyData.path}`, 
              timestamp: new Date() 
            }]);
          } else {
            console.log(`[${new Date().toISOString()}] App: No valid private key data returned from getPrivateKey`);
            if (privateKeyData) {
              console.log('App: Received data structure:', JSON.stringify({
                hasPath: !!privateKeyData.path,
                hasContent: !!privateKeyData.content,
                keys: Object.keys(privateKeyData)
              }));
            }
          }
        } catch (error) {
          console.error(`[${new Date().toISOString()}] App: Error getting private key:`, error);
          setLogs(prevLogs => [...prevLogs, { 
            type: 'error', 
            message: `Error loading private key: ${error.message}`, 
            timestamp: new Date() 
          }]);
        }
      } else {
        console.error(`[${new Date().toISOString()}] App: API not available for loading private key`);
      }
    };
    
    // Load the private key when the component mounts
    loadPrivateKey();
  }, []);

  // Set up event listeners for fold-client logs
  useEffect(() => {
    console.log(`[${new Date().toISOString()}] App: Setting up event listeners`);
    console.log('App: window.api available:', !!window.api);
    
    if (window.api) {
      console.log(`[${new Date().toISOString()}] App: API is available`);
      console.log('App: API methods:', Object.keys(window.api).join(', '));
      
      window.api.onFoldClientLog((data) => {
        console.log(`[${new Date().toISOString()}] App: Received log:`, data);
        setLogs(prevLogs => [...prevLogs, { type: 'info', message: data, timestamp: new Date() }]);
      });

      window.api.onFoldClientError((data) => {
        console.log(`[${new Date().toISOString()}] App: Received error:`, data);
        setLogs(prevLogs => [...prevLogs, { type: 'error', message: data, timestamp: new Date() }]);
      });

      window.api.onFoldClientStopped((data) => {
        console.log(`[${new Date().toISOString()}] App: Received stopped:`, data);
        setLogs(prevLogs => [...prevLogs, { 
          type: 'warning', 
          message: `FoldClient stopped with code ${data.code}`, 
          timestamp: new Date() 
        }]);
        setIsClientRunning(false);
      });

      // Listen for private key loaded from storage
      window.api.onLoadPrivateKey((privateKeyData) => {
        console.log(`[${new Date().toISOString()}] App: Private key loaded from storage (event):`);
        console.log('App: Private key data structure from event:', JSON.stringify({
          hasPath: !!privateKeyData?.path,
          pathLength: privateKeyData?.path ? privateKeyData.path.length : 0,
          hasContent: !!privateKeyData?.content,
          contentLength: privateKeyData?.content ? privateKeyData.content.length : 0,
          keys: privateKeyData ? Object.keys(privateKeyData) : []
        }));
        
        if (privateKeyData && privateKeyData.path && privateKeyData.content) {
          console.log(`[${new Date().toISOString()}] App: Setting private key state from event`);
          
          // Only update if we don't already have a private key or if it's different
          setPrivateKey(currentKey => {
            // If we already have this exact key, don't update
            if (currentKey && 
                currentKey.path === privateKeyData.path && 
                currentKey.content === privateKeyData.content) {
              console.log(`[${new Date().toISOString()}] App: Private key already loaded, skipping update`);
              return currentKey;
            }
            
            // Log the new key being loaded
            setLogs(prevLogs => [...prevLogs, { 
              type: 'info', 
              message: `Private key loaded from storage: ${privateKeyData.path}`, 
              timestamp: new Date() 
            }]);
            
            // Return the new key
            return {
              path: privateKeyData.path,
              content: privateKeyData.content
            };
          });
        } else {
          console.error(`[${new Date().toISOString()}] App: Invalid private key data received:`, privateKeyData);
          setLogs(prevLogs => [...prevLogs, { 
            type: 'error', 
            message: `Failed to load private key from storage: Invalid data`, 
            timestamp: new Date() 
          }]);
        }
      });
      
      // Set up test message listener
      window.api.testMessage((data) => {
        console.log(`[${new Date().toISOString()}] App: Test message received:`, data);
        setLogs(prevLogs => [...prevLogs, { 
          type: 'info', 
          message: `Test message received: ${JSON.stringify(data)}`, 
          timestamp: new Date() 
        }]);
        
        // If we receive a test message but don't have a private key, try to load it again
        if (!privateKey) {
          console.log(`[${new Date().toISOString()}] App: No private key loaded, trying again after test message`);
          window.api.getPrivateKey().then(key => {
            console.log(`[${new Date().toISOString()}] App: Retry got private key:`, key);
            if (key && key.path && key.content) {
              console.log(`[${new Date().toISOString()}] App: Setting private key from retry`);
              setPrivateKey(key);
            }
          }).catch(err => {
            console.error(`[${new Date().toISOString()}] App: Retry error:`, err);
          });
        }
      });
      
      // Debug: Check if event listeners were set up
      console.log(`[${new Date().toISOString()}] App: Event listeners set up`);
    } else {
      console.error(`[${new Date().toISOString()}] App: API is not available`);
    }

    return () => {
      console.log(`[${new Date().toISOString()}] App: Cleaning up event listeners`);
      if (window.api) {
        window.api.removeAllListeners();
      }
    };
  }, [privateKey]);
  
  // Debug effect to monitor privateKey state changes
  useEffect(() => {
    console.log('privateKey state changed:', privateKey);
  }, [privateKey]);

  // Start the FoldClient
  const startFoldClient = async () => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return;
    }

    try {
      const config = { ...nodeConfig };
      
      // Add private key if available
      if (privateKey) {
        config.private_key = privateKey.content;
      }

      const result = await window.api.startFoldClient(config);
      
      setLogs(prevLogs => [...prevLogs, { 
        type: result.success ? 'success' : 'error', 
        message: result.message, 
        timestamp: new Date() 
      }]);
      
      if (result.success) {
        setIsClientRunning(true);
      }
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error starting FoldClient: ${error.message}`, 
        timestamp: new Date() 
      }]);
    }
  };

  // Stop the FoldClient
  const stopFoldClient = async () => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return;
    }

    try {
      const result = await window.api.stopFoldClient();
      
      setLogs(prevLogs => [...prevLogs, { 
        type: result.success ? 'success' : 'error', 
        message: result.message, 
        timestamp: new Date() 
      }]);
      
      if (result.success) {
        setIsClientRunning(false);
      }
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error stopping FoldClient: ${error.message}`, 
        timestamp: new Date() 
      }]);
    }
  };

  // Register a new app
  const registerApp = async (name, permissions) => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return null;
    }

    if (!isClientRunning) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'FoldClient is not running', 
        timestamp: new Date() 
      }]);
      return null;
    }

    try {
      const result = await window.api.registerApp(name, permissions);
      
      setLogs(prevLogs => [...prevLogs, { 
        type: result.success ? 'success' : 'error', 
        message: result.message, 
        timestamp: new Date() 
      }]);
      
      if (result.success) {
        // Parse the app registration data from stdout
        const appData = parseAppRegistration(result.data);
        if (appData) {
          setRegisteredApps(prevApps => [...prevApps, appData]);
          return appData;
        }
      }
      
      return null;
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error registering app: ${error.message}`, 
        timestamp: new Date() 
      }]);
      return null;
    }
  };

  // Parse app registration data from stdout
  const parseAppRegistration = (stdout) => {
    try {
      // This is a simple parser that assumes the output format
      // In a real implementation, you would parse the actual output format
      const appIdMatch = stdout.match(/App registered with ID: ([a-zA-Z0-9-]+)/);
      if (appIdMatch && appIdMatch[1]) {
        return {
          id: appIdMatch[1],
          name: 'App', // This would come from the actual output
          permissions: [], // This would come from the actual output
          registeredAt: new Date()
        };
      }
      return null;
    } catch (error) {
      console.error('Error parsing app registration:', error);
      return null;
    }
  };

  // Launch an app
  const launchApp = async (appId, program, args = []) => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return false;
    }

    if (!isClientRunning) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'FoldClient is not running', 
        timestamp: new Date() 
      }]);
      return false;
    }

    try {
      const result = await window.api.launchApp(appId, program, args);
      
      setLogs(prevLogs => [...prevLogs, { 
        type: result.success ? 'success' : 'error', 
        message: result.message, 
        timestamp: new Date() 
      }]);
      
      if (result.success) {
        // Add to running apps
        const app = registeredApps.find(app => app.id === appId);
        if (app) {
          setRunningApps(prevApps => [...prevApps, {
            ...app,
            program,
            args,
            launchedAt: new Date()
          }]);
        }
        return true;
      }
      
      return false;
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error launching app: ${error.message}`, 
        timestamp: new Date() 
      }]);
      return false;
    }
  };

  // Terminate an app
  const terminateApp = async (appId) => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return false;
    }

    if (!isClientRunning) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'FoldClient is not running', 
        timestamp: new Date() 
      }]);
      return false;
    }

    try {
      const result = await window.api.terminateApp(appId);
      
      setLogs(prevLogs => [...prevLogs, { 
        type: result.success ? 'success' : 'error', 
        message: result.message, 
        timestamp: new Date() 
      }]);
      
      if (result.success) {
        // Remove from running apps
        setRunningApps(prevApps => prevApps.filter(app => app.id !== appId));
        return true;
      }
      
      return false;
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error terminating app: ${error.message}`, 
        timestamp: new Date() 
      }]);
      return false;
    }
  };

  // Select a private key file
  const selectPrivateKeyFile = async () => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return;
    }

    try {
      const result = await window.api.selectPrivateKeyFile();
      
      if (!result.canceled) {
        if (result.error) {
          setLogs(prevLogs => [...prevLogs, { 
            type: 'error', 
            message: `Error reading private key file: ${result.error}`, 
            timestamp: new Date() 
          }]);
        } else {
          setPrivateKey({
            path: result.filePath,
            content: result.fileContent
          });
          
          setLogs(prevLogs => [...prevLogs, { 
            type: 'success', 
            message: `Private key loaded from ${result.filePath}`, 
            timestamp: new Date() 
          }]);
        }
      }
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error selecting private key file: ${error.message}`, 
        timestamp: new Date() 
      }]);
    }
  };

  // Select a program file
  const selectProgramFile = async () => {
    if (!window.api) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: 'API not available', 
        timestamp: new Date() 
      }]);
      return null;
    }

    try {
      const result = await window.api.selectProgramFile();
      
      if (!result.canceled) {
        setLogs(prevLogs => [...prevLogs, { 
          type: 'success', 
          message: `Program selected: ${result.filePath}`, 
          timestamp: new Date() 
        }]);
        
        return result.filePath;
      }
      
      return null;
    } catch (error) {
      setLogs(prevLogs => [...prevLogs, { 
        type: 'error', 
        message: `Error selecting program file: ${error.message}`, 
        timestamp: new Date() 
      }]);
      return null;
    }
  };

  // Update node configuration
  const updateNodeConfig = (newConfig) => {
    setNodeConfig(prevConfig => ({ ...prevConfig, ...newConfig }));
  };

  return (
    <div className="app-container">
      <Sidebar isClientRunning={isClientRunning} />
      
      <div className="main-content">
        <Routes>
          <Route path="/" element={
            <Dashboard 
              isClientRunning={isClientRunning}
              startFoldClient={startFoldClient}
              stopFoldClient={stopFoldClient}
              privateKey={privateKey}
              nodeConfig={nodeConfig}
              registeredApps={registeredApps}
              runningApps={runningApps}
            />
          } />
          <Route path="/private-key" element={
            <PrivateKeyManager 
              privateKey={privateKey}
              selectPrivateKeyFile={selectPrivateKeyFile}
            />
          } />
          <Route path="/node-connection" element={
            <NodeConnection 
              nodeConfig={nodeConfig}
              updateNodeConfig={updateNodeConfig}
              isClientRunning={isClientRunning}
              startFoldClient={startFoldClient}
              stopFoldClient={stopFoldClient}
            />
          } />
          <Route path="/sandboxed-apps" element={
            <SandboxedApps 
              isClientRunning={isClientRunning}
              registeredApps={registeredApps}
              runningApps={runningApps}
              registerApp={registerApp}
              launchApp={launchApp}
              terminateApp={terminateApp}
              selectProgramFile={selectProgramFile}
            />
          } />
          <Route path="/settings" element={
            <Settings 
              nodeConfig={nodeConfig}
              updateNodeConfig={updateNodeConfig}
            />
          } />
          <Route path="/logs" element={
            <Logs logs={logs} />
          } />
        </Routes>
      </div>
    </div>
  );
};

export default App;
