/**
 * Network-related functionality for the DataFold Node UI
 */

/**
 * Initialize the network with the provided configuration
 * @param {object} config - The network configuration
 */
async function initNetwork(config) {
    try {
        await utils.apiRequest('/api/network/init', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(config)
        });
        
        utils.displayResult('Network initialized successfully');
        // Update network status
        await getNetworkStatus();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * Start the network
 */
async function startNetwork() {
    try {
        await utils.apiRequest('/api/network/start', {
            method: 'POST'
        });
        
        utils.displayResult('Network started successfully');
        // Update network status
        await getNetworkStatus();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * Stop the network
 */
async function stopNetwork() {
    try {
        await utils.apiRequest('/api/network/stop', {
            method: 'POST'
        });
        
        utils.displayResult('Network stopped successfully');
        // Update network status
        await getNetworkStatus();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * Get the current network status
 */
async function getNetworkStatus() {
    try {
        const response = await utils.apiRequest('/api/network/status');
        
        const statusDiv = document.getElementById('networkStatus');
        if (statusDiv) {
            const status = response.data;
            
            statusDiv.innerHTML = `
                <div class="network-status">
                    <div class="status-card">
                        <h4>Node ID</h4>
                        <p>${status.node_id || 'Not initialized'}</p>
                    </div>
                    <div class="status-card">
                        <h4>Status</h4>
                        <p>${status.initialized ? 'Initialized' : 'Not initialized'}</p>
                    </div>
                    <div class="status-card">
                        <h4>Connected Nodes</h4>
                        <p>${status.connected_nodes_count}</p>
                    </div>
                </div>
            `;
        }
        
        return response.data;
    } catch (error) {
        console.error('Error getting network status:', error);
        return null;
    }
}

/**
 * Discover nodes on the network
 */
async function discoverNodes() {
    try {
        const response = await utils.apiRequest('/api/network/discover', {
            method: 'POST'
        });
        
        utils.displayResult(response.data);
        return response.data;
    } catch (error) {
        utils.displayResult(error.message, true);
        return null;
    }
}

/**
 * Connect to a specific node
 * @param {string} nodeId - The ID of the node to connect to
 */
async function connectToNode(nodeId) {
    try {
        await utils.apiRequest('/api/network/connect', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                node_id: nodeId
            })
        });
        
        utils.displayResult(`Connected to node ${nodeId}`);
        // Update network status
        await getNetworkStatus();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * List all known nodes
 */
async function listNodes() {
    try {
        const response = await utils.apiRequest('/api/network/nodes');
        
        utils.displayResult(response.data);
        return response.data;
    } catch (error) {
        utils.displayResult(error.message, true);
        return null;
    }
}

// Export functions for use in other modules
window.networkModule = {
    initNetwork,
    startNetwork,
    stopNetwork,
    getNetworkStatus,
    discoverNodes,
    connectToNode,
    listNodes
};
