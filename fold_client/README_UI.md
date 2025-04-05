# FoldClient Electron UI

This is an Electron-based UI for the FoldClient component of the DataFold project. The UI provides a graphical interface for managing private keys, connecting to DataFold nodes, and running sandboxed applications.

## Features

- **Private Key Management**: Add and manage private keys for authentication with DataFold nodes
- **Node Connection**: Configure and manage connections to DataFold nodes
- **Sandboxed Apps**: Register, launch, and manage sandboxed applications
- **Settings**: Configure FoldClient settings such as resource limits and permissions
- **Logs**: View logs of FoldClient operations

## Getting Started

### Prerequisites

- Node.js (v14 or later)
- npm (v6 or later)
- Rust (for building the fold_client binary)

### Installation

1. Build the fold_client binary:
   ```bash
   cargo build
   ```

2. Install dependencies:
   ```bash
   cd fold_client
   npm install
   ```

3. Start the application:
   ```bash
   npm start
   ```

## Development

### Project Structure

- `electron/`: Electron main process files
  - `main.js`: Main process entry point
  - `preload.js`: Preload script for secure IPC communication
- `src/`: React application files
  - `index.html`: HTML entry point
  - `index.js`: JavaScript entry point
  - `App.js`: Main React component
  - `components/`: Reusable UI components
  - `pages/`: Page components
  - `assets/`: Static assets like CSS and images

### Building

To build the application for production:

```bash
npm run build
```

### IPC Communication

The Electron main process communicates with the fold_client binary and exposes an API to the renderer process through IPC. The API is defined in `preload.js` and includes:

- `startFoldClient(config)`: Start the FoldClient with the given configuration
- `stopFoldClient()`: Stop the FoldClient
- `registerApp(name, permissions)`: Register a new sandboxed app
- `launchApp(appId, program, args)`: Launch a sandboxed app
- `terminateApp(appId)`: Terminate a running sandboxed app
- `selectPrivateKeyFile()`: Open a file dialog to select a private key file
- `selectProgramFile()`: Open a file dialog to select a program file

### Persistent Storage

The application stores the following data persistently:

- **Private Keys**: Private keys are stored in the application's user data directory and are loaded automatically when the application starts.

## Troubleshooting

### FoldClient Binary Not Found

If the application cannot find the fold_client binary, make sure you have built it with `cargo build`. The application looks for the binary in the following locations:

1. `fold_client/target/release/fold_client`
2. `fold_client/target/debug/fold_client`
3. `target/release/fold_client`
4. `target/debug/fold_client`

### FoldClient Not Starting

If the FoldClient fails to start, check the logs in the Logs page for error messages. Common issues include:

- Missing or invalid private key
- Invalid node connection configuration
- Permissions issues

## License

This project is licensed under the same license as the DataFold project.
