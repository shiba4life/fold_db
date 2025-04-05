import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

const Dashboard = ({ 
  isClientRunning, 
  startFoldClient, 
  stopFoldClient, 
  privateKey: propPrivateKey, 
  nodeConfig, 
  registeredApps, 
  runningApps 
}) => {
  // Use local state to ensure we have the private key
  const [localPrivateKey, setLocalPrivateKey] = useState(propPrivateKey);
  
  // Load private key directly when component mounts
  useEffect(() => {
    const loadPrivateKey = async () => {
      if (window.api && !localPrivateKey) {
        try {
          console.log('Dashboard: Directly getting private key');
          const privateKeyData = await window.api.getPrivateKey();
          console.log('Dashboard: Got private key:', privateKeyData);
          
          if (privateKeyData && privateKeyData.path && privateKeyData.content) {
            console.log('Dashboard: Setting local private key state');
            setLocalPrivateKey(privateKeyData);
          }
        } catch (error) {
          console.error('Dashboard: Error getting private key:', error);
        }
      }
    };
    
    loadPrivateKey();
  }, []);
  
  // Update local state when prop changes
  useEffect(() => {
    console.log('Dashboard: propPrivateKey changed:', propPrivateKey);
    if (propPrivateKey) {
      setLocalPrivateKey(propPrivateKey);
    }
  }, [propPrivateKey]);
  return (
    <div className="dashboard">
      <h1 className="mb-4">Dashboard</h1>
      
      <div className="row">
        <div className="col-md-6">
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-server me-2"></i>
              FoldClient Status
            </div>
            <div className="card-body">
              <div className="d-flex align-items-center mb-3">
                <span className={`status-indicator ${isClientRunning ? 'running' : 'stopped'} me-2`}></span>
                <h5 className="mb-0">{isClientRunning ? 'Running' : 'Stopped'}</h5>
              </div>
              
              {isClientRunning ? (
                <button 
                  className="btn btn-danger" 
                  onClick={stopFoldClient}
                >
                  <i className="fas fa-stop me-2"></i>
                  Stop FoldClient
                </button>
              ) : (
                <button 
                  className="btn btn-success" 
                  onClick={startFoldClient}
                  disabled={!localPrivateKey || !nodeConfig.node_tcp_address}
                >
                  <i className="fas fa-play me-2"></i>
                  Start FoldClient
                </button>
              )}
              
              {!localPrivateKey && (
                <div className="alert alert-warning mt-3">
                  <i className="fas fa-exclamation-triangle me-2"></i>
                  No private key loaded. <Link to="/private-key">Add a private key</Link> to start FoldClient.
                </div>
              )}
              
              {!nodeConfig.node_tcp_address && (
                <div className="alert alert-warning mt-3">
                  <i className="fas fa-exclamation-triangle me-2"></i>
                  No node connection configured. <Link to="/node-connection">Configure node connection</Link> to start FoldClient.
                </div>
              )}
            </div>
          </div>
          
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-key me-2"></i>
              Private Key
            </div>
            <div className="card-body">
              {localPrivateKey ? (
                <div>
                  <p><strong>File:</strong> {localPrivateKey.path}</p>
                  <div className="private-key-display">
                    {localPrivateKey.content.substring(0, 100)}...
                  </div>
                  <Link to="/private-key" className="btn btn-outline-primary mt-3">
                    <i className="fas fa-edit me-2"></i>
                    Manage Private Key
                  </Link>
                </div>
              ) : (
                <div>
                  <p>No private key loaded.</p>
                  <Link to="/private-key" className="btn btn-primary">
                    <i className="fas fa-plus me-2"></i>
                    Add Private Key
                  </Link>
                </div>
              )}
            </div>
          </div>
        </div>
        
        <div className="col-md-6">
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-network-wired me-2"></i>
              Node Connection
            </div>
            <div className="card-body">
              {nodeConfig.node_tcp_address ? (
                <div>
                  <p><strong>Host:</strong> {nodeConfig.node_tcp_address[0]}</p>
                  <p><strong>Port:</strong> {nodeConfig.node_tcp_address[1]}</p>
                  <Link to="/node-connection" className="btn btn-outline-primary">
                    <i className="fas fa-edit me-2"></i>
                    Manage Connection
                  </Link>
                </div>
              ) : (
                <div>
                  <p>No node connection configured.</p>
                  <Link to="/node-connection" className="btn btn-primary">
                    <i className="fas fa-plus me-2"></i>
                    Configure Connection
                  </Link>
                </div>
              )}
            </div>
          </div>
          
          <div className="card mb-4">
            <div className="card-header">
              <i className="fas fa-cubes me-2"></i>
              Sandboxed Apps
            </div>
            <div className="card-body">
              <div className="mb-3">
                <h6>Registered Apps: {registeredApps.length}</h6>
                <h6>Running Apps: {runningApps.length}</h6>
              </div>
              
              <Link to="/sandboxed-apps" className="btn btn-primary">
                <i className="fas fa-cubes me-2"></i>
                Manage Sandboxed Apps
              </Link>
            </div>
          </div>
        </div>
      </div>
      
      {runningApps.length > 0 && (
        <div className="card mb-4">
          <div className="card-header">
            <i className="fas fa-play-circle me-2"></i>
            Running Apps
          </div>
          <div className="card-body">
            <ul className="app-list">
              {runningApps.map(app => (
                <li key={app.id} className="app-item">
                  <div className="app-item-header">
                    <h5 className="app-item-title">{app.name}</h5>
                    <span className="badge bg-success">Running</span>
                  </div>
                  <div className="app-item-details">
                    <p className="app-item-detail"><strong>ID:</strong> {app.id}</p>
                    <p className="app-item-detail"><strong>Program:</strong> {app.program}</p>
                    <p className="app-item-detail"><strong>Launched:</strong> {app.launchedAt.toLocaleString()}</p>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
};

export default Dashboard;
