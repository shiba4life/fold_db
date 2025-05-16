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
    // Set up event listeners immediately
    setupEventListeners();
    
    // Use a more reliable method to ensure DOM is fully loaded
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', onDOMLoaded);
    } else {
        onDOMLoaded();
    }
}

function onDOMLoaded() {
    // Load initial data
    loadInitialData();
    
    console.log('DataFold Node UI initialized');
}

/**
 * Set up event listeners for UI elements
 */
function setupEventListeners() {
    // Schema tab
    const loadSchemaBtn = document.getElementById('loadSchemaBtn');
    if (loadSchemaBtn) {
        loadSchemaBtn.addEventListener('click', schemaModule.loadSchema);
    }
    
    const loadExampleSchemaBtn = document.getElementById('loadExampleSchemaBtn');
    if (loadExampleSchemaBtn) {
        loadExampleSchemaBtn.addEventListener('click', schemaModule.loadExampleSchema);
    }
    
    // Query tab
    const runQueryBtn = document.getElementById('runQueryBtn');
    if (runQueryBtn) {
        runQueryBtn.addEventListener('click', () => operationsModule.executeOperation('query'));
    }
    
    const loadExampleQueryBtn = document.getElementById('loadExampleQueryBtn');
    if (loadExampleQueryBtn) {
        loadExampleQueryBtn.addEventListener('click', operationsModule.loadExampleQuery);
    }
    
    // Mutation tab
    const runMutationBtn = document.getElementById('runMutationBtn');
    if (runMutationBtn) {
        runMutationBtn.addEventListener('click', () => operationsModule.executeOperation('mutation'));
    }
    
    const loadExampleMutationBtn = document.getElementById('loadExampleMutationBtn');
    if (loadExampleMutationBtn) {
        loadExampleMutationBtn.addEventListener('click', operationsModule.loadExampleMutation);
    }
    
    // Network tab (if exists)
    const startNetworkBtn = document.getElementById('startNetworkBtn');
    if (startNetworkBtn) {
        startNetworkBtn.addEventListener('click', networkModule.startNetwork);
    }
    
    const stopNetworkBtn = document.getElementById('stopNetworkBtn');
    if (stopNetworkBtn) {
        stopNetworkBtn.addEventListener('click', networkModule.stopNetwork);
    }
    
    const discoverNodesBtn = document.getElementById('discoverNodesBtn');
    if (discoverNodesBtn) {
        discoverNodesBtn.addEventListener('click', networkModule.discoverNodes);
    }
    
    const listNodesBtn = document.getElementById('listNodesBtn');
    if (listNodesBtn) {
        listNodesBtn.addEventListener('click', networkModule.listNodes);
    }
    
    // Tab switching
    document.querySelectorAll('.tab-button').forEach(button => {
        const tabName = button.getAttribute('data-tab');
        if (tabName) {
            button.addEventListener('click', () => utils.switchTab(tabName));
        }
    });
}

/**
 * Load initial data for the UI
 */
function loadInitialData() {
    // Load schema list
    schemaModule.loadSchemaList();
    
    // Load sample data if the samples module exists
    if (window.samplesModule && window.samplesModule.initSamplesTab) {
        samplesModule.initSamplesTab();
    }
    
    // Load network status if the element exists
    if (document.getElementById('networkStatus')) {
        networkModule.getNetworkStatus();
    }
}

// Make functions available globally
window.app = {
    initApp,
    setupEventListeners,
    loadInitialData
};
