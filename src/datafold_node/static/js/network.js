/**
 * Network-related functionality for the DataFold Node UI
 */

/**
 * Initialize the network with the provided configuration
 * @param {object} config - The network configuration
 */
async function initNetwork(config) {
    try {
        utils.showLoadingOverlay('Initializing network...');
        
        await utils.apiRequest('/api/network/init', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(config)
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'Network initialized successfully', 'success');
        
        // Update network status
        await getNetworkStatus();
        
        // Update connected nodes
        await listNodes();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to initialize network: ${error.message}`, 'error');
    }
}

/**
 * Start the network
 */
async function startNetwork() {
    try {
        utils.showLoadingOverlay('Starting network...');
        
        await utils.apiRequest('/api/network/start', {
            method: 'POST'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'Network started successfully', 'success');
        
        // Update network status
        await getNetworkStatus();
        
        // Update node status indicator
        updateNodeStatusIndicator(true);
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to start network: ${error.message}`, 'error');
    }
}

/**
 * Stop the network
 */
async function stopNetwork() {
    try {
        utils.showLoadingOverlay('Stopping network...');
        
        await utils.apiRequest('/api/network/stop', {
            method: 'POST'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'Network stopped successfully', 'success');
        
        // Update network status
        await getNetworkStatus();
        
        // Update node status indicator
        updateNodeStatusIndicator(false);
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to stop network: ${error.message}`, 'error');
    }
}

/**
 * Update the node status indicator in the sidebar
 * @param {boolean} isOnline - Whether the node is online
 */
function updateNodeStatusIndicator(isOnline) {
    const statusIndicator = document.querySelector('.node-status .status-indicator');
    if (statusIndicator) {
        statusIndicator.className = `status-indicator ${isOnline ? 'online' : 'offline'}`;
        
        const statusText = statusIndicator.querySelector('.status-text');
        if (statusText) {
            statusText.textContent = isOnline ? 'Node Online' : 'Node Offline';
        }
    }
}

/**
 * Get the current network status
 */
async function getNetworkStatus() {
    try {
        const response = await utils.apiRequest('/api/network/status');
        const status = response.data;
        
        // Update network status card
        const networkStatus = document.getElementById('networkStatus');
        if (networkStatus) {
            const isInitialized = status && status.initialized;
            const nodeId = status && status.node_id ? status.node_id : 'Not initialized';
            const connectedNodes = status ? status.connected_nodes_count : 0;
            
            networkStatus.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Status</div>
                        <div class="stat-value">
                            <span class="status-badge ${isInitialized ? 'online' : 'offline'}">
                                ${isInitialized ? 'Active' : 'Inactive'}
                            </span>
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Node ID</div>
                        <div class="stat-value id-value">${nodeId}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Connected Nodes</div>
                        <div class="stat-value">${connectedNodes}</div>
                    </div>
                </div>
            `;
            
            // Update node status indicator
            updateNodeStatusIndicator(isInitialized);
        }
        
        // Update network stats card on dashboard
        const networkStatsCard = document.getElementById('networkStatsCard');
        if (networkStatsCard) {
            const isInitialized = status && status.initialized;
            const connectedNodes = status ? status.connected_nodes_count : 0;
            
            networkStatsCard.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Status</div>
                        <div class="stat-value">
                            <span class="status-badge ${isInitialized ? 'online' : 'offline'}">
                                ${isInitialized ? 'Active' : 'Inactive'}
                            </span>
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Connected Nodes</div>
                        <div class="stat-value">${connectedNodes}</div>
                    </div>
                </div>
                <div class="card-actions">
                    <button class="btn small primary" onclick="window.app.navigateToPage('network')">
                        <i class="fas fa-network-wired"></i>
                        <span>Manage Network</span>
                    </button>
                </div>
            `;
        }
        
        return status;
    } catch (error) {
        console.error('Error getting network status:', error);
        
        // Update network status card with error
        const networkStatus = document.getElementById('networkStatus');
        if (networkStatus) {
            networkStatus.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error getting network status: ${error.message}</span>
                </div>
                <div class="card-actions">
                    <button class="btn primary" onclick="networkModule.getNetworkStatus()">
                        <i class="fas fa-sync-alt"></i>
                        <span>Try Again</span>
                    </button>
                </div>
            `;
        }
        
        return null;
    }
}

/**
 * Discover nodes on the network
 */
async function discoverNodes() {
    try {
        utils.showLoadingOverlay('Discovering nodes...');
        
        const response = await utils.apiRequest('/api/network/discover', {
            method: 'POST'
        });
        
        utils.hideLoadingOverlay();
        
        // Show notification
        const discoveredCount = response.data && response.data.discovered_nodes ? response.data.discovered_nodes.length : 0;
        utils.showNotification('Success', `Discovered ${discoveredCount} node${discoveredCount !== 1 ? 's' : ''}`, 'success');
        
        // Update connected nodes
        await listNodes();
        
        return response.data;
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to discover nodes: ${error.message}`, 'error');
        return null;
    }
}

/**
 * Connect to a specific node
 * @param {string} nodeId - The ID of the node to connect to
 */
async function connectToNode(nodeId) {
    try {
        utils.showLoadingOverlay(`Connecting to node ${nodeId}...`);
        
        await utils.apiRequest('/api/network/connect', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                node_id: nodeId
            })
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Connected to node ${nodeId}`, 'success');
        
        // Update network status
        await getNetworkStatus();
        
        // Update connected nodes
        await listNodes();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to connect to node: ${error.message}`, 'error');
    }
}

/**
 * List all known nodes
 */
async function listNodes() {
    try {
        const response = await utils.apiRequest('/api/network/nodes');
        
        // Update connected nodes card
        const connectedNodes = document.getElementById('connectedNodes');
        if (connectedNodes) {
            if (!response.data || response.data.length === 0) {
                connectedNodes.innerHTML = `
                    <div class="empty-state">
                        <i class="fas fa-network-wired"></i>
                        <p>No connected nodes</p>
                        <button class="btn primary" onclick="networkModule.discoverNodes()">
                            <i class="fas fa-search"></i>
                            <span>Discover Nodes</span>
                        </button>
                    </div>
                `;
            } else {
                connectedNodes.innerHTML = `
                    <div class="nodes-list">
                        ${response.data.map(node => `
                            <div class="node-item">
                                <div class="node-icon">
                                    <i class="fas fa-server"></i>
                                </div>
                                <div class="node-details">
                                    <div class="node-id">${node.id || 'Unknown ID'}</div>
                                    <div class="node-address">${node.address || 'Unknown Address'}</div>
                                </div>
                                <div class="node-status">
                                    <span class="status-badge online">Connected</span>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                `;
            }
        }
        
        return response.data;
    } catch (error) {
        console.error('Error listing nodes:', error);
        
        // Update connected nodes card with error
        const connectedNodes = document.getElementById('connectedNodes');
        if (connectedNodes) {
            connectedNodes.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error listing nodes: ${error.message}</span>
                </div>
                <div class="card-actions">
                    <button class="btn primary" onclick="networkModule.listNodes()">
                        <i class="fas fa-sync-alt"></i>
                        <span>Try Again</span>
                    </button>
                </div>
            `;
        }
        
        return null;
    }
}

// Add CSS for network components
const style = document.createElement('style');
style.textContent = `
    .nodes-list {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    
    .node-item {
        display: flex;
        align-items: center;
        padding: 12px;
        background-color: #f8f9fa;
        border-radius: 4px;
        transition: background-color 0.2s;
    }
    
    .node-item:hover {
        background-color: #e9ecef;
    }
    
    .node-icon {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: var(--primary-color);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-right: 15px;
    }
    
    .node-details {
        flex: 1;
    }
    
    .node-id {
        font-weight: 500;
        margin-bottom: 3px;
        font-family: monospace;
    }
    
    .node-address {
        font-size: 12px;
        color: var(--text-light);
    }
    
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 30px;
        text-align: center;
    }
    
    .empty-state i {
        font-size: 48px;
        color: var(--text-light);
        margin-bottom: 15px;
    }
    
    .empty-state p {
        color: var(--text-light);
        margin-bottom: 20px;
    }
`;
document.head.appendChild(style);

// Export functions for use in other modules
window.networkModule = {
    initNetwork,
    startNetwork,
    stopNetwork,
    getNetworkStatus,
    discoverNodes,
    connectToNode,
    listNodes,
    updateNodeStatusIndicator
};
