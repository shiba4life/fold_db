import React, { useState } from 'react';

const NodeConnection = ({ 
  nodeConfig, 
  updateNodeConfig, 
  isClientRunning,
  startFoldClient,
  stopFoldClient
}) => {
  const [host, setHost] = useState(nodeConfig.node_tcp_address ? nodeConfig.node_tcp_address[0] : '127.0.0.1');
  const [port, setPort] = useState(nodeConfig.node_tcp_address ? nodeConfig.node_tcp_address[1] : 9000);
  const [socketPath, setSocketPath] = useState(nodeConfig.node_socket_path || '');
  const [connectionType, setConnectionType] = useState(nodeConfig.node_socket_path ? 'unix' : 'tcp');
  
  const handleSave = () => {
    if (connectionType === 'tcp') {
      updateNodeConfig({
        node_tcp_address: [host, parseInt(port, 10)],
        node_socket_path: null
      });
    } else {
      updateNodeConfig({
        node_tcp_address: null,
        node_socket_path: socketPath
      });
    }
  };
  
  return (
    <div className="node-connection">
      <h1 className="mb-4">Node Connection</h1>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-network-wired me-2"></i>
          Connection Status
        </div>
        <div className="card-body">
          <div className="node-connection-status">
            <span className={`status-indicator ${isClientRunning ? 'running' : 'stopped'} me-2`}></span>
            <div>
              <h5 className="mb-0">{isClientRunning ? 'Connected' : 'Disconnected'}</h5>
              {isClientRunning && nodeConfig.node_tcp_address && (
                <p className="mb-0 text-muted">
                  Connected to {nodeConfig.node_tcp_address[0]}:{nodeConfig.node_tcp_address[1]}
                </p>
              )}
              {isClientRunning && nodeConfig.node_socket_path && (
                <p className="mb-0 text-muted">
                  Connected to {nodeConfig.node_socket_path}
                </p>
              )}
            </div>
          </div>
          
          {isClientRunning ? (
            <button 
              className="btn btn-danger" 
              onClick={stopFoldClient}
            >
              <i className="fas fa-stop me-2"></i>
              Disconnect
            </button>
          ) : (
            <button 
              className="btn btn-success" 
              onClick={startFoldClient}
              disabled={!nodeConfig.node_tcp_address && !nodeConfig.node_socket_path}
            >
              <i className="fas fa-play me-2"></i>
              Connect
            </button>
          )}
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-cog me-2"></i>
          Connection Settings
        </div>
        <div className="card-body">
          <form>
            <div className="mb-3">
              <label className="form-label">Connection Type</label>
              <div>
                <div className="form-check form-check-inline">
                  <input 
                    className="form-check-input" 
                    type="radio" 
                    name="connectionType" 
                    id="connectionTypeTcp" 
                    value="tcp" 
                    checked={connectionType === 'tcp'} 
                    onChange={() => setConnectionType('tcp')}
                    disabled={isClientRunning}
                  />
                  <label className="form-check-label" htmlFor="connectionTypeTcp">
                    TCP Socket
                  </label>
                </div>
                <div className="form-check form-check-inline">
                  <input 
                    className="form-check-input" 
                    type="radio" 
                    name="connectionType" 
                    id="connectionTypeUnix" 
                    value="unix" 
                    checked={connectionType === 'unix'} 
                    onChange={() => setConnectionType('unix')}
                    disabled={isClientRunning}
                  />
                  <label className="form-check-label" htmlFor="connectionTypeUnix">
                    Unix Socket
                  </label>
                </div>
              </div>
            </div>
            
            {connectionType === 'tcp' ? (
              <>
                <div className="mb-3">
                  <label htmlFor="host" className="form-label">Host</label>
                  <input 
                    type="text" 
                    className="form-control" 
                    id="host" 
                    value={host} 
                    onChange={(e) => setHost(e.target.value)}
                    disabled={isClientRunning}
                  />
                </div>
                <div className="mb-3">
                  <label htmlFor="port" className="form-label">Port</label>
                  <input 
                    type="number" 
                    className="form-control" 
                    id="port" 
                    value={port} 
                    onChange={(e) => setPort(e.target.value)}
                    disabled={isClientRunning}
                  />
                </div>
              </>
            ) : (
              <div className="mb-3">
                <label htmlFor="socketPath" className="form-label">Socket Path</label>
                <input 
                  type="text" 
                  className="form-control" 
                  id="socketPath" 
                  value={socketPath} 
                  onChange={(e) => setSocketPath(e.target.value)}
                  disabled={isClientRunning}
                />
              </div>
            )}
            
            <button 
              type="button" 
              className="btn btn-primary" 
              onClick={handleSave}
              disabled={isClientRunning}
            >
              <i className="fas fa-save me-2"></i>
              Save Connection Settings
            </button>
          </form>
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-info-circle me-2"></i>
          Connection Information
        </div>
        <div className="card-body">
          <h5>TCP Socket</h5>
          <p>
            Use TCP socket to connect to a DataFold node running on a remote machine.
            You need to specify the host (IP address or hostname) and port number.
          </p>
          
          <h5>Unix Socket</h5>
          <p>
            Use Unix socket to connect to a DataFold node running on the same machine.
            You need to specify the path to the socket file.
          </p>
          
          <div className="alert alert-info">
            <i className="fas fa-info-circle me-2"></i>
            For security reasons, it's recommended to use Unix socket when connecting to a local DataFold node.
          </div>
        </div>
      </div>
    </div>
  );
};

export default NodeConnection;
