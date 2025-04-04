const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld(
  'api', {
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
    
    // Event listeners
    onFoldClientLog: (callback) => ipcRenderer.on('fold-client-log', (_, data) => callback(data)),
    onFoldClientError: (callback) => ipcRenderer.on('fold-client-error', (_, data) => callback(data)),
    onFoldClientStopped: (callback) => ipcRenderer.on('fold-client-stopped', (_, data) => callback(data)),
    
    // Remove event listeners
    removeAllListeners: () => {
      ipcRenderer.removeAllListeners('fold-client-log');
      ipcRenderer.removeAllListeners('fold-client-error');
      ipcRenderer.removeAllListeners('fold-client-stopped');
    }
  }
);
