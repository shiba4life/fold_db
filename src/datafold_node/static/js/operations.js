/**
 * Operations-related functionality for the DataFold Node UI
 */

// Example query data
const exampleQuery = {
    type: "query",
    schema: "UserProfile",
    fields: ["username", "email", "bio"],
    filter: null
};

// Example mutation data
const exampleMutation = {
    type: "mutation",
    schema: "UserProfile",
    mutation_type: "create",
    data: {
        username: "johndoe",
        email: "john.doe@example.com",
        bio: "Software developer"
    }
};

/**
 * Execute an operation (query or mutation)
 * @param {string} type - The type of operation ('query' or 'mutation')
 */
async function executeOperation(type) {
    const input = document.getElementById(`${type}Input`).value;
    
    if (!utils.isValidJSON(input)) {
        utils.showNotification('Error', 'Invalid JSON format in operation', 'error');
        return;
    }

    try {
        utils.showLoadingOverlay(`Executing ${type}...`);
        
        // Parse the input to get a JSON object
        const operation = JSON.parse(input);
        
        console.log(`Sending operation to API: ${JSON.stringify(operation)}`);
        
        const response = await utils.apiRequest('/api/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                operation: operation
            })
        });
        
        console.log(`Received API response: ${JSON.stringify(response)}`);
        
        utils.hideLoadingOverlay();
        
        // Check if response contains data property
        if (response && response.data !== undefined) {
            utils.displayResult(response.data);
            
            // Add to recent operations (simulated)
            addToRecentOperations(type, operation.schema);
        } else if (response && response.error) {
            // Handle error response
            utils.displayResult(`Error: ${response.error}`, true);
        } else {
            // Handle unexpected response format
            console.error("Unexpected response format:", response);
            utils.displayResult(`Unexpected response format: ${JSON.stringify(response)}`, true);
        }
    } catch (error) {
        utils.hideLoadingOverlay();
        console.error("Operation execution error:", error);
        utils.displayResult(`Error: ${error.message}`, true);
    }
}

/**
 * Add an operation to the recent operations list (simulated)
 * @param {string} type - The type of operation ('query' or 'mutation')
 * @param {string} schema - The schema name
 */
function addToRecentOperations(type, schema) {
    const recentOperationsCard = document.getElementById('recentOperationsCard');
    if (!recentOperationsCard) return;
    
    // Check if there's already an operations list
    let operationsList = recentOperationsCard.querySelector('.operations-list');
    
    // If not, create one
    if (!operationsList) {
        operationsList = document.createElement('div');
        operationsList.className = 'operations-list';
        recentOperationsCard.innerHTML = '';
        recentOperationsCard.appendChild(operationsList);
    }
    
    // Create a new operation item
    const operationItem = document.createElement('div');
    operationItem.className = 'operation-item';
    
    // Get icon class based on type
    const iconClass = type === 'query' ? 'search' : 'edit';
    const typeClass = type === 'query' ? 'query' : 'mutation';
    
    // Set operation item content
    operationItem.innerHTML = `
        <div class="operation-icon ${typeClass}">
            <i class="fas fa-${iconClass}"></i>
        </div>
        <div class="operation-details">
            <div class="operation-title">${type.charAt(0).toUpperCase() + type.slice(1)}: ${schema}</div>
            <div class="operation-time">Just now</div>
        </div>
        <div class="operation-status success">
            <i class="fas fa-check-circle"></i>
        </div>
    `;
    
    // Add to the beginning of the list
    operationsList.insertBefore(operationItem, operationsList.firstChild);
    
    // Limit to 5 items
    const items = operationsList.querySelectorAll('.operation-item');
    if (items.length > 5) {
        operationsList.removeChild(items[items.length - 1]);
    }
    
    // Add card actions if not present
    if (!recentOperationsCard.querySelector('.card-actions')) {
        const cardActions = document.createElement('div');
        cardActions.className = 'card-actions';
        cardActions.innerHTML = `
            <button class="btn small primary" onclick="window.app.navigateToPage('operations')">
                <i class="fas fa-code"></i>
                <span>Run Operations</span>
            </button>
        `;
        recentOperationsCard.appendChild(cardActions);
    }
}

/**
 * Load an example query into the textarea
 */
function loadExampleQuery() {
    document.getElementById('queryInput').value = JSON.stringify(exampleQuery, null, 2);
}

/**
 * Load an example mutation into the textarea
 */
function loadExampleMutation() {
    document.getElementById('mutationInput').value = JSON.stringify(exampleMutation, null, 2);
}

/**
 * Create a custom query operation
 * @param {string} schemaName - The schema name
 * @param {Array<string>} fields - The fields to query
 * @param {object} filter - The filter to apply (optional)
 * @returns {object} - The query operation
 */
function createQueryOperation(schemaName, fields, filter = null) {
    return {
        type: "query",
        schema: schemaName,
        fields: fields,
        filter: filter
    };
}

/**
 * Create a custom mutation operation
 * @param {string} schemaName - The schema name
 * @param {string} mutationType - The mutation type ('create', 'update', 'delete')
 * @param {object} data - The data for the mutation
 * @param {object} filter - The filter to apply (for update/delete, optional)
 * @returns {object} - The mutation operation
 */
function createMutationOperation(schemaName, mutationType, data, filter = null) {
    return {
        type: "mutation",
        schema: schemaName,
        mutation_type: mutationType,
        data: data,
        filter: filter
    };
}

/**
 * Execute a custom query
 * @param {string} schemaName - The schema name
 * @param {Array<string>} fields - The fields to query
 * @param {object} filter - The filter to apply (optional)
 */
async function executeCustomQuery(schemaName, fields, filter = null) {
    const query = createQueryOperation(schemaName, fields, filter);
    
    // Navigate to operations page
    window.app.navigateToPage('operations');
    
    // Switch to query tab
    window.app.switchOperationsTab('query');
    
    // Set the query in the query input
    document.getElementById('queryInput').value = JSON.stringify(query, null, 2);
    
    // Execute the query
    await executeOperation('query');
}

/**
 * Execute a custom mutation
 * @param {string} schemaName - The schema name
 * @param {string} mutationType - The mutation type ('create', 'update', 'delete')
 * @param {object} data - The data for the mutation
 * @param {object} filter - The filter to apply (for update/delete, optional)
 */
async function executeCustomMutation(schemaName, mutationType, data, filter = null) {
    const mutation = createMutationOperation(schemaName, mutationType, data, filter);
    
    // Navigate to operations page
    window.app.navigateToPage('operations');
    
    // Switch to mutation tab
    window.app.switchOperationsTab('mutation');
    
    // Set the mutation in the mutation input
    document.getElementById('mutationInput').value = JSON.stringify(mutation, null, 2);
    
    // Execute the mutation
    await executeOperation('mutation');
}

// Export functions for use in other modules
window.operationsModule = {
    executeOperation,
    loadExampleQuery,
    loadExampleMutation,
    createQueryOperation,
    createMutationOperation,
    executeCustomQuery,
    executeCustomMutation
};
