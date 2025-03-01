/**
 * Schema-related operations for the DataFold Node UI
 */

// Example schema data
const exampleSchema = {
    name: "UserProfile",
    fields: {
        username: {
            field_type: "Single",
            permission_policy: {
                read_policy: { NoRequirement: null },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        },
        email: {
            field_type: "Single",
            permission_policy: {
                read_policy: { Distance: 1 },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        },
        bio: {
            field_type: "Single",
            permission_policy: {
                read_policy: { NoRequirement: null },
                write_policy: { Distance: 0 },
                explicit_read_policy: null,
                explicit_write_policy: null
            },
            payment_config: {
                base_multiplier: 1.0,
                trust_distance_scaling: { None: null },
                min_payment: null
            },
            field_mappers: {}
        }
    },
    payment_config: {
        base_multiplier: 1.0,
        min_payment_threshold: 0
    }
};

/**
 * Load the list of schemas from the API
 */
async function loadSchemaList() {
    const schemaListDiv = document.getElementById('schemaList');
    
    // If the element doesn't exist yet, try again later
    if (!schemaListDiv) {
        console.log('Schema list element not found, retrying in 500ms');
        setTimeout(loadSchemaList, 500);
        return;
    }
    
    try {
        utils.showLoading(schemaListDiv);
        
        const response = await utils.apiRequest('/api/schemas');
        
        if (response.data.length === 0) {
            schemaListDiv.className = 'status';
            schemaListDiv.textContent = 'No schemas loaded';
        } else {
            schemaListDiv.innerHTML = response.data.map(schema => `
                <div class="schema-item collapsed" onclick="utils.toggleSchema(this)">
                    <h3>
                        <span>${schema.name}</span>
                        <button class="remove-schema" onclick="removeSchema('${schema.name}', event)">Remove</button>
                    </h3>
                    <pre>${JSON.stringify(schema, null, 2)}</pre>
                </div>
            `).join('');
        }
    } catch (error) {
        if (schemaListDiv) {
            schemaListDiv.className = 'status error';
            schemaListDiv.textContent = error.message;
        } else {
            console.error('Error loading schemas:', error);
        }
    }
}

/**
 * Remove a schema
 * @param {string} schemaName - The name of the schema to remove
 * @param {Event} event - The click event
 */
async function removeSchema(schemaName, event) {
    event.stopPropagation(); // Prevent schema toggle
    
    if (!confirm(`Are you sure you want to remove schema "${schemaName}"?`)) {
        return;
    }

    try {
        await utils.apiRequest(`/api/schema/${encodeURIComponent(schemaName)}`, {
            method: 'DELETE'
        });
        
        utils.displayResult('Schema removed successfully');
        // Refresh schema list
        loadSchemaList();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * Load a schema from the textarea
 */
async function loadSchema() {
    const schemaInput = document.getElementById('schemaInput').value;
    
    if (!utils.isValidJSON(schemaInput)) {
        utils.displayResult('Invalid JSON format in schema', true);
        return;
    }

    try {
        await utils.apiRequest('/api/schema', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: schemaInput
        });
        
        utils.displayResult('Schema loaded successfully');
        // Refresh schema list
        loadSchemaList();
    } catch (error) {
        utils.displayResult(error.message, true);
    }
}

/**
 * Load an example schema into the textarea
 */
function loadExampleSchema() {
    document.getElementById('schemaInput').value = JSON.stringify(exampleSchema, null, 2);
}

// Export functions for use in other modules
window.schemaModule = {
    loadSchemaList,
    removeSchema,
    loadSchema,
    loadExampleSchema
};
