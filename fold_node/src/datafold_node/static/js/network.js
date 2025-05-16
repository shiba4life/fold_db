/**
 * Network-related functionality for the DataFold Node UI
 */

/**
 * Initialize the network with the provided configuration
 * @param {object} config - The network configuration
 */
async function initNetwork(config) {
    try {
        try {
            await utils.apiRequest('/api/network/init', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(config)
            });
            
            utils.displayResult('Network initialized successfully');
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network API not available in this version', true);
            } else {
                // Re-throw other errors
                throw apiError;
            }
        }
        
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
        try {
            await utils.apiRequest('/api/network/start', {
                method: 'POST'
            });
            
            utils.displayResult('Network started successfully');
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network API not available in this version', false);
                utils.showNotification('Network functionality is not available in this version', 'info');
            } else {
                // Re-throw other errors
                throw apiError;
            }
        }
        
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
        try {
            await utils.apiRequest('/api/network/stop', {
                method: 'POST'
            });
            
            utils.displayResult('Network stopped successfully');
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network API not available in this version', false);
                utils.showNotification('Network functionality is not available in this version', 'info');
            } else {
                // Re-throw other errors
                throw apiError;
            }
        }
        
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
        // Try to get network status from API
        try {
            const response = await utils.apiRequest('/api/network/status');
            
            const statusDiv = document.getElementById('networkStatus');
            if (statusDiv) {
                // Check if response data exists
                if (response && response.data) {
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
                                <p>${status.connected_nodes_count || 0}</p>
                            </div>
                        </div>
                    `;
                } else {
                    displayNetworkNotAvailable(statusDiv);
                }
            }
            
            return response ? response.data : null;
        } catch (apiError) {
            // Silently handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                const statusDiv = document.getElementById('networkStatus');
                if (statusDiv) {
                    displayNetworkNotAvailable(statusDiv);
                }
                return null;
            } else {
                // Re-throw other errors
                throw apiError;
            }
        }
    } catch (error) {
        console.warn('Network status not available:', error.message);
        
        // Display error message in the network status div
        const statusDiv = document.getElementById('networkStatus');
        if (statusDiv) {
            displayNetworkNotAvailable(statusDiv);
        }
        
        return null;
    }
}

/**
 * Display a message that network functionality is not available
 * @param {HTMLElement} element - The element to display the message in
 */
function displayNetworkNotAvailable(element) {
    element.innerHTML = `
        <div class="network-status">
            <div class="status-info">
                <p>Network API not available in this version.</p>
            </div>
        </div>
    `;
}

/**
 * Discover nodes on the network
 */
async function discoverNodes() {
    try {
        try {
            const response = await utils.apiRequest('/api/network/discover', {
                method: 'POST'
            });
            
            utils.displayResult(response.data);
            return response.data;
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network discovery API not available in this version', false);
                utils.showNotification('Network discovery is not available in this version', 'info');
            } else {
                // Re-throw other errors
                throw apiError;
            }
            return null;
        }
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
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network connection API not available in this version', false);
                utils.showNotification('Network connection is not available in this version', 'info');
            } else {
                // Re-throw other errors
                throw apiError;
            }
        }
        
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
        try {
            const response = await utils.apiRequest('/api/network/nodes');
            
            utils.displayResult(response.data);
            return response.data;
        } catch (apiError) {
            // Handle API endpoint not found errors
            if (apiError.message && apiError.message.includes('API endpoint not found')) {
                utils.displayResult('Network nodes API not available in this version', false);
                utils.showNotification('Network nodes listing is not available in this version', 'info');
            } else {
                // Re-throw other errors
                throw apiError;
            }
            return null;
        }
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
