# FoldClient UI

An Electron-based user interface for the FoldClient, providing a graphical way to manage private keys, connect to DataFold nodes, and run sandboxed applications.

## Features

- **Private Key Management**: Add and manage private keys for authentication with DataFold nodes.
- **Node Connection**: Configure and manage connections to DataFold nodes.
- **Sandboxed Apps**: Register, launch, and manage sandboxed applications.
- **Settings**: Configure FoldClient settings such as resource limits and permissions.
- **Logs**: View logs of FoldClient operations.

## Getting Started

### Prerequisites

- Node.js (v14 or later)
- npm (v6 or later)
- Rust toolchain (for building the FoldClient binary)

### Installation

1. Build the FoldClient binary:

```bash
cd ..
cargo build --release
```

2. Install dependencies:

```bash
cd fold_client
npm install
```

3. Build the UI:

```bash
npm run webpack
```

4. Start the application:

```bash
npm start
```

## Usage

### Private Key Management

1. Navigate to the "Private Key" section.
2. Click "Select Private Key File" to choose a private key file.
3. The private key will be loaded and displayed.

### Node Connection

1. Navigate to the "Node Connection" section.
2. Configure the connection settings (TCP or Unix socket).
3. Click "Save Connection Settings" to save the configuration.
4. Click "Connect" to connect to the DataFold node.

### Sandboxed Apps

1. Navigate to the "Sandboxed Apps" section.
2. Register a new app by providing a name and selecting permissions.
3. Launch the app by selecting it from the list, choosing a program file, and optionally providing arguments.
4. Manage running apps by terminating them when needed.

### Settings

1. Navigate to the "Settings" section.
2. Configure FoldClient settings such as app socket directory, app data directory, network access, filesystem access, and resource limits.
3. Click "Save Settings" to apply the changes.

### Logs

1. Navigate to the "Logs" section.
2. View logs of FoldClient operations, including info, success, warning, and error messages.

## Development

### Project Structure

- `electron/`: Electron main process files
  - `main.js`: Main entry point for Electron
  - `preload.js`: Preload script for secure IPC communication
- `src/`: React application files
  - `components/`: Reusable React components
  - `pages/`: React components for each page
  - `assets/`: Static assets like CSS files
  - `utils/`: Utility functions
  - `index.js`: Entry point for the React application
  - `App.js`: Main React component

### Building

To build the application for production:

```bash
npm run build
```

This will create a production build in the `dist/` directory.

### Packaging

To package the application for distribution:

```bash
npm run build
npx electron-builder
```

This will create platform-specific packages in the `dist/` directory.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
