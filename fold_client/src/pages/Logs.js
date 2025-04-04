import React, { useRef, useEffect } from 'react';

const Logs = ({ logs }) => {
  const logContainerRef = useRef(null);
  
  // Auto-scroll to the bottom when new logs are added
  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs]);
  
  // Format timestamp
  const formatTimestamp = (timestamp) => {
    return timestamp.toLocaleTimeString();
  };
  
  // Get CSS class for log type
  const getLogClass = (type) => {
    switch (type) {
      case 'error':
        return 'error';
      case 'warning':
        return 'warning';
      case 'success':
        return 'success';
      default:
        return '';
    }
  };
  
  // Get icon for log type
  const getLogIcon = (type) => {
    switch (type) {
      case 'error':
        return 'fas fa-times-circle';
      case 'warning':
        return 'fas fa-exclamation-triangle';
      case 'success':
        return 'fas fa-check-circle';
      default:
        return 'fas fa-info-circle';
    }
  };
  
  return (
    <div className="logs">
      <h1 className="mb-4">Logs</h1>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-list me-2"></i>
          FoldClient Logs
        </div>
        <div className="card-body">
          <div className="log-container" ref={logContainerRef}>
            {logs.length === 0 ? (
              <p className="text-muted">No logs yet.</p>
            ) : (
              logs.map((log, index) => (
                <p key={index} className={`log-entry ${getLogClass(log.type)}`}>
                  <span className="log-timestamp">[{formatTimestamp(log.timestamp)}]</span>
                  <i className={`${getLogIcon(log.type)} ms-2 me-2`}></i>
                  <span className="log-message">{log.message}</span>
                </p>
              ))
            )}
          </div>
        </div>
      </div>
      
      <div className="card mb-4">
        <div className="card-header">
          <i className="fas fa-info-circle me-2"></i>
          About Logs
        </div>
        <div className="card-body">
          <h5>Log Types</h5>
          <ul>
            <li><i className="fas fa-info-circle text-primary me-2"></i> <strong>Info:</strong> General information about FoldClient operations.</li>
            <li><i className="fas fa-check-circle text-success me-2"></i> <strong>Success:</strong> Successful operations.</li>
            <li><i className="fas fa-exclamation-triangle text-warning me-2"></i> <strong>Warning:</strong> Operations that completed with warnings.</li>
            <li><i className="fas fa-times-circle text-danger me-2"></i> <strong>Error:</strong> Failed operations.</li>
          </ul>
          
          <h5>Log Persistence</h5>
          <p>
            Logs are stored in memory and will be lost when the application is closed.
            For persistent logs, check the FoldClient log files in the app data directory.
          </p>
        </div>
      </div>
    </div>
  );
};

export default Logs;
