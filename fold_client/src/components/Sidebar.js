import React from 'react';
import { NavLink } from 'react-router-dom';

const Sidebar = ({ isClientRunning }) => {
  return (
    <div className="sidebar">
      <div className="p-3">
        <h3 className="text-light">FoldClient UI</h3>
        <p className="text-light-50">
          <span className={`status-indicator ${isClientRunning ? 'running' : 'stopped'}`}></span>
          {isClientRunning ? 'Running' : 'Stopped'}
        </p>
      </div>
      
      <ul className="nav flex-column p-3">
        <li className="nav-item">
          <NavLink to="/" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-tachometer-alt"></i> Dashboard
          </NavLink>
        </li>
        <li className="nav-item">
          <NavLink to="/private-key" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-key"></i> Private Key
          </NavLink>
        </li>
        <li className="nav-item">
          <NavLink to="/node-connection" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-network-wired"></i> Node Connection
          </NavLink>
        </li>
        <li className="nav-item">
          <NavLink to="/sandboxed-apps" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-cubes"></i> Sandboxed Apps
          </NavLink>
        </li>
        <li className="nav-item">
          <NavLink to="/settings" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-cog"></i> Settings
          </NavLink>
        </li>
        <li className="nav-item">
          <NavLink to="/logs" className={({ isActive }) => `nav-link ${isActive ? 'active' : ''}`}>
            <i className="fas fa-list"></i> Logs
          </NavLink>
        </li>
      </ul>
      
      <div className="mt-auto p-3">
        <div className="text-light-50 small">
          <p>FoldClient v1.0.0</p>
          <p>© 2025 DataFold</p>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
