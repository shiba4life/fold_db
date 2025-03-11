/**
 * Dashboard-related functionality for the DataFold Node UI
 */

/**
 * Initialize the dashboard
 */
function initDashboard() {
    // Load dashboard data
    loadDashboardData();
    
    // Set up refresh interval (every 30 seconds)
    setInterval(loadDashboardData, 30000);
}

/**
 * Load all dashboard data
 */
function loadDashboardData() {
    // Load node status
    loadNodeStatus();
    
    // Load schema stats
    loadSchemaStats();
    
    // Load network stats
    loadNetworkStats();
    
    // Load apps stats
    loadAppsStats();
    
    // Load recent operations
    loadRecentOperations();
}

/**
 * Load node status for the dashboard
 */
function loadNodeStatus() {
    const nodeStatusCard = document.getElementById('nodeStatusCard');
    if (!nodeStatusCard) return;
    
    utils.showLoading(nodeStatusCard);
    
    // Try to get actual network status to determine if node is online
    utils.apiRequest('/api/network/status')
        .then(response => {
            const status = response.data;
            const isInitialized = status && status.initialized;
            const nodeId = status && status.node_id ? status.node_id : 'Not initialized';
            
            // Calculate uptime (simulated)
            const uptime = getSimulatedUptime();
            
            nodeStatusCard.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Status</div>
                        <div class="stat-value">
                            <span class="status-badge ${isInitialized ? 'online' : 'offline'}">
                                ${isInitialized ? 'Online' : 'Offline'}
                            </span>
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Uptime</div>
                        <div class="stat-value">${uptime}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Version</div>
                        <div class="stat-value">1.0.0</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Node ID</div>
                        <div class="stat-value id-value">${nodeId}</div>
                    </div>
                </div>
            `;
            
            // Update node status indicator
            networkModule.updateNodeStatusIndicator(isInitialized);
        })
        .catch(error => {
            console.error('Error loading node status:', error);
            nodeStatusCard.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading node status</span>
                </div>
            `;
            
            // Update node status indicator to offline
            networkModule.updateNodeStatusIndicator(false);
        });
}

/**
 * Get simulated uptime for the node
 * @returns {string} - Formatted uptime string
 */
function getSimulatedUptime() {
    // Generate a random uptime between 1 hour and 7 days
    const hours = Math.floor(Math.random() * 168) + 1;
    
    if (hours < 24) {
        return `${hours}h ${Math.floor(Math.random() * 60)}m`;
    } else {
        const days = Math.floor(hours / 24);
        const remainingHours = hours % 24;
        return `${days}d ${remainingHours}h`;
    }
}

/**
 * Load schema stats for the dashboard
 */
function loadSchemaStats() {
    const schemaStatsCard = document.getElementById('schemaStatsCard');
    if (!schemaStatsCard) return;
    
    utils.showLoading(schemaStatsCard);
    
    // Try to get actual schema count
    utils.apiRequest('/api/schemas')
        .then(response => {
            const schemaCount = response.data ? response.data.length : 0;
            
            schemaStatsCard.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Total Schemas</div>
                        <div class="stat-value">${schemaCount}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Active Schemas</div>
                        <div class="stat-value">${schemaCount}</div>
                    </div>
                </div>
                <div class="card-actions">
                    <button class="btn small primary" onclick="window.app.navigateToPage('schemas')">
                        <i class="fas fa-database"></i>
                        <span>Manage Schemas</span>
                    </button>
                </div>
            `;
        })
        .catch(error => {
            console.error('Error loading schema stats:', error);
            schemaStatsCard.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading schema stats</span>
                </div>
            `;
        });
}

/**
 * Load network stats for the dashboard
 */
function loadNetworkStats() {
    const networkStatsCard = document.getElementById('networkStatsCard');
    if (!networkStatsCard) return;
    
    utils.showLoading(networkStatsCard);
    
    // Try to get actual network status
    utils.apiRequest('/api/network/status')
        .then(response => {
            const status = response.data;
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
        })
        .catch(error => {
            console.error('Error loading network stats:', error);
            networkStatsCard.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading network stats</span>
                </div>
            `;
        });
}

/**
 * Load apps stats for the dashboard
 */
function loadAppsStats() {
    const appsStatsCard = document.getElementById('appsStatsCard');
    if (!appsStatsCard) return;
    
    utils.showLoading(appsStatsCard);
    
    // Try to get actual apps list
    utils.apiRequest('/api/apps')
        .then(response => {
            const appsCount = response.data ? response.data.length : 0;
            const runningApps = response.data ? response.data.filter(app => app.status === 'running').length : 0;
            
            appsStatsCard.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Total Apps</div>
                        <div class="stat-value">${appsCount}</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Running Apps</div>
                        <div class="stat-value">${runningApps}</div>
                    </div>
                </div>
                <div class="card-actions">
                    <button class="btn small primary" onclick="window.app.navigateToPage('apps')">
                        <i class="fas fa-th-large"></i>
                        <span>Manage Apps</span>
                    </button>
                </div>
            `;
        })
        .catch(error => {
            console.error('Error loading apps stats:', error);
            appsStatsCard.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading apps stats</span>
                </div>
            `;
        });
}

/**
 * Load recent operations for the dashboard
 */
function loadRecentOperations() {
    const recentOperationsCard = document.getElementById('recentOperationsCard');
    if (!recentOperationsCard) return;
    
    utils.showLoading(recentOperationsCard);
    
    // Simulate loading recent operations
    setTimeout(() => {
        // Generate some sample operations
        const operations = [
            {
                type: 'query',
                schema: 'UserProfile',
                time: '2 minutes ago',
                status: 'success'
            },
            {
                type: 'mutation',
                schema: 'UserProfile',
                time: '15 minutes ago',
                status: 'success'
            },
            {
                type: 'schema',
                schema: 'UserProfile',
                time: '30 minutes ago',
                status: 'success'
            }
        ];
        
        recentOperationsCard.innerHTML = `
            <div class="operations-list">
                ${operations.map(op => `
                    <div class="operation-item">
                        <div class="operation-icon ${op.type}">
                            <i class="fas fa-${op.type === 'query' ? 'search' : op.type === 'mutation' ? 'edit' : 'database'}"></i>
                        </div>
                        <div class="operation-details">
                            <div class="operation-title">${op.type.charAt(0).toUpperCase() + op.type.slice(1)}: ${op.schema}</div>
                            <div class="operation-time">${op.time}</div>
                        </div>
                        <div class="operation-status ${op.status}">
                            <i class="fas fa-${op.status === 'success' ? 'check-circle' : 'exclamation-circle'}"></i>
                        </div>
                    </div>
                `).join('')}
            </div>
            <div class="card-actions">
                <button class="btn small primary" onclick="window.app.navigateToPage('operations')">
                    <i class="fas fa-code"></i>
                    <span>Run Operations</span>
                </button>
            </div>
        `;
    }, 500);
}

// Export functions for use in other modules
window.dashboardModule = {
    initDashboard,
    loadDashboardData,
    loadNodeStatus,
    loadSchemaStats,
    loadNetworkStats,
    loadAppsStats,
    loadRecentOperations
};

// Initialize dashboard when the module loads
document.addEventListener('DOMContentLoaded', () => {
    // Initialize dashboard after a short delay to ensure all elements are loaded
    setTimeout(initDashboard, 1000);
});
