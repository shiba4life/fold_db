/**
 * Utility functions for the DataFold Node UI
 */

/**
 * Display a result or error message in the results area
 * @param {string|object} data - The data to display
 * @param {boolean} isError - Whether this is an error message
 */
function displayResult(data, isError = false) {
    const resultsDiv = document.getElementById('results');
    if (!resultsDiv) {
        console.error('Results div not found');
        return;
    }
    
    // Format the data
    const formattedData = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
    
    // Add appropriate styling
    resultsDiv.className = '';
    resultsDiv.innerHTML = formattedData;
    
    // Scroll to results
    resultsDiv.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    
    // Update node status indicator
    const nodeStatus = document.getElementById('nodeStatus');
    if (nodeStatus) {
        nodeStatus.className = isError ? 'status error' : 'status success';
        const statusIcon = document.getElementById('statusIcon');
        if (statusIcon && window.icons) {
            statusIcon.innerHTML = isError ? icons.x() : icons.check();
        }
    }
}

/**
 * Check if a string is valid JSON
 * @param {string} str - The string to validate
 * @returns {boolean} - Whether the string is valid JSON
 */
function isValidJSON(str) {
    try {
        JSON.parse(str);
        return true;
    } catch (e) {
        return false;
    }
}

/**
 * Show a loading indicator in the specified element
 * @param {HTMLElement} element - The element to show loading in
 * @param {string} message - Optional message to display
 */
function showLoading(element, message = 'Loading...') {
    if (!element) {
        console.error('Cannot show loading: element is null');
        return;
    }
    
    element.innerHTML = `
        <div class="status info">
            <span class="loading"></span>
            <span>${message}</span>
        </div>
    `;
}

/**
 * Handle API errors consistently
 * @param {Error} error - The error object
 * @returns {string} - Formatted error message
 */
function handleApiError(error) {
    console.error('API Error:', error);
    
    // Update node status indicator to show error
    const nodeStatus = document.getElementById('nodeStatus');
    if (nodeStatus) {
        nodeStatus.className = 'status error';
        const statusIcon = document.getElementById('statusIcon');
        if (statusIcon && window.icons) {
            statusIcon.innerHTML = icons.x();
        }
    }
    
    return `Error: ${error.message || 'Unknown error occurred'}`;
}

/**
 * Make an API request with proper error handling
 * @param {string} url - The API endpoint
 * @param {object} options - Fetch options
 * @returns {Promise<object>} - The API response
 */
async function apiRequest(url, options = {}) {
    try {
        const response = await fetch(url, options);
        const data = await response.json();
        
        if (data.error) {
            throw new Error(data.error);
        }
        
        return data;
    } catch (error) {
        throw new Error(handleApiError(error));
    }
}

/**
 * Switch between tabs
 * @param {string} tabName - The name of the tab to switch to
 */
function switchTab(tabName) {
    // Update tab buttons
    const tabButtons = document.querySelectorAll('.tab-button');
    if (tabButtons.length === 0) {
        console.error('No tab buttons found');
        return;
    }
    
    tabButtons.forEach(button => {
        button.classList.remove('active');
    });
    
    const activeButton = document.querySelector(`.tab-button[data-tab="${tabName}"]`);
    if (activeButton) {
        activeButton.classList.add('active');
    } else {
        console.error(`Tab button for ${tabName} not found`);
    }

    // Update tab content
    const tabContents = document.querySelectorAll('.tab-content');
    if (tabContents.length === 0) {
        console.error('No tab contents found');
        return;
    }
    
    tabContents.forEach(content => {
        content.classList.remove('active');
    });
    
    const activeTab = document.getElementById(`${tabName}Tab`);
    if (activeTab) {
        activeTab.classList.add('active');
        
        // Refresh schema list when switching to schemas tab
        if (tabName === 'schemas') {
            schemaModule.loadSchemaList();
        }
        
        // Refresh network status when switching to network tab
        if (tabName === 'network' && window.networkModule) {
            networkModule.getNetworkStatus();
        }
    } else {
        console.error(`Tab content for ${tabName} not found`);
    }
}

/**
 * Show a notification message
 * @param {string} message - The message to display
 * @param {string} type - The type of notification (success, error, warning, info)
 * @param {number} duration - How long to show the notification in ms
 */
function showNotification(message, type = 'info', duration = 3000) {
    // Create notification element if it doesn't exist
    let notificationContainer = document.getElementById('notification-container');
    
    if (!notificationContainer) {
        notificationContainer = document.createElement('div');
        notificationContainer.id = 'notification-container';
        notificationContainer.style.position = 'fixed';
        notificationContainer.style.top = '20px';
        notificationContainer.style.right = '20px';
        notificationContainer.style.zIndex = '1000';
        document.body.appendChild(notificationContainer);
    }
    
    // Create notification
    const notification = document.createElement('div');
    notification.className = `alert alert-${type} fade-in`;
    notification.style.marginBottom = '10px';
    
    // Add icon based on type
    let icon = '';
    if (window.icons) {
        switch (type) {
            case 'success': icon = icons.check(); break;
            case 'error': icon = icons.x(); break;
            case 'warning': icon = icons.warning(); break;
            case 'info': icon = icons.info(); break;
        }
    }
    
    notification.innerHTML = `${icon} ${message}`;
    
    // Add to container
    notificationContainer.appendChild(notification);
    
    // Remove after duration
    setTimeout(() => {
        notification.style.opacity = '0';
        setTimeout(() => {
            notificationContainer.removeChild(notification);
        }, 300);
    }, duration);
}

/**
 * Toggle schema expansion/collapse
 * @param {HTMLElement} element - The schema item element
 */
function toggleSchema(element) {
    if (element.classList.contains('collapsed')) {
        element.classList.remove('collapsed');
        element.classList.add('expanded');
    } else {
        element.classList.remove('expanded');
        element.classList.add('collapsed');
    }
}

// Export functions for use in other modules
window.utils = {
    displayResult,
    isValidJSON,
    showLoading,
    handleApiError,
    apiRequest,
    switchTab,
    toggleSchema,
    showNotification
};
