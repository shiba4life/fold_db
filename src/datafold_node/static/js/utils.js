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
    
    // Clear any existing content
    resultsDiv.innerHTML = '';
    
    // Create a formatted display
    if (typeof data === 'string') {
        resultsDiv.textContent = data;
    } else {
        // Format JSON with syntax highlighting
        const formattedJson = formatJson(data);
        resultsDiv.innerHTML = formattedJson;
    }
    
    // Show notification
    if (isError) {
        showNotification('Error', typeof data === 'string' ? data : 'Operation failed', 'error');
    } else {
        showNotification('Success', 'Operation completed successfully', 'success');
    }
}

/**
 * Format JSON with syntax highlighting
 * @param {object} json - The JSON object to format
 * @returns {string} - HTML string with syntax highlighting
 */
function formatJson(json) {
    if (!json) return '';
    
    const jsonString = JSON.stringify(json, null, 2);
    
    // Simple syntax highlighting
    return jsonString
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match) {
            let cls = 'json-number';
            if (/^"/.test(match)) {
                if (/:$/.test(match)) {
                    cls = 'json-key';
                } else {
                    cls = 'json-string';
                }
            } else if (/true|false/.test(match)) {
                cls = 'json-boolean';
            } else if (/null/.test(match)) {
                cls = 'json-null';
            }
            return '<span class="' + cls + '">' + match + '</span>';
        });
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
    
    element.innerHTML = `<div class="status-loading">${message}</div>`;
}

/**
 * Show the global loading overlay
 * @param {string} message - The message to display
 */
function showLoadingOverlay(message = 'Loading...') {
    const overlay = document.getElementById('loadingOverlay');
    if (overlay) {
        const messageElement = overlay.querySelector('.loading-message');
        if (messageElement) {
            messageElement.textContent = message;
        }
        overlay.classList.add('active');
    }
}

/**
 * Hide the global loading overlay
 */
function hideLoadingOverlay() {
    const overlay = document.getElementById('loadingOverlay');
    if (overlay) {
        overlay.classList.remove('active');
    }
}

/**
 * Show a notification
 * @param {string} title - The notification title
 * @param {string} message - The notification message
 * @param {string} type - The notification type (success, error, info, warning)
 * @param {number} duration - How long to show the notification in ms (0 for no auto-hide)
 */
function showNotification(title, message, type = 'info', duration = 5000) {
    const container = document.getElementById('notificationContainer');
    if (!container) {
        console.error('Notification container not found');
        return;
    }
    
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    
    // Get icon based on type
    let icon = 'info-circle';
    switch (type) {
        case 'success':
            icon = 'check-circle';
            break;
        case 'error':
            icon = 'exclamation-circle';
            break;
        case 'warning':
            icon = 'exclamation-triangle';
            break;
    }
    
    // Set notification content
    notification.innerHTML = `
        <div class="notification-icon">
            <i class="fas fa-${icon}"></i>
        </div>
        <div class="notification-content">
            <div class="notification-title">${title}</div>
            <div class="notification-message">${message}</div>
        </div>
        <button class="notification-close">&times;</button>
    `;
    
    // Add to container
    container.appendChild(notification);
    
    // Add close button event listener
    const closeBtn = notification.querySelector('.notification-close');
    if (closeBtn) {
        closeBtn.addEventListener('click', () => {
            notification.remove();
        });
    }
    
    // Auto-hide after duration (if not 0)
    if (duration > 0) {
        setTimeout(() => {
            // Add fade-out animation
            notification.style.opacity = '0';
            notification.style.transform = 'translateX(100%)';
            
            // Remove after animation
            setTimeout(() => {
                if (notification.parentNode === container) {
                    container.removeChild(notification);
                }
            }, 300);
        }, duration);
    }
}

/**
 * Handle API errors consistently
 * @param {Error} error - The error object
 * @returns {string} - Formatted error message
 */
function handleApiError(error) {
    console.error('API Error:', error);
    
    // Show notification for the error
    showNotification('API Error', error.message || 'Unknown error occurred', 'error');
    
    return error.message || 'Unknown error occurred';
}

/**
 * Make an API request with proper error handling
 * @param {string} url - The API endpoint
 * @param {object} options - Fetch options
 * @returns {Promise<object>} - The API response
 */
async function apiRequest(url, options = {}) {
    try {
        console.log(`API Request to ${url} with options:`, options);
        
        // Show loading overlay for POST, PUT, DELETE requests
        const isWriteOperation = options.method && options.method !== 'GET';
        if (isWriteOperation) {
            showLoadingOverlay('Processing request...');
        }
        
        const response = await fetch(url, options);
        
        // Hide loading overlay
        if (isWriteOperation) {
            hideLoadingOverlay();
        }
        
        // Log the raw response for debugging
        const responseText = await response.text();
        console.log(`Raw API Response: ${responseText}`);
        
        // Try to parse the response as JSON
        let data;
        try {
            data = JSON.parse(responseText);
            console.log(`Parsed API Response:`, data);
        } catch (parseError) {
            console.error(`Error parsing response as JSON: ${parseError.message}`);
            console.error(`Response that failed to parse: ${responseText}`);
            throw new Error(`Invalid JSON response: ${responseText.substring(0, 100)}${responseText.length > 100 ? '...' : ''}`);
        }
        
        if (data.error) {
            console.error(`API returned error: ${data.error}`);
            throw new Error(data.error);
        }
        
        return data;
    } catch (error) {
        console.error(`API Request failed: ${error.message}`);
        
        // Hide loading overlay if it was shown
        hideLoadingOverlay();
        
        throw new Error(handleApiError(error));
    }
}

/**
 * Format a date string
 * @param {string|Date} date - The date to format
 * @returns {string} - Formatted date string
 */
function formatDate(date) {
    if (!date) return '';
    
    const d = new Date(date);
    if (isNaN(d.getTime())) return '';
    
    return d.toLocaleDateString() + ' ' + d.toLocaleTimeString();
}

/**
 * Format a relative time (e.g., "2 minutes ago")
 * @param {string|Date} date - The date to format
 * @returns {string} - Formatted relative time
 */
function formatRelativeTime(date) {
    if (!date) return '';
    
    const d = new Date(date);
    if (isNaN(d.getTime())) return '';
    
    const now = new Date();
    const diffMs = now - d;
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);
    
    if (diffSec < 60) {
        return `${diffSec} second${diffSec !== 1 ? 's' : ''} ago`;
    } else if (diffMin < 60) {
        return `${diffMin} minute${diffMin !== 1 ? 's' : ''} ago`;
    } else if (diffHour < 24) {
        return `${diffHour} hour${diffHour !== 1 ? 's' : ''} ago`;
    } else if (diffDay < 30) {
        return `${diffDay} day${diffDay !== 1 ? 's' : ''} ago`;
    } else {
        return formatDate(date);
    }
}

/**
 * Truncate a string to a maximum length
 * @param {string} str - The string to truncate
 * @param {number} maxLength - The maximum length
 * @returns {string} - Truncated string
 */
function truncateString(str, maxLength = 100) {
    if (!str) return '';
    if (str.length <= maxLength) return str;
    return str.substring(0, maxLength) + '...';
}

/**
 * Copy text to clipboard
 * @param {string} text - The text to copy
 * @returns {Promise<boolean>} - Whether the copy was successful
 */
async function copyToClipboard(text) {
    try {
        await navigator.clipboard.writeText(text);
        showNotification('Success', 'Copied to clipboard', 'success', 2000);
        return true;
    } catch (error) {
        console.error('Failed to copy text:', error);
        showNotification('Error', 'Failed to copy to clipboard', 'error');
        return false;
    }
}

/**
 * Generate a random ID
 * @param {number} length - The length of the ID
 * @returns {string} - Random ID
 */
function generateId(length = 8) {
    return Math.random().toString(36).substring(2, 2 + length);
}

// Add CSS for JSON formatting
const style = document.createElement('style');
style.textContent = `
    .json-key {
        color: #0066cc;
    }
    .json-string {
        color: #008800;
    }
    .json-number {
        color: #aa0000;
    }
    .json-boolean {
        color: #0000aa;
    }
    .json-null {
        color: #666666;
    }
    
    .stat-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
        gap: 15px;
        margin-bottom: 15px;
    }
    
    .stat-item {
        display: flex;
        flex-direction: column;
    }
    
    .stat-label {
        font-size: 12px;
        color: var(--text-light);
        margin-bottom: 5px;
    }
    
    .stat-value {
        font-size: 16px;
        font-weight: 500;
    }
    
    .id-value {
        font-family: monospace;
        font-size: 14px;
        word-break: break-all;
    }
    
    .status-badge {
        display: inline-block;
        padding: 3px 8px;
        border-radius: 12px;
        font-size: 12px;
        font-weight: 500;
    }
    
    .status-badge.online {
        background-color: rgba(40, 167, 69, 0.2);
        color: var(--success-color);
    }
    
    .status-badge.offline {
        background-color: rgba(220, 53, 69, 0.2);
        color: var(--danger-color);
    }
    
    .card-actions {
        display: flex;
        justify-content: flex-end;
        margin-top: 15px;
    }
    
    .error-message {
        display: flex;
        align-items: center;
        color: var(--danger-color);
        font-size: 14px;
    }
    
    .error-message i {
        margin-right: 8px;
    }
    
    .operations-list {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    
    .operation-item {
        display: flex;
        align-items: center;
        padding: 10px;
        background-color: #f8f9fa;
        border-radius: 4px;
        transition: background-color 0.2s;
    }
    
    .operation-item:hover {
        background-color: #e9ecef;
    }
    
    .operation-icon {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-right: 12px;
        color: white;
    }
    
    .operation-icon.query {
        background-color: var(--info-color);
    }
    
    .operation-icon.mutation {
        background-color: var(--primary-color);
    }
    
    .operation-icon.schema {
        background-color: var(--success-color);
    }
    
    .operation-details {
        flex: 1;
    }
    
    .operation-title {
        font-weight: 500;
        margin-bottom: 2px;
    }
    
    .operation-time {
        font-size: 12px;
        color: var(--text-light);
    }
    
    .operation-status {
        margin-left: 10px;
    }
    
    .operation-status.success {
        color: var(--success-color);
    }
    
    .operation-status.error {
        color: var(--danger-color);
    }
`;
document.head.appendChild(style);

// Export functions for use in other modules
window.utils = {
    displayResult,
    formatJson,
    isValidJSON,
    showLoading,
    showLoadingOverlay,
    hideLoadingOverlay,
    showNotification,
    handleApiError,
    apiRequest,
    formatDate,
    formatRelativeTime,
    truncateString,
    copyToClipboard,
    generateId
};
