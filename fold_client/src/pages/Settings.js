import React, { useState, useEffect } from 'react';

const Settings = ({ nodeConfig, updateNodeConfig }) => {
  const [appSocketDir, setAppSocketDir] = useState(nodeConfig.app_socket_dir || '');
  const [appDataDir, setAppDataDir] = useState(nodeConfig.app_data_dir || '');
  const [allowNetworkAccess, setAllowNetworkAccess] = useState(nodeConfig.allow_network_access || false);
  const [allowFilesystemAccess, setAllowFilesystemAccess] = useState(nodeConfig.allow_filesystem_access || false);
  const [maxMemoryMb, setMaxMemoryMb] = useState(nodeConfig.max_memory_mb || 1024);
  const [maxCpuPercent, setMaxCpuPercent] = useState(nodeConfig.max_cpu_percent || 50);
  
  // Update local state when nodeConfig changes
  useEffect(() => {
    setAppSocketDir(nodeConfig.app_socket_dir || '');
    setAppDataDir(nodeConfig.app_data_dir || '');
    setAllowNetworkAccess(nodeConfig.allow_network_access || false);
    setAllowFilesystemAccess(nodeConfig.allow_filesystem_access || false);
    setMaxMemoryMb(nodeConfig.max_memory_mb || 1024);
    setMaxCpuPercent(nodeConfig.max_cpu_percent || 50);
  }, [nodeConfig]);
  
  const handleSave = () => {
    updateNodeConfig({
      app_socket_dir: appSocketDir,
      app_data_dir: appDataDir,
      allow_network_access: allowNetworkAccess,
      allow_filesystem_access: allowFilesystemAccess,
      max_memory_mb: parseInt(maxMemoryMb, 10),
      max_cpu_percent: parseInt(maxCpuPercent, 10)
    });
  };
  
  return (
    <div className="settings">
      <h1 className="mb-4">Settings</h1>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-cog me-2"></i>
          FoldClient Settings
        </div>
        <div className="card-body">
          <form>
            <div className="mb-3">
              <label htmlFor="appSocketDir" className="form-label">App Socket Directory</label>
              <input 
                type="text" 
                className="form-control" 
                id="appSocketDir" 
                value={appSocketDir} 
                onChange={(e) => setAppSocketDir(e.target.value)}
                placeholder="Leave empty for default"
              />
              <div className="form-text">
                Directory where app sockets will be created. Leave empty to use the default.
              </div>
            </div>
            
            <div className="mb-3">
              <label htmlFor="appDataDir" className="form-label">App Data Directory</label>
              <input 
                type="text" 
                className="form-control" 
                id="appDataDir" 
                value={appDataDir} 
                onChange={(e) => setAppDataDir(e.target.value)}
                placeholder="Leave empty for default"
              />
              <div className="form-text">
                Directory where app data will be stored. Leave empty to use the default.
              </div>
            </div>
            
            <div className="mb-3 form-check">
              <input 
                type="checkbox" 
                className="form-check-input" 
                id="allowNetworkAccess" 
                checked={allowNetworkAccess} 
                onChange={(e) => setAllowNetworkAccess(e.target.checked)}
              />
              <label className="form-check-label" htmlFor="allowNetworkAccess">
                Allow Network Access
              </label>
              <div className="form-text">
                Allow sandboxed apps to access the network directly.
              </div>
            </div>
            
            <div className="mb-3 form-check">
              <input 
                type="checkbox" 
                className="form-check-input" 
                id="allowFilesystemAccess" 
                checked={allowFilesystemAccess} 
                onChange={(e) => setAllowFilesystemAccess(e.target.checked)}
              />
              <label className="form-check-label" htmlFor="allowFilesystemAccess">
                Allow Filesystem Access
              </label>
              <div className="form-text">
                Allow sandboxed apps to access the file system outside their working directory.
              </div>
            </div>
            
            <div className="mb-3">
              <label htmlFor="maxMemoryMb" className="form-label">Maximum Memory (MB)</label>
              <input 
                type="number" 
                className="form-control" 
                id="maxMemoryMb" 
                value={maxMemoryMb} 
                onChange={(e) => setMaxMemoryMb(e.target.value)}
                min="128"
              />
              <div className="form-text">
                Maximum memory usage for sandboxed apps in megabytes.
              </div>
            </div>
            
            <div className="mb-3">
              <label htmlFor="maxCpuPercent" className="form-label">Maximum CPU (%)</label>
              <input 
                type="number" 
                className="form-control" 
                id="maxCpuPercent" 
                value={maxCpuPercent} 
                onChange={(e) => setMaxCpuPercent(e.target.value)}
                min="1"
                max="100"
              />
              <div className="form-text">
                Maximum CPU usage for sandboxed apps as a percentage.
              </div>
            </div>
            
            <button 
              type="button" 
              className="btn btn-primary" 
              onClick={handleSave}
            >
              <i className="fas fa-save me-2"></i>
              Save Settings
            </button>
          </form>
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-info-circle me-2"></i>
          About Settings
        </div>
        <div className="card-body">
          <h5>App Socket Directory</h5>
          <p>
            The directory where app sockets will be created. This is used for communication between the FoldClient and sandboxed apps.
            If left empty, the default directory will be used.
          </p>
          
          <h5>App Data Directory</h5>
          <p>
            The directory where app data will be stored. This includes app registrations, tokens, and other data.
            If left empty, the default directory will be used.
          </p>
          
          <h5>Allow Network Access</h5>
          <p>
            If enabled, sandboxed apps will be allowed to access the network directly. This is disabled by default for security reasons.
            Only enable this if you trust the apps you're running.
          </p>
          
          <h5>Allow Filesystem Access</h5>
          <p>
            If enabled, sandboxed apps will be allowed to access the file system outside their working directory. This is disabled by default for security reasons.
            Only enable this if you trust the apps you're running.
          </p>
          
          <h5>Maximum Memory</h5>
          <p>
            The maximum amount of memory that sandboxed apps can use, in megabytes. This helps prevent memory exhaustion attacks.
          </p>
          
          <h5>Maximum CPU</h5>
          <p>
            The maximum amount of CPU that sandboxed apps can use, as a percentage. This helps prevent CPU exhaustion attacks.
          </p>
          
          <div className="alert alert-warning">
            <i className="fas fa-exclamation-triangle me-2"></i>
            Changing these settings will only affect newly launched apps. Existing apps will continue to use the settings they were launched with.
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;
