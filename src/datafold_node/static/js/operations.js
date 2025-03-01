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
        utils.displayResult('Invalid JSON format in operation', true);
        return;
    }

    try {
        const resultsDiv = document.getElementById('results');
        utils.showLoading(resultsDiv, `Executing ${type}...`);
        
        const response = await utils.apiRequest('/api/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                operation: input
            })
        });
        
        utils.displayResult(response.data);
    } catch (error) {
        utils.displayResult(error.message, true);
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

// Export functions for use in other modules
window.operationsModule = {
    executeOperation,
    loadExampleQuery,
    loadExampleMutation
};
