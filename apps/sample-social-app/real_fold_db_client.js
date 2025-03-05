/**
 * Real FoldDB Client for Social App
 * 
 * This module provides a client for interacting with a real DataFold node.
 * It handles schema loading, queries, and mutations by connecting to the DataFold node API.
 */

const fs = require('fs');
const path = require('path');
const fetch = require('node-fetch');

class RealFoldDBClient {
  constructor(nodeUrl = 'http://localhost:8080') {
    this.nodeUrl = nodeUrl;
    this.apiUrl = `${nodeUrl}/api`;
    this.schemas = {};
    this.initialized = false;
  }
  
  /**
   * Initialize the FoldDB client
   * @param {string} schemasDir - Directory containing schema files
   * @returns {Promise<void>}
   */
  async initialize(schemasDir) {
    console.log(`Initializing FoldDB client with schemas from ${schemasDir}`);
    
    try {
      // Check if DataFold node is running
      try {
        const response = await fetch(`${this.apiUrl}/schemas`);
        if (!response.ok) {
          throw new Error(`Failed to connect to DataFold node: ${response.statusText}`);
        }
      } catch (error) {
        throw new Error(`DataFold node is not running or not accessible at ${this.nodeUrl}: ${error.message}`);
      }
      
      // Load schemas from directory
      const schemaFiles = fs.readdirSync(schemasDir);
      
      for (const file of schemaFiles) {
        if (file.endsWith('.json')) {
          const schemaPath = path.join(schemasDir, file);
          const schemaContent = fs.readFileSync(schemaPath, 'utf8');
          const schema = JSON.parse(schemaContent);
          
          // Store schema by name
          const schemaName = path.basename(file, '.json');
          this.schemas[schemaName] = schema;
          
          // Register schema with DataFold node
          await this.registerSchema(schemaName, schema);
          
          console.log(`Loaded and registered schema: ${schemaName}`);
        }
      }
      
      this.initialized = true;
      console.log('FoldDB client initialized successfully');
    } catch (error) {
      console.error('Error initializing FoldDB client:', error);
      throw error;
    }
  }
  
  /**
   * Register a schema with the DataFold node
   * @param {string} schemaName - Schema name
   * @param {Object} schema - Schema definition
   * @returns {Promise<void>}
   */
  async registerSchema(schemaName, schema) {
    try {
      const response = await fetch(`${this.apiUrl}/schema`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          name: schemaName,
          schema: schema
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(`Failed to register schema: ${errorData.error || response.statusText}`);
      }
      
      const result = await response.json();
      console.log(`Schema registered: ${result.name}`);
    } catch (error) {
      console.error(`Error registering schema ${schemaName}:`, error);
      throw error;
    }
  }
  
  /**
   * Execute a query operation
   * @param {Object} operation - Query operation
   * @returns {Promise<Array>} - Query results
   */
  async executeQuery(operation) {
    if (!this.initialized) {
      throw new Error('FoldDB client not initialized');
    }
    
    console.log(`Executing query on schema: ${operation.schema}`);
    
    try {
      const response = await fetch(`${this.apiUrl}/execute`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          operation_type: 'query',
          schema: operation.schema,
          fields: operation.fields || [],
          filter: operation.filter || {},
          sort: operation.sort || {}
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(`Query execution failed: ${errorData.error || response.statusText}`);
      }
      
      const result = await response.json();
      return result.data || [];
    } catch (error) {
      console.error('Error executing query:', error);
      throw error;
    }
  }
  
  /**
   * Execute a mutation operation
   * @param {Object} operation - Mutation operation
   * @returns {Promise<Object>} - Mutation result
   */
  async executeMutation(operation) {
    if (!this.initialized) {
      throw new Error('FoldDB client not initialized');
    }
    
    console.log(`Executing mutation on schema: ${operation.schema}`);
    
    try {
      const response = await fetch(`${this.apiUrl}/execute`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          operation_type: 'mutation',
          schema: operation.schema,
          mutation_type: operation.mutation_type,
          data: operation.data || {}
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(`Mutation execution failed: ${errorData.error || response.statusText}`);
      }
      
      const result = await response.json();
      return { success: true, data: result.data };
    } catch (error) {
      console.error('Error executing mutation:', error);
      throw error;
    }
  }
  
  /**
   * Get schema by name
   * @param {string} schemaName - Schema name
   * @returns {Object|null} - Schema object or null if not found
   */
  getSchema(schemaName) {
    return this.schemas[schemaName] || null;
  }
  
  /**
   * Validate data against schema
   * @param {string} schemaName - Schema name
   * @param {Object} data - Data to validate
   * @returns {boolean} - True if valid, false otherwise
   */
  validateAgainstSchema(schemaName, data) {
    const schema = this.getSchema(schemaName);
    if (!schema) {
      return false;
    }
    
    // Simple validation - check if required fields exist
    for (const [fieldName, fieldDef] of Object.entries(schema.fields)) {
      if (fieldDef.required && !(fieldName in data)) {
        return false;
      }
    }
    
    return true;
  }
}

// Export a singleton instance
const realFoldDBClient = new RealFoldDBClient();
module.exports = realFoldDBClient;
