# React Integration Tutorial

Build a complete React application with DataFold signature authentication. This tutorial covers React hooks, context providers, error handling, and real-world patterns.

## 🎯 What You'll Build

A production-ready React application featuring:
- 🔐 **Authentication Context** - Global authentication state management
- 🪝 **Custom Hooks** - Reusable authentication logic
- 🛡️ **Protected Routes** - Route guards for authenticated content
- ⚡ **Automatic Token Management** - Seamless key management
- 🔄 **Error Handling** - Graceful authentication failure recovery
- 📊 **Real-time Status** - Authentication status monitoring

## ⏱️ Estimated Time: 30 minutes

## 🛠️ Prerequisites

- Node.js 16+ and npm/yarn
- Basic React knowledge (hooks, context, components)
- Completed [5-Minute Integration](../../quickstart/5-minute-integration.md)

## 🚀 Step 1: Project Setup

### Create React App
```bash
# Create new React app with TypeScript
npx create-react-app datafold-auth-demo --template typescript
cd datafold-auth-demo

# Install DataFold SDK
npm install @datafold/sdk

# Install additional dependencies
npm install react-router-dom @types/react-router-dom
```

### Project Structure
```
src/
├── components/
│   ├── AuthStatus.tsx
│   ├── ProtectedRoute.tsx
│   └── SchemaManager.tsx
├── contexts/
│   └── AuthContext.tsx
├── hooks/
│   ├── useAuth.tsx
│   └── useDataFold.tsx
├── services/
│   └── datafold.ts
└── App.tsx
```

## 🔧 Step 2: Authentication Service

Create the core authentication service:

```typescript
// src/services/datafold.ts
import { DataFoldClient, generateKeyPair, KeyPair } from '@datafold/sdk';

export interface AuthConfig {
  serverUrl: string;
  clientId: string;
  keyPair: KeyPair;
}

export class DataFoldService {
  private client: DataFoldClient | null = null;
  private config: AuthConfig | null = null;

  async initialize(serverUrl: string): Promise<AuthConfig> {
    try {
      // Generate new keypair
      const keyPair = await generateKeyPair();
      
      // Generate unique client ID
      const clientId = `react-app-${Date.now()}`;
      
      // Register public key
      const response = await fetch(`${serverUrl}/api/crypto/keys/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          client_id: clientId,
          public_key: keyPair.publicKey,
          key_name: 'React App Key',
          metadata: {
            framework: 'react',
            environment: process.env.NODE_ENV,
            timestamp: new Date().toISOString()
          }
        })
      });

      if (!response.ok) {
        throw new Error(`Registration failed: ${response.statusText}`);
      }

      const registration = await response.json();
      
      this.config = {
        serverUrl,
        clientId: registration.data.client_id,
        keyPair
      };

      // Create authenticated client
      this.client = new DataFoldClient({
        serverUrl,
        clientId: this.config.clientId,
        privateKey: keyPair.privateKey
      });

      return this.config;
    } catch (error) {
      console.error('DataFold initialization failed:', error);
      throw error;
    }
  }

  getClient(): DataFoldClient {
    if (!this.client) {
      throw new Error('DataFold not initialized. Call initialize() first.');
    }
    return this.client;
  }

  getConfig(): AuthConfig | null {
    return this.config;
  }

  async testConnection(): Promise<boolean> {
    try {
      const client = this.getClient();
      await client.get('/api/system/status');
      return true;
    } catch (error) {
      console.error('Connection test failed:', error);
      return false;
    }
  }

  // Schema operations
  async getSchemas() {
    const client = this.getClient();
    const response = await client.get('/api/schemas');
    return response.data;
  }

  async createSchema(name: string, fields: any[]) {
    const client = this.getClient();
    const response = await client.post('/api/schemas', {
      name,
      fields,
      version: '1.0.0'
    });
    return response.data;
  }

  async deleteSchema(name: string) {
    const client = this.getClient();
    await client.delete(`/api/schemas/${name}`);
  }
}

// Singleton instance
export const datafoldService = new DataFoldService();
```

## 🎭 Step 3: Authentication Context

Create React context for global authentication state:

```typescript
// src/contexts/AuthContext.tsx
import React, { createContext, useContext, useReducer, useEffect, ReactNode } from 'react';
import { datafoldService, AuthConfig } from '../services/datafold';

interface AuthState {
  isInitialized: boolean;
  isAuthenticated: boolean;
  isLoading: boolean;
  config: AuthConfig | null;
  error: string | null;
}

type AuthAction = 
  | { type: 'INIT_START' }
  | { type: 'INIT_SUCCESS'; config: AuthConfig }
  | { type: 'INIT_FAILURE'; error: string }
  | { type: 'RESET' };

const initialState: AuthState = {
  isInitialized: false,
  isAuthenticated: false,
  isLoading: false,
  config: null,
  error: null
};

function authReducer(state: AuthState, action: AuthAction): AuthState {
  switch (action.type) {
    case 'INIT_START':
      return { ...state, isLoading: true, error: null };
    case 'INIT_SUCCESS':
      return {
        ...state,
        isInitialized: true,
        isAuthenticated: true,
        isLoading: false,
        config: action.config,
        error: null
      };
    case 'INIT_FAILURE':
      return {
        ...state,
        isInitialized: false,
        isAuthenticated: false,
        isLoading: false,
        error: action.error
      };
    case 'RESET':
      return initialState;
    default:
      return state;
  }
}

interface AuthContextType extends AuthState {
  initialize: (serverUrl: string) => Promise<void>;
  reset: () => void;
  testConnection: () => Promise<boolean>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(authReducer, initialState);

  const initialize = async (serverUrl: string) => {
    dispatch({ type: 'INIT_START' });
    
    try {
      const config = await datafoldService.initialize(serverUrl);
      dispatch({ type: 'INIT_SUCCESS', config });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      dispatch({ type: 'INIT_FAILURE', error: errorMessage });
    }
  };

  const reset = () => {
    dispatch({ type: 'RESET' });
  };

  const testConnection = async (): Promise<boolean> => {
    return datafoldService.testConnection();
  };

  // Auto-initialize with default server URL if provided
  useEffect(() => {
    const defaultServerUrl = process.env.REACT_APP_DATAFOLD_SERVER_URL;
    if (defaultServerUrl && !state.isInitialized && !state.isLoading) {
      initialize(defaultServerUrl);
    }
  }, [state.isInitialized, state.isLoading]);

  const value: AuthContextType = {
    ...state,
    initialize,
    reset,
    testConnection
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextType {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
```

## 🪝 Step 4: Custom Hooks

Create reusable hooks for DataFold operations:

```typescript
// src/hooks/useDataFold.tsx
import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { datafoldService } from '../services/datafold';

export function useSchemas() {
  const { isAuthenticated } = useAuth();
  const [schemas, setSchemas] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSchemas = useCallback(async () => {
    if (!isAuthenticated) return;

    setLoading(true);
    setError(null);
    
    try {
      const data = await datafoldService.getSchemas();
      setSchemas(data);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch schemas';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  const createSchema = useCallback(async (name: string, fields: any[]) => {
    setLoading(true);
    setError(null);
    
    try {
      const newSchema = await datafoldService.createSchema(name, fields);
      setSchemas(prev => [...prev, newSchema]);
      return newSchema;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create schema';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteSchema = useCallback(async (name: string) => {
    setLoading(true);
    setError(null);
    
    try {
      await datafoldService.deleteSchema(name);
      setSchemas(prev => prev.filter(s => s.name !== name));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete schema';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSchemas();
  }, [fetchSchemas]);

  return {
    schemas,
    loading,
    error,
    refetch: fetchSchemas,
    createSchema,
    deleteSchema
  };
}

export function useConnectionStatus() {
  const { isAuthenticated, testConnection } = useAuth();
  const [isConnected, setIsConnected] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(false);

  const checkConnection = useCallback(async () => {
    if (!isAuthenticated) {
      setIsConnected(false);
      return;
    }

    setChecking(true);
    try {
      const connected = await testConnection();
      setIsConnected(connected);
    } catch (error) {
      setIsConnected(false);
    } finally {
      setChecking(false);
    }
  }, [isAuthenticated, testConnection]);

  useEffect(() => {
    checkConnection();
    
    // Check connection periodically
    const interval = setInterval(checkConnection, 30000); // Every 30 seconds
    return () => clearInterval(interval);
  }, [checkConnection]);

  return {
    isConnected,
    checking,
    checkConnection
  };
}
```

## 🛡️ Step 5: Protected Routes Component

Create a component to protect authenticated routes:

```typescript
// src/components/ProtectedRoute.tsx
import React, { ReactNode } from 'react';
import { useAuth } from '../contexts/AuthContext';

interface ProtectedRouteProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export function ProtectedRoute({ children, fallback }: ProtectedRouteProps) {
  const { isAuthenticated, isLoading, error } = useAuth();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-2 text-gray-600">Initializing authentication...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Authentication Error</h3>
        <p className="text-red-600 mt-1">{error}</p>
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      fallback || (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <h3 className="text-yellow-800 font-medium">Authentication Required</h3>
          <p className="text-yellow-600 mt-1">Please configure DataFold authentication to access this content.</p>
        </div>
      )
    );
  }

  return <>{children}</>;
}
```

## 📊 Step 6: Authentication Status Component

Create a component to display authentication status:

```typescript
// src/components/AuthStatus.tsx
import React, { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useConnectionStatus } from '../hooks/useDataFold';

export function AuthStatus() {
  const { isAuthenticated, isLoading, config, error, initialize, reset } = useAuth();
  const { isConnected, checking, checkConnection } = useConnectionStatus();
  const [serverUrl, setServerUrl] = useState('https://api.datafold.com');

  const getStatusColor = () => {
    if (isLoading || checking) return 'text-yellow-600';
    if (error) return 'text-red-600';
    if (isAuthenticated && isConnected) return 'text-green-600';
    return 'text-gray-600';
  };

  const getStatusText = () => {
    if (isLoading) return 'Initializing...';
    if (checking) return 'Checking connection...';
    if (error) return `Error: ${error}`;
    if (isAuthenticated && isConnected) return 'Connected and authenticated';
    if (isAuthenticated && isConnected === false) return 'Authenticated but disconnected';
    return 'Not authenticated';
  };

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-medium text-gray-900">DataFold Authentication</h3>
        <div className="flex items-center space-x-2">
          <div className={`w-3 h-3 rounded-full ${
            isAuthenticated && isConnected ? 'bg-green-400' : 
            error ? 'bg-red-400' : 'bg-gray-400'
          }`}></div>
          <span className={`text-sm font-medium ${getStatusColor()}`}>
            {getStatusText()}
          </span>
        </div>
      </div>

      {!isAuthenticated && !isLoading && (
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Server URL
            </label>
            <input
              type="url"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="https://api.datafold.com"
            />
          </div>
          <button
            onClick={() => initialize(serverUrl)}
            disabled={isLoading}
            className="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Initialize Authentication
          </button>
        </div>
      )}

      {isAuthenticated && config && (
        <div className="space-y-2 text-sm">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <span className="font-medium text-gray-700">Client ID:</span>
              <span className="ml-2 text-gray-600">{config.clientId}</span>
            </div>
            <div>
              <span className="font-medium text-gray-700">Server:</span>
              <span className="ml-2 text-gray-600">{config.serverUrl}</span>
            </div>
          </div>
          
          <div className="flex space-x-2 mt-3">
            <button
              onClick={checkConnection}
              disabled={checking}
              className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 disabled:opacity-50"
            >
              {checking ? 'Checking...' : 'Test Connection'}
            </button>
            <button
              onClick={reset}
              className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200"
            >
              Reset
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
```

## 📋 Step 7: Schema Manager Component

Create a component to manage schemas:

```typescript
// src/components/SchemaManager.tsx
import React, { useState } from 'react';
import { useSchemas } from '../hooks/useDataFold';

export function SchemaManager() {
  const { schemas, loading, error, createSchema, deleteSchema, refetch } = useSchemas();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newSchemaName, setNewSchemaName] = useState('');

  const handleCreateSchema = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newSchemaName.trim()) return;

    try {
      await createSchema(newSchemaName, [
        { name: 'id', type: 'string', required: true },
        { name: 'timestamp', type: 'datetime', required: true },
        { name: 'data', type: 'json', required: false }
      ]);
      setNewSchemaName('');
      setShowCreateForm(false);
    } catch (error) {
      // Error is handled by the hook
    }
  };

  const handleDeleteSchema = async (name: string) => {
    if (window.confirm(`Are you sure you want to delete schema "${name}"?`)) {
      try {
        await deleteSchema(name);
      } catch (error) {
        // Error is handled by the hook
      }
    }
  };

  if (loading && schemas.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
        <p className="mt-2 text-gray-600">Loading schemas...</p>
      </div>
    );
  }

  return (
    <div className="bg-white border border-gray-200 rounded-lg shadow-sm">
      <div className="px-4 py-3 border-b border-gray-200 flex items-center justify-between">
        <h3 className="text-lg font-medium text-gray-900">Schema Management</h3>
        <div className="flex space-x-2">
          <button
            onClick={refetch}
            disabled={loading}
            className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 disabled:opacity-50"
          >
            {loading ? 'Refreshing...' : 'Refresh'}
          </button>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Create Schema
          </button>
        </div>
      </div>

      {error && (
        <div className="p-4 bg-red-50 border-l-4 border-red-400">
          <p className="text-red-700">{error}</p>
        </div>
      )}

      {showCreateForm && (
        <div className="p-4 border-b border-gray-200 bg-gray-50">
          <form onSubmit={handleCreateSchema} className="flex space-x-2">
            <input
              type="text"
              value={newSchemaName}
              onChange={(e) => setNewSchemaName(e.target.value)}
              placeholder="Schema name (e.g., user_events)"
              className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              type="submit"
              disabled={loading || !newSchemaName.trim()}
              className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
            >
              Create
            </button>
            <button
              type="button"
              onClick={() => setShowCreateForm(false)}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400"
            >
              Cancel
            </button>
          </form>
        </div>
      )}

      <div className="p-4">
        {schemas.length === 0 ? (
          <p className="text-gray-500 text-center py-4">No schemas found. Create your first schema to get started.</p>
        ) : (
          <div className="space-y-2">
            {schemas.map((schema) => (
              <div key={schema.name} className="flex items-center justify-between p-3 border border-gray-200 rounded-md">
                <div>
                  <h4 className="font-medium text-gray-900">{schema.name}</h4>
                  <p className="text-sm text-gray-600">
                    {schema.fields?.length || 0} fields • Version {schema.version || '1.0.0'}
                  </p>
                </div>
                <button
                  onClick={() => handleDeleteSchema(schema.name)}
                  disabled={loading}
                  className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200 disabled:opacity-50"
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
```

## 🏠 Step 8: Main App Component

Put it all together in the main App component:

```typescript
// src/App.tsx
import React from 'react';
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { AuthStatus } from './components/AuthStatus';
import { ProtectedRoute } from './components/ProtectedRoute';
import { SchemaManager } from './components/SchemaManager';
import './App.css';

function App() {
  return (
    <AuthProvider>
      <Router>
        <div className="min-h-screen bg-gray-50">
          <nav className="bg-white shadow-sm border-b border-gray-200">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex justify-between h-16">
                <div className="flex items-center">
                  <h1 className="text-xl font-semibold text-gray-900">DataFold React Demo</h1>
                </div>
                <div className="flex items-center space-x-4">
                  <Link to="/" className="text-gray-700 hover:text-gray-900">Home</Link>
                  <Link to="/schemas" className="text-gray-700 hover:text-gray-900">Schemas</Link>
                </div>
              </div>
            </div>
          </nav>

          <main className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/schemas" element={
                <ProtectedRoute>
                  <SchemasPage />
                </ProtectedRoute>
              } />
            </Routes>
          </main>
        </div>
      </Router>
    </AuthProvider>
  );
}

function HomePage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-4">DataFold React Integration Demo</h2>
        <p className="text-gray-600 mb-6">
          This demo showcases DataFold signature authentication integrated into a React application.
          Initialize authentication below to get started.
        </p>
      </div>

      <AuthStatus />

      <div className="bg-white border border-gray-200 rounded-lg p-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">Features Demonstrated</h3>
        <ul className="space-y-2 text-gray-600">
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            Automatic key generation and registration
          </li>
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            React Context for global authentication state
          </li>
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            Custom hooks for DataFold operations
          </li>
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            Protected routes with authentication guards
          </li>
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            Real-time connection status monitoring
          </li>
          <li className="flex items-center">
            <span className="w-2 h-2 bg-green-400 rounded-full mr-3"></span>
            Error handling and graceful degradation
          </li>
        </ul>
      </div>
    </div>
  );
}

function SchemasPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-2">Schema Management</h2>
        <p className="text-gray-600">
          Manage your DataFold schemas with full CRUD operations.
        </p>
      </div>

      <SchemaManager />
    </div>
  );
}

export default App;
```

## 🎨 Step 9: Styling (Optional)

Add Tailwind CSS for better styling:

```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

Update `tailwind.config.js`:
```javascript
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

Add to `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

## ✅ Step 10: Testing Your Integration

### Manual Testing
```bash
# Start the development server
npm start

# The app should open at http://localhost:3000
```

### Test Flow
1. **Initialize Authentication** - Enter your DataFold server URL and click "Initialize"
2. **Check Status** - Verify the green status indicator appears
3. **Test Connection** - Click "Test Connection" to verify connectivity
4. **Navigate to Schemas** - Go to `/schemas` to test protected routes
5. **Create Schema** - Test CRUD operations on schemas
6. **Test Error Handling** - Try invalid operations to see error handling

### Automated Testing
```typescript
// src/App.test.tsx
import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from './App';

// Mock the DataFold SDK
jest.mock('@datafold/sdk', () => ({
  generateKeyPair: jest.fn(() => Promise.resolve({
    privateKey: 'mock-private-key',
    publicKey: 'mock-public-key'
  })),
  DataFoldClient: jest.fn().mockImplementation(() => ({
    get: jest.fn(() => Promise.resolve({ data: [] })),
    post: jest.fn(() => Promise.resolve({ data: {} })),
    delete: jest.fn(() => Promise.resolve())
  }))
}));

test('renders authentication status component', () => {
  render(<App />);
  expect(screen.getByText('DataFold Authentication')).toBeInTheDocument();
});

test('shows initialize button when not authenticated', () => {
  render(<App />);
  expect(screen.getByText('Initialize Authentication')).toBeInTheDocument();
});

test('protects schemas route when not authenticated', () => {
  render(<App />);
  userEvent.click(screen.getByText('Schemas'));
  expect(screen.getByText('Authentication Required')).toBeInTheDocument();
});
```

## 🚀 Production Considerations

### Environment Variables
```bash
# .env
REACT_APP_DATAFOLD_SERVER_URL=https://api.datafold.com
REACT_APP_ENVIRONMENT=production
```

### Error Boundaries
```typescript
// src/components/ErrorBoundary.tsx
import React, { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = { hasError: false };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Authentication error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
          <div className="text-center">
            <h1 className="text-2xl font-bold text-red-600 mb-4">Authentication Error</h1>
            <p className="text-gray-600 mb-4">
              Something went wrong with the authentication system.
            </p>
            <button 
              onClick={() => this.setState({ hasError: false })}
              className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
            >
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
```

### Performance Optimization
```typescript
// Optimize with React.memo for expensive components
export const SchemaManager = React.memo(function SchemaManager() {
  // Component implementation
});

// Use React.lazy for code splitting
const SchemaManager = React.lazy(() => import('./components/SchemaManager'));

// Wrap with Suspense
<Suspense fallback={<div>Loading...</div>}>
  <SchemaManager />
</Suspense>
```

## 🔧 Common Issues & Solutions

### Issue: CORS Errors
```typescript
// Development proxy in package.json
{
  "name": "datafold-react-demo",
  "proxy": "https://api.datafold.com",
  // ... rest of package.json
}
```

### Issue: Authentication State Persistence
```typescript
// Add to AuthContext for persistence
const saveAuthState = (config: AuthConfig) => {
  localStorage.setItem('datafold-auth', JSON.stringify({
    serverUrl: config.serverUrl,
    clientId: config.clientId,
    // Note: Never store private keys in localStorage in production!
  }));
};

const loadAuthState = (): Partial<AuthConfig> | null => {
  const saved = localStorage.getItem('datafold-auth');
  return saved ? JSON.parse(saved) : null;
};
```

### Issue: Memory Leaks
```typescript
// Cleanup in useEffect
useEffect(() => {
  const interval = setInterval(checkConnection, 30000);
  return () => clearInterval(interval); // Important cleanup
}, [checkConnection]);
```

## 🎯 Next Steps

### Advanced Features
- **[State Management](../../advanced/state-management-patterns.md)** - Redux/Zustand integration
- **[Offline Support](../../advanced/offline-authentication.md)** - Handle network interruptions
- **[Real-time Updates](../../advanced/realtime-integration.md)** - WebSocket authentication

### Deployment
- **[Docker Deployment](../../deployment/docker-integration-guide.md)** - Containerize your React app
- **[CI/CD Setup](../../deployment/ci-cd-integration-tutorial.md)** - Automated deployment pipeline

### Testing
- **[Advanced Testing](../../development/testing-authenticated-apps.md)** - Comprehensive testing strategies

---

🎉 **Congratulations!** You've built a production-ready React application with DataFold signature authentication. Your app now has secure, cryptographically-verified communication with DataFold APIs.

💡 **Pro Tips**:
- Always handle authentication errors gracefully
- Use TypeScript for better developer experience
- Implement proper loading states for better UX
- Never store private keys in browser storage in production
- Monitor authentication metrics in production