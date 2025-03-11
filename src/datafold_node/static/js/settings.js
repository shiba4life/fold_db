/**
 * Settings-related functionality for the DataFold Node UI
 */

/**
 * Initialize the settings page
 */
function initSettings() {
    // Load settings data
    loadSettingsData();
}

/**
 * Load settings data
 */
function loadSettingsData() {
    // Load general settings
    loadGeneralSettings();
    
    // Load authentication settings
    loadAuthSettings();
}

/**
 * Load general settings
 */
function loadGeneralSettings() {
    // Simulate loading general settings
    setTimeout(() => {
        // Set default values
        document.getElementById('nodeName').value = 'DataFold Node';
        document.getElementById('dataDirectory').value = '/data';
        document.getElementById('enableLogging').checked = true;
    }, 500);
}

/**
 * Load authentication settings
 */
function loadAuthSettings() {
    // Simulate loading authentication settings
    setTimeout(() => {
        // Set default values
        document.getElementById('publicKey').value = 'pk_' + utils.generateId(12);
        document.getElementById('privateKey').value = 'sk_' + utils.generateId(16);
    }, 500);
}

/**
 * Save general settings
 */
async function saveGeneralSettings() {
    try {
        utils.showLoadingOverlay('Saving general settings...');
        
        // Get form values
        const nodeName = document.getElementById('nodeName').value;
        const dataDirectory = document.getElementById('dataDirectory').value;
        const enableLogging = document.getElementById('enableLogging').checked;
        
        // Validate form values
        if (!nodeName) {
            utils.hideLoadingOverlay();
            utils.showNotification('Error', 'Node name is required', 'error');
            return;
        }
        
        if (!dataDirectory) {
            utils.hideLoadingOverlay();
            utils.showNotification('Error', 'Data directory is required', 'error');
            return;
        }
        
        // Simulate API request
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'General settings saved successfully', 'success');
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to save general settings: ${error.message}`, 'error');
    }
}

/**
 * Save authentication settings
 */
async function saveAuthSettings() {
    try {
        utils.showLoadingOverlay('Saving authentication settings...');
        
        // Get form values
        const publicKey = document.getElementById('publicKey').value;
        const privateKey = document.getElementById('privateKey').value;
        
        // Validate form values
        if (!publicKey) {
            utils.hideLoadingOverlay();
            utils.showNotification('Error', 'Public key is required', 'error');
            return;
        }
        
        if (!privateKey) {
            utils.hideLoadingOverlay();
            utils.showNotification('Error', 'Private key is required', 'error');
            return;
        }
        
        // Simulate API request
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'Authentication settings saved successfully', 'success');
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to save authentication settings: ${error.message}`, 'error');
    }
}

/**
 * Generate new keys
 */
async function generateKeys() {
    try {
        utils.showLoadingOverlay('Generating new keys...');
        
        // Simulate API request
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Generate new keys
        const publicKey = 'pk_' + utils.generateId(12);
        const privateKey = 'sk_' + utils.generateId(16);
        
        // Update form values
        document.getElementById('publicKey').value = publicKey;
        document.getElementById('privateKey').value = privateKey;
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'New keys generated successfully', 'success');
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to generate new keys: ${error.message}`, 'error');
    }
}

/**
 * Copy public key to clipboard
 */
async function copyPublicKey() {
    const publicKey = document.getElementById('publicKey').value;
    if (publicKey) {
        await utils.copyToClipboard(publicKey);
    }
}

/**
 * Copy private key to clipboard
 */
async function copyPrivateKey() {
    const privateKey = document.getElementById('privateKey').value;
    if (privateKey) {
        await utils.copyToClipboard(privateKey);
    }
}

// Export functions for use in other modules
window.settingsModule = {
    initSettings,
    loadSettingsData,
    loadGeneralSettings,
    loadAuthSettings,
    saveGeneralSettings,
    saveAuthSettings,
    generateKeys,
    copyPublicKey,
    copyPrivateKey
};

// Initialize settings when the module loads
document.addEventListener('DOMContentLoaded', () => {
    // Initialize settings after a short delay to ensure all elements are loaded
    setTimeout(initSettings, 1000);
});
