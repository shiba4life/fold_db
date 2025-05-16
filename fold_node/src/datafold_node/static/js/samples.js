/**
 * Sample data management for the DataFold Node UI
 */

// Sample descriptions
const sampleDescriptions = {
    // Schema descriptions
    'UserProfile': 'A basic user profile schema with common fields like username, email, and bio. Includes various permission policies to demonstrate different access levels.',
    'ProductCatalog': 'A product catalog schema suitable for e-commerce applications. Includes fields for product details, pricing, and inventory management.',
    'UserProfile2': 'An extended user profile schema with additional fields and more complex permission policies.',
    'UserProfile3': 'A user profile schema with collection fields and custom field mappers.',
    
    // Query descriptions
    'BasicUserQuery': 'A simple query to retrieve basic user information without any filtering.',
    'UserProfileQuery1': 'A query to retrieve user profile information with specific fields.',
    'UserProfileQuery2': 'A filtered query to retrieve user profiles matching specific criteria.',
    
    // Mutation descriptions
    'CreateUser': 'A mutation to create a new user profile with basic information.',
    'UserProfileMutation1': 'A mutation to update an existing user profile.',
    'UserProfileMutation2': 'A mutation to delete a user profile.'
};

/**
 * Load sample schemas from the API
 */
async function loadSampleSchemas() {
    const schemaItemsDiv = document.getElementById('schemaItems');
    
    try {
        utils.showLoading(schemaItemsDiv);
        
        try {
            const response = await utils.apiRequest('/api/samples/schemas');
            
            // Check if response data exists and is an array
            if (!response || !response.data || !Array.isArray(response.data)) {
                schemaItemsDiv.innerHTML = '<div class="status">Sample schemas API not available</div>';
                return;
            }
            
            if (response.data.length === 0) {
                schemaItemsDiv.innerHTML = '<div class="status">No sample schemas available</div>';
            } else {
                schemaItemsDiv.innerHTML = '';
                
                response.data.forEach(name => {
                    const sampleItem = document.createElement('div');
                    sampleItem.className = 'sample-item';
                    
                    const description = sampleDescriptions[name] || 'A sample schema for DataFold.';
                    
                    sampleItem.innerHTML = `
                        <div class="sample-header">
                            <h4>${name}</h4>
                            <div class="sample-actions">
                                <button class="btn btn-sm btn-secondary" onclick="samplesModule.previewSample('schema', '${name}')">
                                    <span class="icon">${icons.eye()}</span> Preview
                                </button>
                                <button class="btn btn-sm btn-primary" onclick="samplesModule.loadSample('schema', '${name}')">
                                    <span class="icon">${icons.download()}</span> Load
                                </button>
                            </div>
                        </div>
                        <div class="sample-description">${description}</div>
                    `;
                    
                    schemaItemsDiv.appendChild(sampleItem);
                });
            }
        } catch (error) {
            console.error('Error loading sample schemas:', error);
            schemaItemsDiv.innerHTML = '<div class="status error">Sample schemas API not available</div>';
            return;
        }
    } catch (error) {
        schemaItemsDiv.innerHTML = `<div class="status error">${error.message}</div>`;
    }
}

/**
 * Load sample queries from the API
 */
async function loadSampleQueries() {
    const queryItemsDiv = document.getElementById('queryItems');
    
    try {
        utils.showLoading(queryItemsDiv);
        
        try {
            const response = await utils.apiRequest('/api/samples/queries');
            
            // Check if response data exists and is an array
            if (!response || !response.data || !Array.isArray(response.data)) {
                queryItemsDiv.innerHTML = '<div class="status">Sample queries API not available</div>';
                return;
            }
            
            if (response.data.length === 0) {
                queryItemsDiv.innerHTML = '<div class="status">No sample queries available</div>';
            } else {
                queryItemsDiv.innerHTML = '';
                
                response.data.forEach(name => {
                    const sampleItem = document.createElement('div');
                    sampleItem.className = 'sample-item';
                    
                    const description = sampleDescriptions[name] || 'A sample query for DataFold.';
                    
                    sampleItem.innerHTML = `
                        <div class="sample-header">
                            <h4>${name}</h4>
                            <div class="sample-actions">
                                <button class="btn btn-sm btn-secondary" onclick="samplesModule.previewSample('query', '${name}')">
                                    <span class="icon">${icons.eye()}</span> Preview
                                </button>
                                <button class="btn btn-sm btn-primary" onclick="samplesModule.loadSample('query', '${name}')">
                                    <span class="icon">${icons.download()}</span> Load
                                </button>
                            </div>
                        </div>
                        <div class="sample-description">${description}</div>
                    `;
                    
                    queryItemsDiv.appendChild(sampleItem);
                });
            }
        } catch (error) {
            console.error('Error loading sample queries:', error);
            if (error.message && error.message.includes('API endpoint not found')) {
                queryItemsDiv.innerHTML = '<div class="status">Sample queries API not available</div>';
            } else {
                queryItemsDiv.innerHTML = '<div class="status error">Error loading sample queries</div>';
            }
            return;
        }
    } catch (error) {
        queryItemsDiv.innerHTML = `<div class="status error">${error.message}</div>`;
    }
}

/**
 * Load sample mutations from the API
 */
async function loadSampleMutations() {
    const mutationItemsDiv = document.getElementById('mutationItems');
    
    try {
        utils.showLoading(mutationItemsDiv);
        
        try {
            const response = await utils.apiRequest('/api/samples/mutations');
            
            // Check if response data exists and is an array
            if (!response || !response.data || !Array.isArray(response.data)) {
                mutationItemsDiv.innerHTML = '<div class="status">Sample mutations API not available</div>';
                return;
            }
            
            if (response.data.length === 0) {
                mutationItemsDiv.innerHTML = '<div class="status">No sample mutations available</div>';
            } else {
                mutationItemsDiv.innerHTML = '';
                
                response.data.forEach(name => {
                    const sampleItem = document.createElement('div');
                    sampleItem.className = 'sample-item';
                    
                    const description = sampleDescriptions[name] || 'A sample mutation for DataFold.';
                    
                    sampleItem.innerHTML = `
                        <div class="sample-header">
                            <h4>${name}</h4>
                            <div class="sample-actions">
                                <button class="btn btn-sm btn-secondary" onclick="samplesModule.previewSample('mutation', '${name}')">
                                    <span class="icon">${icons.eye()}</span> Preview
                                </button>
                                <button class="btn btn-sm btn-primary" onclick="samplesModule.loadSample('mutation', '${name}')">
                                    <span class="icon">${icons.download()}</span> Load
                                </button>
                            </div>
                        </div>
                        <div class="sample-description">${description}</div>
                    `;
                    
                    mutationItemsDiv.appendChild(sampleItem);
                });
            }
        } catch (error) {
            console.error('Error loading sample mutations:', error);
            if (error.message && error.message.includes('API endpoint not found')) {
                mutationItemsDiv.innerHTML = '<div class="status">Sample mutations API not available</div>';
            } else {
                mutationItemsDiv.innerHTML = '<div class="status error">Error loading sample mutations</div>';
            }
            return;
        }
    } catch (error) {
        mutationItemsDiv.innerHTML = `<div class="status error">${error.message}</div>`;
    }
}

/**
 * Preview a sample
 * @param {string} type - The type of sample ('schema', 'query', or 'mutation')
 * @param {string} name - The name of the sample
 */
async function previewSample(type, name) {
    try {
        const response = await utils.apiRequest(`/api/samples/${type}/${name}`);
        
        // Check if response data exists
        if (!response || !response.data) {
            utils.displayResult(`Sample ${type} '${name}' not available`, true);
            return;
        }
        
        // Set modal title
        document.getElementById('previewTitle').textContent = `${name} (${type})`;
        
        // Set description
        const description = sampleDescriptions[name] || `A sample ${type} for DataFold.`;
        document.getElementById('previewDescription').textContent = description;
        
        // Set code preview
        document.getElementById('previewCode').textContent = JSON.stringify(response.data, null, 2);
        
        // Set load button action
        const loadBtn = document.getElementById('loadSampleBtn');
        loadBtn.onclick = () => {
            loadSample(type, name);
            document.getElementById('samplePreviewModal').style.display = 'none';
        };
        
        // Show modal
        document.getElementById('samplePreviewModal').style.display = 'block';
    } catch (error) {
        utils.displayResult(`Error loading sample: ${error.message}`, true);
    }
}

/**
 * Load a sample
 * @param {string} type - The type of sample ('schema', 'query', or 'mutation')
 * @param {string} name - The name of the sample
 */
async function loadSample(type, name) {
    try {
        const response = await utils.apiRequest(`/api/samples/${type}/${name}`);
        
        // Check if response data exists
        if (!response || !response.data) {
            utils.displayResult(`Sample ${type} '${name}' not available`, true);
            return;
        }
        
        switch (type) {
            case 'schema':
                // Load schema into schema editor
                document.getElementById('schemaInput').value = JSON.stringify(response.data, null, 2);
                
                // Switch to schema tab
                document.querySelector('.tab-button[data-tab="schema"]').click();
                
                utils.displayResult(`Sample schema '${name}' loaded successfully`);
                break;
                
            case 'query':
                // Load query into query editor
                document.getElementById('queryInput').value = JSON.stringify(response.data, null, 2);
                
                // Switch to query tab
                document.querySelector('.tab-button[data-tab="query"]').click();
                
                utils.displayResult(`Sample query '${name}' loaded successfully`);
                break;
                
            case 'mutation':
                // Load mutation into mutation editor
                document.getElementById('mutationInput').value = JSON.stringify(response.data, null, 2);
                
                // Switch to mutation tab
                document.querySelector('.tab-button[data-tab="mutation"]').click();
                
                utils.displayResult(`Sample mutation '${name}' loaded successfully`);
                break;
        }
    } catch (error) {
        utils.displayResult(`Error loading sample: ${error.message}`, true);
    }
}

/**
 * Initialize the samples tab
 */
function initSamplesTab() {
    // Set up tab switching first
    setupSamplesTabNavigation();
    
    // Load sample data with a slight delay to ensure DOM is ready
    setTimeout(() => {
        // Only load the active tab initially (which should be schemas)
        loadSampleSchemas();
        
        // Pre-initialize the other tabs with "not available" messages
        const queryItemsDiv = document.getElementById('queryItems');
        if (queryItemsDiv) {
            queryItemsDiv.innerHTML = '<div class="status">Sample queries API not available</div>';
        }
        
        const mutationItemsDiv = document.getElementById('mutationItems');
        if (mutationItemsDiv) {
            mutationItemsDiv.innerHTML = '<div class="status">Sample mutations API not available</div>';
        }
    }, 100);
}

/**
 * Set up navigation between sample types (schemas, queries, mutations)
 */
function setupSamplesTabNavigation() {
    // Wait for DOM to be fully loaded
    if (!document.getElementById('samplesTab')) {
        console.log('Samples tab not found, will try again in 100ms');
        setTimeout(setupSamplesTabNavigation, 100);
        return;
    }
    
    const sampleNavButtons = document.querySelectorAll('.samples-nav-button');
    const sampleLists = document.querySelectorAll('.sample-list');
    
    if (sampleNavButtons.length === 0 || sampleLists.length === 0) {
        console.log('Sample navigation elements not found, will try again in 100ms');
        setTimeout(setupSamplesTabNavigation, 100);
        return;
    }
    
    console.log('Setting up samples tab navigation');
    
    // First, make sure all sample lists are hidden except the first one
    sampleLists.forEach((list, index) => {
        if (index === 0) {
            list.classList.add('active');
        } else {
            list.classList.remove('active');
        }
    });
    
    // Then set up click handlers for the buttons
    sampleNavButtons.forEach(button => {
        button.addEventListener('click', () => {
            console.log(`Clicked on ${button.dataset.sampleType} tab`);
            
            // Remove active class from all buttons and lists
            sampleNavButtons.forEach(btn => btn.classList.remove('active'));
            sampleLists.forEach(list => list.classList.remove('active'));
            
            // Add active class to clicked button and corresponding list
            button.classList.add('active');
            const sampleType = button.dataset.sampleType;
            const targetList = document.getElementById(`${sampleType}Samples`);
            
            if (targetList) {
                targetList.classList.add('active');
                
                // Reload the content for the selected tab
                if (sampleType === 'schemas') {
                    loadSampleSchemas();
                } else if (sampleType === 'queries') {
                    loadSampleQueries();
                } else if (sampleType === 'mutations') {
                    loadSampleMutations();
                }
            }
        });
    });
}

// Export functions for use in other modules
window.samplesModule = {
    loadSampleSchemas,
    loadSampleQueries,
    loadSampleMutations,
    previewSample,
    loadSample,
    initSamplesTab
};