const express = require('express');
const { spawn, exec } = require('child_process');
const path = require('path');
const fs = require('fs');
const fetch = (...args) => import('node-fetch').then(({default: fetch}) => fetch(...args));
const app = express();
const port = 8000;

// Store running node processes
const runningNodes = {};

// Middleware
app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));

// Enable CORS
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Headers', 'Origin, X-Requested-With, Content-Type, Accept');
  res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  next();
});

// Routes
app.get('/api/nodes', (req, res) => {
  const nodes = Object.keys(runningNodes).map(port => ({
    port: parseInt(port),
    status: runningNodes[port].status,
    pid: runningNodes[port].process ? runningNodes[port].process.pid : null,
    nodeId: runningNodes[port].nodeId || null,
    startTime: runningNodes[port].startTime
  }));
  
  res.json({ nodes });
});

app.post('/api/nodes/start', (req, res) => {
  const { port, configFile } = req.body;
  
  if (!port || !configFile) {
    return res.status(400).json({ error: 'Port and configFile are required' });
  }
  
  // Check if node is already running
  if (runningNodes[port] && runningNodes[port].process) {
    return res.status(400).json({ error: `Node on port ${port} is already running` });
  }
  
  // Log current working directory
  console.log('Current working directory:', process.cwd());
  
  // Get absolute path to config file
  const absoluteConfigPath = path.resolve(__dirname, '..', configFile);
  console.log('Full config path:', absoluteConfigPath);
  
  try {
    // Start the node process
    const nodeProcess = spawn('cargo', [
      'run', 
      '--bin', 
      'datafold_node', 
      '--', 
      '--port', 
      port.toString()
    ], {
      env: { ...process.env, NODE_CONFIG: absoluteConfigPath },
      detached: true,
      cwd: path.resolve(__dirname, '..')  // Set working directory to project root
    });
    
    // Store the process
    runningNodes[port] = {
      process: nodeProcess,
      status: 'starting',
      startTime: new Date().toISOString(),
      nodeId: null
    };
    
    // Handle process output
    nodeProcess.stdout.on('data', (data) => {
      console.log(`Node ${port} output: ${data}`);
      
      // Update status when node is ready
      if (data.toString().includes('App server running at')) {
        runningNodes[port].status = 'running';
      }
    });
    
    nodeProcess.stderr.on('data', (data) => {
      console.error(`Node ${port} error: ${data}`);
    });
    
    nodeProcess.on('close', (code) => {
      console.log(`Node ${port} exited with code ${code}`);
      if (runningNodes[port]) {
        runningNodes[port].status = 'stopped';
        runningNodes[port].process = null;
      }
    });
    
    res.json({ 
      message: `Node started on port ${port}`,
      port: port
    });
  } catch (error) {
    console.error(`Error starting node: ${error.message}`);
    res.status(500).json({ error: `Failed to start node: ${error.message}` });
  }
});

app.post('/api/nodes/stop', (req, res) => {
  const { port } = req.body;
  
  if (!port) {
    return res.status(400).json({ error: 'Port is required' });
  }
  
  // Check if node is running
  if (!runningNodes[port] || !runningNodes[port].process) {
    return res.status(400).json({ error: `No node running on port ${port}` });
  }
  
  try {
    // Kill the process
    if (process.platform === 'win32') {
      // Windows
      exec(`taskkill /pid ${runningNodes[port].process.pid} /T /F`);
    } else {
      // Unix-like
      process.kill(-runningNodes[port].process.pid, 'SIGTERM');
    }
    
    runningNodes[port].status = 'stopping';
    
    res.json({ 
      message: `Node on port ${port} is stopping`,
      port: port
    });
  } catch (error) {
    console.error(`Error stopping node: ${error.message}`);
    res.status(500).json({ error: `Failed to stop node: ${error.message}` });
  }
});

app.post('/api/nodes/init-network', async (req, res) => {
  const { port, discoveryPort, listenPort } = req.body;
  
  if (!port || !discoveryPort || !listenPort) {
    return res.status(400).json({ error: 'Port, discoveryPort, and listenPort are required' });
  }
  
  // Check if node is running
  if (!runningNodes[port] || runningNodes[port].status !== 'running') {
    return res.status(400).json({ error: `Node on port ${port} is not running` });
  }
  
  try {
    // Initialize network
    const response = await fetch(`http://localhost:${port}/api/init_network`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        enable_discovery: true,
        discovery_port: discoveryPort,
        listen_port: listenPort,
        max_connections: 50,
        connection_timeout_secs: 10,
        announcement_interval_secs: 60
      })
    });
    
    if (!response.ok) {
      throw new Error(`Failed to initialize network: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    // Store node ID
    if (data.data && data.data.node_id) {
      runningNodes[port].nodeId = data.data.node_id;
    }
    
    res.json({ 
      message: `Network initialized for node on port ${port}`,
      nodeId: runningNodes[port].nodeId,
      data: data
    });
  } catch (error) {
    console.error(`Error initializing network: ${error.message}`);
    res.status(500).json({ error: `Failed to initialize network: ${error.message}` });
  }
});

app.post('/api/nodes/connect', async (req, res) => {
  const { fromPort, toNodeId } = req.body;
  
  if (!fromPort || !toNodeId) {
    return res.status(400).json({ error: 'fromPort and toNodeId are required' });
  }
  
  // Check if node is running
  if (!runningNodes[fromPort] || runningNodes[fromPort].status !== 'running') {
    return res.status(400).json({ error: `Node on port ${fromPort} is not running` });
  }
  
  try {
    // Connect to node
    const response = await fetch(`http://localhost:${fromPort}/api/connect`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        node_id: toNodeId
      })
    });
    
    if (!response.ok) {
      throw new Error(`Failed to connect to node: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    res.json({ 
      message: `Node on port ${fromPort} connected to node ${toNodeId}`,
      data: data
    });
  } catch (error) {
    console.error(`Error connecting to node: ${error.message}`);
    res.status(500).json({ error: `Failed to connect to node: ${error.message}` });
  }
});

app.post('/api/nodes/query', async (req, res) => {
  const { fromPort, toNodeId, schema, fields } = req.body;
  
  if (!fromPort || !toNodeId || !schema || !fields) {
    return res.status(400).json({ error: 'fromPort, toNodeId, schema, and fields are required' });
  }
  
  // Check if node is running
  if (!runningNodes[fromPort] || runningNodes[fromPort].status !== 'running') {
    return res.status(400).json({ error: `Node on port ${fromPort} is not running` });
  }
  
  try {
    // Query node
    const response = await fetch(`http://localhost:${fromPort}/api/query_node`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        node_id: toNodeId,
        query: {
          schema_name: schema,
          fields: fields,
          pub_key: "",
          trust_distance: 1
        }
      })
    });
    
    if (!response.ok) {
      throw new Error(`Failed to query node: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    res.json({ 
      message: `Node on port ${fromPort} queried node ${toNodeId}`,
      data: data
    });
  } catch (error) {
    console.error(`Error querying node: ${error.message}`);
    res.status(500).json({ error: `Failed to query node: ${error.message}` });
  }
});

app.get('/api/nodes/node-id', async (req, res) => {
  const { port } = req.query;
  
  if (!port) {
    return res.status(400).json({ error: 'Port is required' });
  }
  
  // Check if node is running
  if (!runningNodes[port] || runningNodes[port].status !== 'running') {
    return res.status(400).json({ error: `Node on port ${port} is not running` });
  }
  
  try {
    // Get node ID
    const response = await fetch(`http://localhost:${port}/api/node_id`);
    
    if (!response.ok) {
      throw new Error(`Failed to get node ID: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    // Store node ID
    if (data.data && data.data.node_id) {
      runningNodes[port].nodeId = data.data.node_id;
    }
    
    res.json({ 
      nodeId: runningNodes[port].nodeId,
      data: data
    });
  } catch (error) {
    console.error(`Error getting node ID: ${error.message}`);
    res.status(500).json({ error: `Failed to get node ID: ${error.message}` });
  }
});

app.get('/api/nodes/connected-nodes', async (req, res) => {
  const { port } = req.query;
  
  if (!port) {
    return res.status(400).json({ error: 'Port is required' });
  }
  
  // Check if node is running
  if (!runningNodes[port] || runningNodes[port].status !== 'running') {
    return res.status(400).json({ error: `Node on port ${port} is not running` });
  }
  
  try {
    // Get connected nodes
    const response = await fetch(`http://localhost:${port}/api/connected_nodes`);
    
    if (!response.ok) {
      throw new Error(`Failed to get connected nodes: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    res.json({ 
      message: `Connected nodes for node on port ${port}`,
      data: data
    });
  } catch (error) {
    console.error(`Error getting connected nodes: ${error.message}`);
    res.status(500).json({ error: `Failed to get connected nodes: ${error.message}` });
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Network visualizer server running at http://localhost:${port}`);
});
