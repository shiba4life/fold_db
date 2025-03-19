/**
 * Operations-related functionality for the DataFold Node UI
 */

// Function to fetch example query from the API
async function fetchExampleQuery() {
    try {
        const response = await utils.apiRequest('/api/examples/user_profile_queries.json');
        // Return the first query from the array
        if (Array.isArray(response.data) && response.data.length > 0) {
            return response.data[0];
        }
        throw new Error('No example queries found');
    } catch (error) {
        console.error('Error fetching example query:', error);
        // Fallback to a simple example if the API fails
        return {
            type: "query",
            schema: "UserProfile",
            fields: ["username", "email", "bio"],
            filter: null
        };
    }
}

// Function to fetch example mutation from the API
async function fetchExampleMutation() {
    try {
        const response = await utils.apiRequest('/api/examples/user_profile_mutations.json');
        // Return the first mutation from the array
        if (Array.isArray(response.data) && response.data.length > 0) {
            return response.data[0];
        }
        throw new Error('No example mutations found');
    } catch (error) {
        console.error('Error fetching example mutation:', error);
        // Fallback to a simple example if the API fails
        return {
            type: "mutation",
            schema: "UserProfile",
            mutation_type: "create",
            data: {
                username: "johndoe",
                email: "john.doe@example.com",
                bio: "Software developer"
            }
        };
    }
}

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
async function loadExampleQuery() {
    const exampleQuery = await fetchExampleQuery();
    document.getElementById('queryInput').value = JSON.stringify(exampleQuery, null, 2);
}

/**
 * Load an example mutation into the textarea
 */
async function loadExampleMutation() {
    const exampleMutation = await fetchExampleMutation();
    document.getElementById('mutationInput').value = JSON.stringify(exampleMutation, null, 2);
}

// Export functions for use in other modules
window.operationsModule = {
    executeOperation,
    loadExampleQuery,
    loadExampleMutation
};
