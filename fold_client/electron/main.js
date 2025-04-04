const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

// Keep a global reference of the window object to prevent garbage collection
let mainWindow;

// Path to the fold_client binary
let foldClientBinaryPath = path.join(__dirname, '..', '..', 'fold_client', 'target', 'release', 'fold_client');

// Check if the binary exists, if not, use the debug version
if (!fs.existsSync(foldClientBinaryPath)) {
  console.log(`Release binary not found at ${foldClientBinaryPath}, trying debug version`);
  const debugPath = path.join(__dirname, '..', '..', 'fold_client', 'target', 'debug', 'fold_client');
  
  if (fs.existsSync(debugPath)) {
    console.log(`Using debug binary at ${debugPath}`);
    foldClientBinaryPath = debugPath;
  } else {
    console.log(`Debug binary not found at ${debugPath}, trying parent directory`);
    const parentReleasePath = path.join(__dirname, '..', '..', 'target', 'release', 'fold_client');
    
    if (fs.existsSync(parentReleasePath)) {
      console.log(`Using release binary at ${parentReleasePath}`);
      foldClientBinaryPath = parentReleasePath;
    } else {
      console.log(`Release binary not found at ${parentReleasePath}, trying parent debug version`);
      const parentDebugPath = path.join(__dirname, '..', '..', 'target', 'debug', 'fold_client');
      
      if (fs.existsSync(parentDebugPath)) {
        console.log(`Using debug binary at ${parentDebugPath}`);
        foldClientBinaryPath = parentDebugPath;
      } else {
        console.error('Could not find fold_client binary');
      }
    }
  }
}

// Store the fold_client process
let foldClientProcess = null;

// Create the browser window
function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  });

  // Load the index.html file
  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'));
    // Open DevTools in development mode
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'));
  }

  // Emitted when the window is closed
  mainWindow.on('closed', () => {
    mainWindow = null;
    stopFoldClient();
  });
}

// Start the fold_client process
function startFoldClient(config) {
  if (foldClientProcess) {
    console.log('FoldClient is already running');
    return { success: true, message: 'FoldClient is already running' };
  }

  try {
    // Create a temporary config file
    const configPath = path.join(app.getPath('temp'), 'fold_client_config.json');
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

    // Start the fold_client process
    foldClientProcess = spawn(foldClientBinaryPath, ['start', '--config', configPath]);

    // Handle stdout
    foldClientProcess.stdout.on('data', (data) => {
      console.log(`FoldClient stdout: ${data}`);
      if (mainWindow) {
        mainWindow.webContents.send('fold-client-log', data.toString());
      }
    });

    // Handle stderr
    foldClientProcess.stderr.on('data', (data) => {
      console.error(`FoldClient stderr: ${data}`);
      if (mainWindow) {
        mainWindow.webContents.send('fold-client-error', data.toString());
      }
    });

    // Handle process exit
    foldClientProcess.on('close', (code) => {
      console.log(`FoldClient process exited with code ${code}`);
      foldClientProcess = null;
      if (mainWindow) {
        mainWindow.webContents.send('fold-client-stopped', { code });
      }
    });

    return { success: true, message: 'FoldClient started successfully' };
  } catch (error) {
    console.error('Failed to start FoldClient:', error);
    return { success: false, message: `Failed to start FoldClient: ${error.message}` };
  }
}

// Stop the fold_client process
function stopFoldClient() {
  if (foldClientProcess) {
    foldClientProcess.kill();
    foldClientProcess = null;
    return { success: true, message: 'FoldClient stopped successfully' };
  } else {
    return { success: true, message: 'FoldClient is not running' };
  }
}

// Register an app with fold_client
async function registerApp(name, permissions) {
  if (!foldClientProcess) {
    return { success: false, message: 'FoldClient is not running' };
  }

  try {
    const registerProcess = spawn(foldClientBinaryPath, [
      'register-app',
      '--name', name,
      '--permissions', permissions.join(',')
    ]);

    return new Promise((resolve, reject) => {
      let stdout = '';
      let stderr = '';

      registerProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      registerProcess.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      registerProcess.on('close', (code) => {
        if (code === 0) {
          resolve({ success: true, message: 'App registered successfully', data: stdout });
        } else {
          reject({ success: false, message: `Failed to register app: ${stderr}` });
        }
      });
    });
  } catch (error) {
    return { success: false, message: `Failed to register app: ${error.message}` };
  }
}

// Launch an app with fold_client
async function launchApp(appId, program, args) {
  if (!foldClientProcess) {
    return { success: false, message: 'FoldClient is not running' };
  }

  try {
    const launchArgs = ['launch-app', '--id', appId, '--program', program];
    
    if (args && args.length > 0) {
      launchArgs.push('--args');
      launchArgs.push(...args);
    }

    const launchProcess = spawn(foldClientBinaryPath, launchArgs);

    return new Promise((resolve, reject) => {
      let stdout = '';
      let stderr = '';

      launchProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      launchProcess.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      launchProcess.on('close', (code) => {
        if (code === 0) {
          resolve({ success: true, message: 'App launched successfully', data: stdout });
        } else {
          reject({ success: false, message: `Failed to launch app: ${stderr}` });
        }
      });
    });
  } catch (error) {
    return { success: false, message: `Failed to launch app: ${error.message}` };
  }
}

// Terminate an app with fold_client
async function terminateApp(appId) {
  if (!foldClientProcess) {
    return { success: false, message: 'FoldClient is not running' };
  }

  try {
    const terminateProcess = spawn(foldClientBinaryPath, ['terminate-app', '--id', appId]);

    return new Promise((resolve, reject) => {
      let stdout = '';
      let stderr = '';

      terminateProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });

      terminateProcess.stderr.on('data', (data) => {
        stderr += data.toString();
      });

      terminateProcess.on('close', (code) => {
        if (code === 0) {
          resolve({ success: true, message: 'App terminated successfully', data: stdout });
        } else {
          reject({ success: false, message: `Failed to terminate app: ${stderr}` });
        }
      });
    });
  } catch (error) {
    return { success: false, message: `Failed to terminate app: ${error.message}` };
  }
}

// This method will be called when Electron has finished initialization
app.whenReady().then(() => {
  createWindow();

  // Set up IPC handlers
  ipcMain.handle('start-fold-client', (event, config) => {
    return startFoldClient(config);
  });

  ipcMain.handle('stop-fold-client', () => {
    return stopFoldClient();
  });

  ipcMain.handle('register-app', (event, { name, permissions }) => {
    return registerApp(name, permissions);
  });

  ipcMain.handle('launch-app', (event, { appId, program, args }) => {
    return launchApp(appId, program, args);
  });

  ipcMain.handle('terminate-app', (event, { appId }) => {
    return terminateApp(appId);
  });

  ipcMain.handle('select-private-key-file', async () => {
    const result = await dialog.showOpenDialog(mainWindow, {
      properties: ['openFile'],
      filters: [
        { name: 'Private Key Files', extensions: ['pem', 'key'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    });

    if (result.canceled) {
      return { canceled: true };
    }

    try {
      const filePath = result.filePaths[0];
      const fileContent = fs.readFileSync(filePath, 'utf8');
      return { canceled: false, filePath, fileContent };
    } catch (error) {
      return { canceled: false, error: error.message };
    }
  });

  ipcMain.handle('select-program-file', async () => {
    const result = await dialog.showOpenDialog(mainWindow, {
      properties: ['openFile'],
      filters: [
        { name: 'Executable Files', extensions: ['exe', ''] },
        { name: 'All Files', extensions: ['*'] }
      ]
    });

    if (result.canceled) {
      return { canceled: true };
    }

    return { canceled: false, filePath: result.filePaths[0] };
  });

  app.on('activate', () => {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// Quit when all windows are closed, except on macOS
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// On macOS it's common to re-create a window in the app when the
// dock icon is clicked and there are no other windows open.
app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});
