/**
 * Applications-related functionality for the DataFold Node UI
 */

/**
 * Load the list of applications from the API
 */
async function loadAppsList() {
    const appsList = document.getElementById('appsList');
    
    // If the element doesn't exist yet, try again later
    if (!appsList) {
        console.log('Apps list element not found, retrying in 500ms');
        setTimeout(loadAppsList, 500);
        return;
    }
    
    try {
        utils.showLoading(appsList, 'Loading applications...');
        
        const response = await utils.apiRequest('/api/apps');
        
        if (!response.data || response.data.length === 0) {
            appsList.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-th-large"></i>
                    <p>No applications registered</p>
                    <button class="btn primary" id="emptyStateRegisterAppBtn">
                        <i class="fas fa-plus"></i>
                        <span>Register Application</span>
                    </button>
                </div>
            `;
            
            // Add event listener to the empty state button
            const emptyStateRegisterAppBtn = document.getElementById('emptyStateRegisterAppBtn');
            if (emptyStateRegisterAppBtn) {
                emptyStateRegisterAppBtn.addEventListener('click', window.app.openAppModal);
            }
        } else {
            // Create app cards
            appsList.innerHTML = response.data.map(app => createAppCard(app)).join('');
            
            // Add event listeners to app cards
            response.data.forEach(app => {
                // Start app button
                const startAppBtn = document.getElementById(`startApp_${app.name}`);
                if (startAppBtn) {
                    startAppBtn.addEventListener('click', (event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        startApp(app.name);
                    });
                }
                
                // Stop app button
                const stopAppBtn = document.getElementById(`stopApp_${app.name}`);
                if (stopAppBtn) {
                    stopAppBtn.addEventListener('click', (event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        stopApp(app.name);
                    });
                }
                
                // Unload app button
                const unloadAppBtn = document.getElementById(`unloadApp_${app.name}`);
                if (unloadAppBtn) {
                    unloadAppBtn.addEventListener('click', (event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        confirmUnloadApp(app.name);
                    });
                }
            });
            
            // Update dashboard stats if on dashboard
            if (document.getElementById('appsStatsCard')) {
                const appsStatsCard = document.getElementById('appsStatsCard');
                const runningApps = response.data.filter(app => app.status === 'running').length;
                
                appsStatsCard.innerHTML = `
                    <div class="stat-grid">
                        <div class="stat-item">
                            <div class="stat-label">Total Apps</div>
                            <div class="stat-value">${response.data.length}</div>
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
            }
        }
    } catch (error) {
        if (appsList) {
            appsList.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading applications: ${error.message}</span>
                </div>
                <div class="card-actions">
                    <button class="btn primary" onclick="appsModule.loadAppsList()">
                        <i class="fas fa-sync-alt"></i>
                        <span>Try Again</span>
                    </button>
                </div>
            `;
        } else {
            console.error('Error loading applications:', error);
        }
    }
}

/**
 * Create an app card HTML
 * @param {object} app - The app object
 * @returns {string} - HTML for the app card
 */
function createAppCard(app) {
    const isRunning = app.status === 'running';
    
    return `
        <div class="app-card" id="app_${app.name}">
            <div class="app-header">
                <h3>${app.name}</h3>
                <div class="app-actions">
                    ${isRunning ? `
                        <button class="btn small danger" id="stopApp_${app.name}">
                            <i class="fas fa-stop"></i>
                            <span>Stop</span>
                        </button>
                    ` : `
                        <button class="btn small success" id="startApp_${app.name}">
                            <i class="fas fa-play"></i>
                            <span>Start</span>
                        </button>
                    `}
                    <button class="btn small danger" id="unloadApp_${app.name}">
                        <i class="fas fa-trash"></i>
                        <span>Unload</span>
                    </button>
                </div>
            </div>
            <div class="app-body">
                <div class="app-description">
                    ${app.description || 'No description available'}
                </div>
                <div class="app-meta">
                    <div class="app-meta-item">
                        <i class="fas fa-folder"></i>
                        <span>${app.path || 'Unknown path'}</span>
                    </div>
                    <div class="app-meta-item">
                        <i class="fas fa-circle ${isRunning ? 'text-success' : 'text-danger'}"></i>
                        <span>${isRunning ? 'Running' : 'Stopped'}</span>
                    </div>
                </div>
            </div>
        </div>
    `;
}

/**
 * Register a new application
 * @param {object} appData - The application data
 */
async function registerApp(appData) {
    try {
        utils.showLoadingOverlay('Registering application...');
        
        await utils.apiRequest('/api/apps', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(appData)
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Application "${appData.name}" registered successfully`, 'success');
        
        // Refresh apps list
        loadAppsList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to register application: ${error.message}`, 'error');
    }
}

/**
 * Start an application
 * @param {string} appName - The name of the application to start
 */
async function startApp(appName) {
    try {
        utils.showLoadingOverlay(`Starting application "${appName}"...`);
        
        await utils.apiRequest(`/api/apps/${encodeURIComponent(appName)}/start`, {
            method: 'POST'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Application "${appName}" started successfully`, 'success');
        
        // Refresh apps list
        loadAppsList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to start application: ${error.message}`, 'error');
    }
}

/**
 * Stop an application
 * @param {string} appName - The name of the application to stop
 */
async function stopApp(appName) {
    try {
        utils.showLoadingOverlay(`Stopping application "${appName}"...`);
        
        await utils.apiRequest(`/api/apps/${encodeURIComponent(appName)}/stop`, {
            method: 'POST'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Application "${appName}" stopped successfully`, 'success');
        
        // Refresh apps list
        loadAppsList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to stop application: ${error.message}`, 'error');
    }
}

/**
 * Confirm and unload an application
 * @param {string} appName - The name of the application to unload
 */
async function confirmUnloadApp(appName) {
    if (!confirm(`Are you sure you want to unload application "${appName}"?`)) {
        return;
    }

    try {
        utils.showLoadingOverlay(`Unloading application "${appName}"...`);
        
        await utils.apiRequest(`/api/apps/${encodeURIComponent(appName)}`, {
            method: 'DELETE'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Application "${appName}" unloaded successfully`, 'success');
        
        // Refresh apps list
        loadAppsList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to unload application: ${error.message}`, 'error');
    }
}

// Add CSS for apps components
const style = document.createElement('style');
style.textContent = `
    .text-success {
        color: var(--success-color);
    }
    
    .text-danger {
        color: var(--danger-color);
    }
`;
document.head.appendChild(style);

// Export functions for use in other modules
window.appsModule = {
    loadAppsList,
    registerApp,
    startApp,
    stopApp,
    confirmUnloadApp
};

// Initialize apps list when the module loads
document.addEventListener('DOMContentLoaded', () => {
    // Initialize apps list after a short delay to ensure all elements are loaded
    setTimeout(loadAppsList, 1000);
});
