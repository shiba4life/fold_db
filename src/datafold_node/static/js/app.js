/**
 * Main application file for the DataFold Node UI
 */

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    initApp();
});

/**
 * Initialize the application
 */
function initApp() {
    // Set up event listeners
    setupEventListeners();
    
    // Load initial data
    loadInitialData();
    
    console.log('DataFold Node UI initialized');
}

/**
 * Set up event listeners for UI elements
 */
function setupEventListeners() {
    // Navigation
    setupNavigation();
    
    // Dashboard
    setupDashboard();
    
    // Schemas
    setupSchemas();
    
    // Operations
    setupOperations();
    
    // Network
    setupNetwork();
    
    // Apps
    setupApps();
    
    // Settings
    setupSettings();
    
    // Global refresh button
    const refreshBtn = document.getElementById('refreshBtn');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', () => {
            refreshCurrentPage();
        });
    }
}

/**
 * Set up navigation event listeners
 */
function setupNavigation() {
    // Sidebar navigation
    document.querySelectorAll('.nav-item').forEach(item => {
        item.addEventListener('click', () => {
            const pageName = item.getAttribute('data-page');
            if (pageName) {
                navigateToPage(pageName);
            }
        });
    });
}

/**
 * Navigate to a specific page
 * @param {string} pageName - The name of the page to navigate to
 */
function navigateToPage(pageName) {
    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.remove('active');
    });
    
    const activeNavItem = document.querySelector(`.nav-item[data-page="${pageName}"]`);
    if (activeNavItem) {
        activeNavItem.classList.add('active');
    }
    
    // Update page title
    const pageTitle = document.querySelector('.page-title');
    if (pageTitle) {
        pageTitle.textContent = pageName.charAt(0).toUpperCase() + pageName.slice(1);
    }
    
    // Show the selected page
    document.querySelectorAll('.page').forEach(page => {
        page.classList.remove('active');
    });
    
    const activePage = document.getElementById(`${pageName}Page`);
    if (activePage) {
        activePage.classList.add('active');
        
        // Refresh data for the page
        refreshPageData(pageName);
    }
}

/**
 * Refresh data for the current page
 */
function refreshCurrentPage() {
    const activePage = document.querySelector('.page.active');
    if (activePage) {
        const pageName = activePage.id.replace('Page', '');
        refreshPageData(pageName);
    }
}

/**
 * Refresh data for a specific page
 * @param {string} pageName - The name of the page to refresh
 */
function refreshPageData(pageName) {
    switch (pageName) {
        case 'dashboard':
            loadDashboardData();
            break;
        case 'schemas':
            schemaModule.loadSchemaList();
            break;
        case 'network':
            networkModule.getNetworkStatus();
            networkModule.listNodes();
            break;
        case 'apps':
            if (typeof appsModule !== 'undefined') {
                appsModule.loadAppsList();
            }
            break;
    }
}

/**
 * Set up dashboard event listeners
 */
function setupDashboard() {
    // Quick action buttons
    const loadSchemaBtn = document.getElementById('loadSchemaBtn');
    if (loadSchemaBtn) {
        loadSchemaBtn.addEventListener('click', () => {
            navigateToPage('schemas');
            openSchemaModal();
        });
    }
    
    const runQueryBtn = document.getElementById('runQueryBtn');
    if (runQueryBtn) {
        runQueryBtn.addEventListener('click', () => {
            navigateToPage('operations');
            // Ensure query tab is active
            document.querySelector('.tab[data-tab="query"]').click();
        });
    }
    
    const runMutationBtn = document.getElementById('runMutationBtn');
    if (runMutationBtn) {
        runMutationBtn.addEventListener('click', () => {
            navigateToPage('operations');
            // Ensure mutation tab is active
            document.querySelector('.tab[data-tab="mutation"]').click();
        });
    }
    
    const startNetworkBtn = document.getElementById('startNetworkBtn');
    if (startNetworkBtn) {
        startNetworkBtn.addEventListener('click', () => {
            networkModule.startNetwork();
        });
    }
}

/**
 * Set up schemas event listeners
 */
function setupSchemas() {
    // New schema button
    const newSchemaBtn = document.getElementById('newSchemaBtn');
    if (newSchemaBtn) {
        newSchemaBtn.addEventListener('click', openSchemaModal);
    }
    
    // Refresh schemas button
    const refreshSchemasBtn = document.getElementById('refreshSchemasBtn');
    if (refreshSchemasBtn) {
        refreshSchemasBtn.addEventListener('click', () => {
            schemaModule.loadSchemaList();
        });
    }
    
    // Schema modal
    const closeSchemaModal = document.getElementById('closeSchemaModal');
    if (closeSchemaModal) {
        closeSchemaModal.addEventListener('click', closeSchemaModal);
    }
    
    // Load example schema button
    const loadExampleSchemaBtn = document.getElementById('loadExampleSchemaBtn');
    if (loadExampleSchemaBtn) {
        loadExampleSchemaBtn.addEventListener('click', () => {
            schemaModule.loadExampleSchema();
        });
    }
    
    // Save schema button
    const saveSchemaBtn = document.getElementById('saveSchemaBtn');
    if (saveSchemaBtn) {
        saveSchemaBtn.addEventListener('click', () => {
            schemaModule.loadSchema();
            closeSchemaModal();
        });
    }
}

/**
 * Open the schema modal
 */
function openSchemaModal() {
    const modal = document.getElementById('schemaModal');
    if (modal) {
        modal.classList.add('active');
    }
}

/**
 * Close the schema modal
 */
function closeSchemaModal() {
    const modal = document.getElementById('schemaModal');
    if (modal) {
        modal.classList.remove('active');
    }
}

/**
 * Set up operations event listeners
 */
function setupOperations() {
    // Tab switching
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.getAttribute('data-tab');
            if (tabName) {
                switchOperationsTab(tabName);
            }
        });
    });
    
    // Execute query button
    const executeQueryBtn = document.getElementById('executeQueryBtn');
    if (executeQueryBtn) {
        executeQueryBtn.addEventListener('click', () => {
            operationsModule.executeOperation('query');
        });
    }
    
    // Execute mutation button
    const executeMutationBtn = document.getElementById('executeMutationBtn');
    if (executeMutationBtn) {
        executeMutationBtn.addEventListener('click', () => {
            operationsModule.executeOperation('mutation');
        });
    }
    
    // Load example query button
    const loadExampleQueryBtn = document.getElementById('loadExampleQueryBtn');
    if (loadExampleQueryBtn) {
        loadExampleQueryBtn.addEventListener('click', () => {
            operationsModule.loadExampleQuery();
        });
    }
    
    // Load example mutation button
    const loadExampleMutationBtn = document.getElementById('loadExampleMutationBtn');
    if (loadExampleMutationBtn) {
        loadExampleMutationBtn.addEventListener('click', () => {
            operationsModule.loadExampleMutation();
        });
    }
    
    // Clear results button
    const clearResultsBtn = document.getElementById('clearResultsBtn');
    if (clearResultsBtn) {
        clearResultsBtn.addEventListener('click', () => {
            const resultsDiv = document.getElementById('results');
            if (resultsDiv) {
                resultsDiv.innerHTML = '<div class="empty-results">No results to display</div>';
            }
        });
    }
}

/**
 * Switch between operations tabs
 * @param {string} tabName - The name of the tab to switch to
 */
function switchOperationsTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.tab').forEach(tab => {
        tab.classList.remove('active');
    });
    
    const activeTab = document.querySelector(`.tab[data-tab="${tabName}"]`);
    if (activeTab) {
        activeTab.classList.add('active');
    }
    
    // Update tab panes
    document.querySelectorAll('.tab-pane').forEach(pane => {
        pane.classList.remove('active');
    });
    
    const activePane = document.getElementById(`${tabName}Tab`);
    if (activePane) {
        activePane.classList.add('active');
    }
}

/**
 * Set up network event listeners
 */
function setupNetwork() {
    // Initialize network button
    const initNetworkBtn = document.getElementById('initNetworkBtn');
    if (initNetworkBtn) {
        initNetworkBtn.addEventListener('click', () => {
            const config = {
                listen_address: document.getElementById('listenAddress').value,
                discovery_port: parseInt(document.getElementById('discoveryPort').value),
                max_connections: parseInt(document.getElementById('maxConnections').value),
                connection_timeout_secs: parseInt(document.getElementById('connectionTimeout').value),
                announcement_interval_secs: parseInt(document.getElementById('announcementInterval').value),
                enable_discovery: document.getElementById('enableDiscovery').checked
            };
            
            networkModule.initNetwork(config);
        });
    }
    
    // Save network config button
    const saveNetworkConfigBtn = document.getElementById('saveNetworkConfigBtn');
    if (saveNetworkConfigBtn) {
        saveNetworkConfigBtn.addEventListener('click', () => {
            const config = {
                listen_address: document.getElementById('listenAddress').value,
                discovery_port: parseInt(document.getElementById('discoveryPort').value),
                max_connections: parseInt(document.getElementById('maxConnections').value),
                connection_timeout_secs: parseInt(document.getElementById('connectionTimeout').value),
                announcement_interval_secs: parseInt(document.getElementById('announcementInterval').value),
                enable_discovery: document.getElementById('enableDiscovery').checked
            };
            
            networkModule.initNetwork(config);
        });
    }
    
    // Refresh network button
    const refreshNetworkBtn = document.getElementById('refreshNetworkBtn');
    if (refreshNetworkBtn) {
        refreshNetworkBtn.addEventListener('click', () => {
            networkModule.getNetworkStatus();
            networkModule.listNodes();
        });
    }
    
    // Start network button
    const startNetworkPageBtn = document.getElementById('startNetworkPageBtn');
    if (startNetworkPageBtn) {
        startNetworkPageBtn.addEventListener('click', () => {
            networkModule.startNetwork();
        });
    }
    
    // Stop network button
    const stopNetworkBtn = document.getElementById('stopNetworkBtn');
    if (stopNetworkBtn) {
        stopNetworkBtn.addEventListener('click', () => {
            networkModule.stopNetwork();
        });
    }
    
    // Discover nodes button
    const discoverNodesBtn = document.getElementById('discoverNodesBtn');
    if (discoverNodesBtn) {
        discoverNodesBtn.addEventListener('click', () => {
            networkModule.discoverNodes();
        });
    }
    
    // List nodes button
    const listNodesBtn = document.getElementById('listNodesBtn');
    if (listNodesBtn) {
        listNodesBtn.addEventListener('click', () => {
            networkModule.listNodes();
        });
    }
    
    // Connect to node button
    const connectToNodeBtn = document.getElementById('connectToNodeBtn');
    if (connectToNodeBtn) {
        connectToNodeBtn.addEventListener('click', () => {
            const nodeId = document.getElementById('nodeId').value.trim();
            if (nodeId) {
                networkModule.connectToNode(nodeId);
            } else {
                utils.showNotification('Error', 'Please enter a node ID', 'error');
            }
        });
    }
}

/**
 * Set up apps event listeners
 */
function setupApps() {
    // Register app button
    const registerAppBtn = document.getElementById('registerAppBtn');
    if (registerAppBtn) {
        registerAppBtn.addEventListener('click', openAppModal);
    }
    
    // Refresh apps button
    const refreshAppsBtn = document.getElementById('refreshAppsBtn');
    if (refreshAppsBtn) {
        refreshAppsBtn.addEventListener('click', () => {
            if (typeof appsModule !== 'undefined') {
                appsModule.loadAppsList();
            }
        });
    }
    
    // App modal
    const closeAppModal = document.getElementById('closeAppModal');
    if (closeAppModal) {
        closeAppModal.addEventListener('click', closeAppModal);
    }
    
    // Save app button
    const saveAppBtn = document.getElementById('saveAppBtn');
    if (saveAppBtn) {
        saveAppBtn.addEventListener('click', () => {
            if (typeof appsModule !== 'undefined') {
                const appData = {
                    name: document.getElementById('appName').value.trim(),
                    path: document.getElementById('appPath').value.trim(),
                    description: document.getElementById('appDescription').value.trim()
                };
                
                if (appData.name && appData.path) {
                    appsModule.registerApp(appData);
                    closeAppModal();
                } else {
                    utils.showNotification('Error', 'Please fill in all required fields', 'error');
                }
            }
        });
    }
}

/**
 * Open the app modal
 */
function openAppModal() {
    const modal = document.getElementById('appModal');
    if (modal) {
        modal.classList.add('active');
    }
}

/**
 * Close the app modal
 */
function closeAppModal() {
    const modal = document.getElementById('appModal');
    if (modal) {
        modal.classList.remove('active');
    }
}

/**
 * Set up settings event listeners
 */
function setupSettings() {
    // Save general settings button
    const saveGeneralSettingsBtn = document.getElementById('saveGeneralSettingsBtn');
    if (saveGeneralSettingsBtn) {
        saveGeneralSettingsBtn.addEventListener('click', () => {
            if (typeof settingsModule !== 'undefined') {
                settingsModule.saveGeneralSettings();
            } else {
                utils.showNotification('Success', 'Settings saved successfully', 'success');
            }
        });
    }
    
    // Save authentication settings button
    const saveAuthSettingsBtn = document.getElementById('saveAuthSettingsBtn');
    if (saveAuthSettingsBtn) {
        saveAuthSettingsBtn.addEventListener('click', () => {
            if (typeof settingsModule !== 'undefined') {
                settingsModule.saveAuthSettings();
            } else {
                utils.showNotification('Success', 'Authentication settings saved successfully', 'success');
            }
        });
    }
    
    // Generate keys button
    const generateKeysBtn = document.getElementById('generateKeysBtn');
    if (generateKeysBtn) {
        generateKeysBtn.addEventListener('click', () => {
            if (typeof settingsModule !== 'undefined') {
                settingsModule.generateKeys();
            } else {
                // Simulate key generation
                document.getElementById('publicKey').value = 'pk_' + Math.random().toString(36).substring(2, 15);
                document.getElementById('privateKey').value = 'sk_' + Math.random().toString(36).substring(2, 15);
                utils.showNotification('Success', 'New keys generated successfully', 'success');
            }
        });
    }
}

/**
 * Load initial data for the UI
 */
function loadInitialData() {
    // Start with dashboard page
    navigateToPage('dashboard');
    
    // Load dashboard data
    loadDashboardData();
}

/**
 * Load dashboard data
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
    if (nodeStatusCard) {
        utils.showLoading(nodeStatusCard);
        
        // Simulate loading node status
        setTimeout(() => {
            nodeStatusCard.innerHTML = `
                <div class="stat-grid">
                    <div class="stat-item">
                        <div class="stat-label">Status</div>
                        <div class="stat-value">
                            <span class="status-badge online">Online</span>
                        </div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Uptime</div>
                        <div class="stat-value">3h 45m</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Version</div>
                        <div class="stat-value">1.0.0</div>
                    </div>
                    <div class="stat-item">
                        <div class="stat-label">Node ID</div>
                        <div class="stat-value id-value">node_${Math.random().toString(36).substring(2, 10)}</div>
                    </div>
                </div>
            `;
        }, 500);
    }
}

/**
 * Load schema stats for the dashboard
 */
function loadSchemaStats() {
    const schemaStatsCard = document.getElementById('schemaStatsCard');
    if (schemaStatsCard) {
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
                        <button class="btn small primary" onclick="navigateToPage('schemas')">
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
}

/**
 * Load network stats for the dashboard
 */
function loadNetworkStats() {
    const networkStatsCard = document.getElementById('networkStatsCard');
    if (networkStatsCard) {
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
                        <button class="btn small primary" onclick="navigateToPage('network')">
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
}

/**
 * Load apps stats for the dashboard
 */
function loadAppsStats() {
    const appsStatsCard = document.getElementById('appsStatsCard');
    if (appsStatsCard) {
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
                        <button class="btn small primary" onclick="navigateToPage('apps')">
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
}

/**
 * Load recent operations for the dashboard
 */
function loadRecentOperations() {
    const recentOperationsCard = document.getElementById('recentOperationsCard');
    if (recentOperationsCard) {
        utils.showLoading(recentOperationsCard);
        
        // Simulate loading recent operations
        setTimeout(() => {
            recentOperationsCard.innerHTML = `
                <div class="operations-list">
                    <div class="operation-item">
                        <div class="operation-icon query">
                            <i class="fas fa-search"></i>
                        </div>
                        <div class="operation-details">
                            <div class="operation-title">Query: UserProfile</div>
                            <div class="operation-time">2 minutes ago</div>
                        </div>
                        <div class="operation-status success">
                            <i class="fas fa-check-circle"></i>
                        </div>
                    </div>
                    <div class="operation-item">
                        <div class="operation-icon mutation">
                            <i class="fas fa-edit"></i>
                        </div>
                        <div class="operation-details">
                            <div class="operation-title">Mutation: Create UserProfile</div>
                            <div class="operation-time">15 minutes ago</div>
                        </div>
                        <div class="operation-status success">
                            <i class="fas fa-check-circle"></i>
                        </div>
                    </div>
                    <div class="operation-item">
                        <div class="operation-icon schema">
                            <i class="fas fa-database"></i>
                        </div>
                        <div class="operation-details">
                            <div class="operation-title">Schema: Load UserProfile</div>
                            <div class="operation-time">30 minutes ago</div>
                        </div>
                        <div class="operation-status success">
                            <i class="fas fa-check-circle"></i>
                        </div>
                    </div>
                </div>
                <div class="card-actions">
                    <button class="btn small primary" onclick="navigateToPage('operations')">
                        <i class="fas fa-code"></i>
                        <span>Run Operations</span>
                    </button>
                </div>
            `;
        }, 500);
    }
}

// Make functions available globally
window.app = {
    initApp,
    setupEventListeners,
    loadInitialData,
    navigateToPage,
    refreshCurrentPage,
    openSchemaModal,
    closeSchemaModal,
    openAppModal,
    closeAppModal,
    switchOperationsTab
};
