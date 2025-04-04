import React, { useState } from 'react';

const SandboxedApps = ({ 
  isClientRunning, 
  registeredApps, 
  runningApps, 
  registerApp, 
  launchApp, 
  terminateApp, 
  selectProgramFile 
}) => {
  const [newAppName, setNewAppName] = useState('');
  const [newAppPermissions, setNewAppPermissions] = useState([
    'list_schemas',
    'query',
    'mutation'
  ]);
  const [selectedAppId, setSelectedAppId] = useState('');
  const [programPath, setProgramPath] = useState('');
  const [programArgs, setProgramArgs] = useState('');
  
  const handleRegisterApp = async () => {
    if (!newAppName) {
      alert('Please enter an app name');
      return;
    }
    
    const app = await registerApp(newAppName, newAppPermissions);
    if (app) {
      setNewAppName('');
      setSelectedAppId(app.id);
    }
  };
  
  const handleSelectProgram = async () => {
    const path = await selectProgramFile();
    if (path) {
      setProgramPath(path);
    }
  };
  
  const handleLaunchApp = async () => {
    if (!selectedAppId) {
      alert('Please select an app');
      return;
    }
    
    if (!programPath) {
      alert('Please select a program');
      return;
    }
    
    const args = programArgs.split(' ').filter(arg => arg.trim() !== '');
    await launchApp(selectedAppId, programPath, args);
  };
  
  const handleTerminateApp = async (appId) => {
    await terminateApp(appId);
  };
  
  // Find if an app is running
  const isAppRunning = (appId) => {
    return runningApps.some(app => app.id === appId);
  };
  
  // Get running app details
  const getRunningApp = (appId) => {
    return runningApps.find(app => app.id === appId);
  };
  
  // Available permissions
  const availablePermissions = [
    { id: 'list_schemas', name: 'List Schemas' },
    { id: 'query', name: 'Query Data' },
    { id: 'mutation', name: 'Mutate Data' },
    { id: 'discover_nodes', name: 'Discover Nodes' },
    { id: 'query_remote', name: 'Query Remote Nodes' }
  ];
  
  // Toggle permission
  const togglePermission = (permissionId) => {
    if (newAppPermissions.includes(permissionId)) {
      setNewAppPermissions(newAppPermissions.filter(p => p !== permissionId));
    } else {
      setNewAppPermissions([...newAppPermissions, permissionId]);
    }
  };
  
  return (
    <div className="sandboxed-apps">
      <h1 className="mb-4">Sandboxed Apps</h1>
      
      <div className="row">
        <div className="col-md-6">
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-plus-circle me-2"></i>
              Register New App
            </div>
            <div className="card-body">
              {!isClientRunning ? (
                <div className="alert alert-warning">
                  <i className="fas fa-exclamation-triangle me-2"></i>
                  FoldClient is not running. Start FoldClient to register apps.
                </div>
              ) : (
                <form>
                  <div className="mb-3">
                    <label htmlFor="appName" className="form-label">App Name</label>
                    <input 
                      type="text" 
                      className="form-control" 
                      id="appName" 
                      value={newAppName} 
                      onChange={(e) => setNewAppName(e.target.value)}
                    />
                  </div>
                  
                  <div className="mb-3">
                    <label className="form-label">Permissions</label>
                    {availablePermissions.map(permission => (
                      <div className="form-check" key={permission.id}>
                        <input 
                          className="form-check-input" 
                          type="checkbox" 
                          id={`permission-${permission.id}`} 
                          checked={newAppPermissions.includes(permission.id)} 
                          onChange={() => togglePermission(permission.id)}
                        />
                        <label className="form-check-label" htmlFor={`permission-${permission.id}`}>
                          {permission.name}
                        </label>
                      </div>
                    ))}
                  </div>
                  
                  <button 
                    type="button" 
                    className="btn btn-primary" 
                    onClick={handleRegisterApp}
                  >
                    <i className="fas fa-plus-circle me-2"></i>
                    Register App
                  </button>
                </form>
              )}
            </div>
          </div>
        </div>
        
        <div className="col-md-6">
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-play-circle me-2"></i>
              Launch App
            </div>
            <div className="card-body">
              {!isClientRunning ? (
                <div className="alert alert-warning">
                  <i className="fas fa-exclamation-triangle me-2"></i>
                  FoldClient is not running. Start FoldClient to launch apps.
                </div>
              ) : (
                <form>
                  <div className="mb-3">
                    <label htmlFor="appId" className="form-label">Select App</label>
                    <select 
                      className="form-select" 
                      id="appId" 
                      value={selectedAppId} 
                      onChange={(e) => setSelectedAppId(e.target.value)}
                    >
                      <option value="">Select an app</option>
                      {registeredApps.map(app => (
                        <option key={app.id} value={app.id}>
                          {app.name} ({app.id})
                        </option>
                      ))}
                    </select>
                  </div>
                  
                  <div className="mb-3">
                    <label htmlFor="programPath" className="form-label">Program Path</label>
                    <div className="input-group">
                      <input 
                        type="text" 
                        className="form-control" 
                        id="programPath" 
                        value={programPath} 
                        onChange={(e) => setProgramPath(e.target.value)}
                        readOnly
                      />
                      <button 
                        className="btn btn-outline-secondary" 
                        type="button" 
                        onClick={handleSelectProgram}
                      >
                        <i className="fas fa-folder-open"></i>
                      </button>
                    </div>
                  </div>
                  
                  <div className="mb-3">
                    <label htmlFor="programArgs" className="form-label">Program Arguments</label>
                    <input 
                      type="text" 
                      className="form-control" 
                      id="programArgs" 
                      value={programArgs} 
                      onChange={(e) => setProgramArgs(e.target.value)}
                      placeholder="arg1 arg2 arg3"
                    />
                  </div>
                  
                  <button 
                    type="button" 
                    className="btn btn-success" 
                    onClick={handleLaunchApp}
                    disabled={!selectedAppId || !programPath}
                  >
                    <i className="fas fa-play-circle me-2"></i>
                    Launch App
                  </button>
                </form>
              )}
            </div>
          </div>
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-list me-2"></i>
          Registered Apps
        </div>
        <div className="card-body">
          {registeredApps.length === 0 ? (
            <div className="alert alert-info">
              <i className="fas fa-info-circle me-2"></i>
              No apps registered yet. Register an app to get started.
            </div>
          ) : (
            <ul className="app-list">
              {registeredApps.map(app => (
                <li key={app.id} className="app-item">
                  <div className="app-item-header">
                    <h5 className="app-item-title">{app.name}</h5>
                    <div className="app-item-actions">
                      {isAppRunning(app.id) ? (
                        <button 
                          className="btn btn-sm btn-danger" 
                          onClick={() => handleTerminateApp(app.id)}
                        >
                          <i className="fas fa-stop me-1"></i>
                          Terminate
                        </button>
                      ) : (
                        <button 
                          className="btn btn-sm btn-success" 
                          onClick={() => {
                            setSelectedAppId(app.id);
                            document.getElementById('appId').scrollIntoView({ behavior: 'smooth' });
                          }}
                        >
                          <i className="fas fa-play me-1"></i>
                          Launch
                        </button>
                      )}
                    </div>
                  </div>
                  <div className="app-item-details">
                    <p className="app-item-detail"><strong>ID:</strong> {app.id}</p>
                    <p className="app-item-detail"><strong>Status:</strong> 
                      <span className={`status-indicator ${isAppRunning(app.id) ? 'running' : 'stopped'} ms-2 me-1`}></span>
                      {isAppRunning(app.id) ? 'Running' : 'Stopped'}
                    </p>
                    {isAppRunning(app.id) && (
                      <>
                        <p className="app-item-detail"><strong>Program:</strong> {getRunningApp(app.id).program}</p>
                        <p className="app-item-detail"><strong>Launched:</strong> {getRunningApp(app.id).launchedAt.toLocaleString()}</p>
                      </>
                    )}
                    <p className="app-item-detail"><strong>Permissions:</strong> {app.permissions.join(', ') || 'None'}</p>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
};

export default SandboxedApps;
