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
            utils.showNotification('Network initialized successfully', 'success');
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
        utils.showNotification(error.message, 'error');
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
            utils.showNotification('Network started successfully', 'success');
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
        utils.showNotification(error.message, 'error');
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
            utils.showNotification('Network stopped successfully', 'success');
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
            
            if (response && response.data) {
                renderNetworkStatus(response.data);
            } else {
                renderNetworkStatus(null);
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

        renderNetworkStatus(null);
        utils.showNotification(error.message, 'error');
        return null;
    }
}

/**
 * Render the network status information in the DOM
 * @param {object|null} status - Status data from the server
 */
function renderNetworkStatus(status) {
    const statusDiv = document.getElementById('networkStatus');
    if (!statusDiv) return;

    if (!status) {
        displayNetworkNotAvailable(statusDiv);
        return;
    }

    const connected = status.connected_nodes ?? status.connected_nodes_count ?? 0;

    statusDiv.innerHTML = `
        <div class="network-status">
            ${status.node_id ? `<div class="status-card"><h4>Node ID</h4><p>${status.node_id}</p></div>` : ''}
            <div class="status-card">
                <h4>Status</h4>
                <p>${status.running || status.initialized ? 'Running' : 'Stopped'}</p>
            </div>
            <div class="status-card">
                <h4>Connected Nodes</h4>
                <p>${connected}</p>
            </div>
            ${status.discovery_enabled !== undefined ? `<div class="status-card"><h4>Discovery</h4><p>${status.discovery_enabled ? 'Enabled' : 'Disabled'}</p></div>` : ''}
            ${status.listen_address ? `<div class="status-card"><h4>Listen Address</h4><p>${status.listen_address}</p></div>` : ''}
            ${status.last_error ? `<div class="status-card error"><h4>Last Error</h4><p>${status.last_error}</p></div>` : ''}
        </div>
    `;
}

/**
 * Render a list of nodes in the DOM
 * @param {Array} nodes - Array of node info objects or strings
 */
function renderNodesList(nodes) {
    const container = document.getElementById('nodesList');
    if (!container) return;
    container.innerHTML = formatNodeList(nodes);
}

/**
 * Convert a list of nodes to HTML
 * @param {Array} nodes
 * @returns {string}
 */
function formatNodeList(nodes) {
    if (!Array.isArray(nodes) || nodes.length === 0) {
        return 'No nodes found';
    }
    const items = nodes
        .map(n => `<li>${n.id ?? n}</li>`) 
        .join('');
    return `<ul>${items}</ul>`;
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

            utils.displayResult(formatNodeList(response.data));
            utils.showNotification('Discovery complete', 'success');
            renderNodesList(response.data);
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
        utils.showNotification(error.message, 'error');
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
            utils.showNotification(`Connected to node ${nodeId}`, 'success');
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

            utils.displayResult(formatNodeList(response.data));
            utils.showNotification('Retrieved node list', 'success');
            renderNodesList(response.data);
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
        utils.showNotification(error.message, 'error');
        return null;
    }
}

// Export functions for use in other modules
window.networkModule = {
    initNetwork,
    startNetwork,
    stopNetwork,
    getNetworkStatus,
    renderNetworkStatus,
    discoverNodes,
    connectToNode,
    listNodes,
    renderNodesList
};
