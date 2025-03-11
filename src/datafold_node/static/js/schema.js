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
 * Create a query for all fields in a schema
 * @param {string} schemaName - The name of the schema to query
 * @param {object} schema - The schema object
 * @returns {object} - The query operation
 */
function createQueryAllFieldsOperation(schemaName, schema) {
    // Extract all field names from the schema
    const fields = Object.keys(schema.fields || {});
    
    // Create a query operation
    return {
        type: "query",
        schema: schemaName,
        fields: fields.length > 0 ? fields : ["*"], // Use all fields or wildcard if no fields
        filter: null
    };
}

/**
 * Execute a query for all fields in a schema
 * @param {string} schemaName - The name of the schema to query
 * @param {object} schema - The schema object
 */
async function queryAllFields(schemaName, schema) {
    try {
        utils.showLoadingOverlay('Querying all fields...');
        
        const query = createQueryAllFieldsOperation(schemaName, schema);
        
        // Navigate to operations page
        window.app.navigateToPage('operations');
        
        // Switch to query tab
        window.app.switchOperationsTab('query');
        
        // Set the query in the query input
        document.getElementById('queryInput').value = JSON.stringify(query, null, 2);
        
        // Execute the query - send the operation object directly
        console.log(`Querying all fields for schema ${schemaName}: ${JSON.stringify(query)}`);
        const response = await utils.apiRequest('/api/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                operation: query
            })
        });
        
        utils.hideLoadingOverlay();
        utils.displayResult(response.data);
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.displayResult(error.message, true);
    }
}

/**
 * Load the list of schemas from the API
 */
async function loadSchemaList() {
    const schemasList = document.getElementById('schemasList');
    
    // If the element doesn't exist yet, try again later
    if (!schemasList) {
        console.log('Schema list element not found, retrying in 500ms');
        setTimeout(loadSchemaList, 500);
        return;
    }
    
    try {
        utils.showLoading(schemasList, 'Loading schemas...');
        
        const response = await utils.apiRequest('/api/schemas');
        
        if (!response.data || response.data.length === 0) {
            schemasList.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-database"></i>
                    <p>No schemas loaded</p>
                    <button class="btn primary" id="emptyStateNewSchemaBtn">
                        <i class="fas fa-plus"></i>
                        <span>Add Schema</span>
                    </button>
                </div>
            `;
            
            // Add event listener to the empty state button
            const emptyStateNewSchemaBtn = document.getElementById('emptyStateNewSchemaBtn');
            if (emptyStateNewSchemaBtn) {
                emptyStateNewSchemaBtn.addEventListener('click', window.app.openSchemaModal);
            }
        } else {
            // Create schema cards
            schemasList.innerHTML = response.data.map(schema => createSchemaCard(schema)).join('');
            
            // Add event listeners to schema cards
            response.data.forEach(schema => {
                // Query all fields button
                const queryAllFieldsBtn = document.getElementById(`queryAllFields_${schema.name}`);
                if (queryAllFieldsBtn) {
                    queryAllFieldsBtn.addEventListener('click', (event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        queryAllFields(schema.name, schema);
                    });
                }
                
                // Remove schema button
                const removeSchemaBtn = document.getElementById(`removeSchema_${schema.name}`);
                if (removeSchemaBtn) {
                    removeSchemaBtn.addEventListener('click', (event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        confirmRemoveSchema(schema.name);
                    });
                }
                
                // Toggle schema details
                const schemaCard = document.getElementById(`schema_${schema.name}`);
                if (schemaCard) {
                    schemaCard.addEventListener('click', () => {
                        toggleSchemaDetails(schema.name);
                    });
                }
            });
            
            // Update dashboard stats if on dashboard
            if (document.getElementById('schemaStatsCard')) {
                const schemaStatsCard = document.getElementById('schemaStatsCard');
                schemaStatsCard.innerHTML = `
                    <div class="stat-grid">
                        <div class="stat-item">
                            <div class="stat-label">Total Schemas</div>
                            <div class="stat-value">${response.data.length}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-label">Active Schemas</div>
                            <div class="stat-value">${response.data.length}</div>
                        </div>
                    </div>
                    <div class="card-actions">
                        <button class="btn small primary" onclick="window.app.navigateToPage('schemas')">
                            <i class="fas fa-database"></i>
                            <span>Manage Schemas</span>
                        </button>
                    </div>
                `;
            }
        }
    } catch (error) {
        if (schemasList) {
            schemasList.innerHTML = `
                <div class="error-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Error loading schemas: ${error.message}</span>
                </div>
                <div class="card-actions">
                    <button class="btn primary" onclick="schemaModule.loadSchemaList()">
                        <i class="fas fa-sync-alt"></i>
                        <span>Try Again</span>
                    </button>
                </div>
            `;
        } else {
            console.error('Error loading schemas:', error);
        }
    }
}

/**
 * Create a schema card HTML
 * @param {object} schema - The schema object
 * @returns {string} - HTML for the schema card
 */
function createSchemaCard(schema) {
    const fieldCount = Object.keys(schema.fields || {}).length;
    
    return `
        <div class="schema-card" id="schema_${schema.name}">
            <div class="schema-header">
                <h3>${schema.name}</h3>
                <div class="schema-actions">
                    <button class="btn small success" id="queryAllFields_${schema.name}">
                        <i class="fas fa-search"></i>
                        <span>Query</span>
                    </button>
                    <button class="btn small danger" id="removeSchema_${schema.name}">
                        <i class="fas fa-trash"></i>
                        <span>Remove</span>
                    </button>
                </div>
            </div>
            <div class="schema-body">
                <div class="schema-meta">
                    <div class="schema-meta-item">
                        <i class="fas fa-table"></i>
                        <span>${fieldCount} field${fieldCount !== 1 ? 's' : ''}</span>
                    </div>
                </div>
                <div class="schema-fields" id="schemaFields_${schema.name}" style="display: none;">
                    <div class="schema-fields-header">Fields</div>
                    ${Object.entries(schema.fields || {}).map(([fieldName, field]) => `
                        <div class="schema-field">
                            <div class="field-name">${fieldName}</div>
                            <div class="field-type">${field.field_type}</div>
                        </div>
                    `).join('')}
                </div>
            </div>
        </div>
    `;
}

/**
 * Toggle schema details visibility
 * @param {string} schemaName - The name of the schema
 */
function toggleSchemaDetails(schemaName) {
    const schemaFields = document.getElementById(`schemaFields_${schemaName}`);
    if (schemaFields) {
        const isVisible = schemaFields.style.display !== 'none';
        schemaFields.style.display = isVisible ? 'none' : 'block';
    }
}

/**
 * Confirm and remove a schema
 * @param {string} schemaName - The name of the schema to remove
 */
async function confirmRemoveSchema(schemaName) {
    if (!confirm(`Are you sure you want to remove schema "${schemaName}"?`)) {
        return;
    }

    try {
        utils.showLoadingOverlay(`Removing schema "${schemaName}"...`);
        
        await utils.apiRequest(`/api/schema/${encodeURIComponent(schemaName)}`, {
            method: 'DELETE'
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', `Schema "${schemaName}" removed successfully`, 'success');
        
        // Refresh schema list
        loadSchemaList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to remove schema: ${error.message}`, 'error');
    }
}

/**
 * Load a schema from the textarea
 */
async function loadSchema() {
    const schemaInput = document.getElementById('schemaInput').value;
    
    if (!utils.isValidJSON(schemaInput)) {
        utils.showNotification('Error', 'Invalid JSON format in schema', 'error');
        return;
    }

    try {
        utils.showLoadingOverlay('Loading schema...');
        
        await utils.apiRequest('/api/schema', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: schemaInput
        });
        
        utils.hideLoadingOverlay();
        utils.showNotification('Success', 'Schema loaded successfully', 'success');
        
        // Close the modal
        window.app.closeSchemaModal();
        
        // Refresh schema list
        loadSchemaList();
    } catch (error) {
        utils.hideLoadingOverlay();
        utils.showNotification('Error', `Failed to load schema: ${error.message}`, 'error');
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
    loadSchema,
    loadExampleSchema,
    queryAllFields,
    createQueryAllFieldsOperation,
    toggleSchemaDetails,
    confirmRemoveSchema
};
