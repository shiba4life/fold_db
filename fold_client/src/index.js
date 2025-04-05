import React from 'react';
import { createRoot } from 'react-dom/client';
import { HashRouter } from 'react-router-dom';
import App from './App';
import './assets/styles.css';

// Import Bootstrap CSS
import 'bootstrap/dist/css/bootstrap.min.css';
// Import FontAwesome CSS
import '@fortawesome/fontawesome-free/css/all.min.css';

// Debug: Log that the app is starting
console.log('Starting FoldClient UI...');

const container = document.getElementById('root');
const root = createRoot(container);

// Use HashRouter instead of BrowserRouter for Electron
root.render(
  <React.StrictMode>
    <HashRouter>
      <App />
    </HashRouter>
  </React.StrictMode>
);

// Debug: Log that the app has rendered
console.log('FoldClient UI rendered');
