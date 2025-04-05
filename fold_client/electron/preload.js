const { contextBridge, ipcRenderer } = require('electron');

console.log(`[${new Date().toISOString()}] Preload script starting`);

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
const apiObject = {
  // FoldClient operations
  startFoldClient: (config) => ipcRenderer.invoke('start-fold-client', config),
  stopFoldClient: () => ipcRenderer.invoke('stop-fold-client'),
  
  // App operations
  registerApp: (name, permissions) => ipcRenderer.invoke('register-app', { name, permissions }),
  launchApp: (appId, program, args) => ipcRenderer.invoke('launch-app', { appId, program, args }),
  terminateApp: (appId) => ipcRenderer.invoke('terminate-app', { appId }),
  
  // File operations
  selectPrivateKeyFile: () => ipcRenderer.invoke('select-private-key-file'),
  selectProgramFile: () => ipcRenderer.invoke('select-program-file'),
  getPrivateKey: () => {
    console.log(`[${new Date().toISOString()}] Preload: Invoking get-private-key`);
    return ipcRenderer.invoke('get-private-key');
  },
  
  // Event listeners
  onFoldClientLog: (callback) => ipcRenderer.on('fold-client-log', (_, data) => callback(data)),
  onFoldClientError: (callback) => ipcRenderer.on('fold-client-error', (_, data) => callback(data)),
  onFoldClientStopped: (callback) => ipcRenderer.on('fold-client-stopped', (_, data) => callback(data)),
  onLoadPrivateKey: (callback) => {
    console.log(`[${new Date().toISOString()}] Preload: Setting up load-private-key listener`);
    ipcRenderer.on('load-private-key', (_, data) => {
      console.log(`[${new Date().toISOString()}] Preload: Received load-private-key event`);
      console.log('Preload: Private key data structure:', JSON.stringify({
        hasPath: !!data.path,
        pathLength: data.path ? data.path.length : 0,
        hasContent: !!data.content,
        contentLength: data.content ? data.content.length : 0
      }));
      callback(data);
    });
  },
  
  // Test message for IPC debugging
  testMessage: (callback) => {
    console.log(`[${new Date().toISOString()}] Preload: Setting up test-message listener`);
    ipcRenderer.on('test-message', (_, data) => {
      console.log(`[${new Date().toISOString()}] Preload: Received test message:`, data);
      callback(data);
    });
  },
  
  // Remove event listeners
  removeAllListeners: () => {
    console.log(`[${new Date().toISOString()}] Preload: Removing all listeners`);
    ipcRenderer.removeAllListeners('fold-client-log');
    ipcRenderer.removeAllListeners('fold-client-error');
    ipcRenderer.removeAllListeners('fold-client-stopped');
    ipcRenderer.removeAllListeners('load-private-key');
    ipcRenderer.removeAllListeners('test-message');
  }
};

contextBridge.exposeInMainWorld('api', apiObject);

console.log(`[${new Date().toISOString()}] Preload script executed, exposed API:`, Object.keys(apiObject).join(', '));
