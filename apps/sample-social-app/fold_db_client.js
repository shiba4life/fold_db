/**
 * FoldDB Client for Social App
 * 
 * This module provides a client for interacting with FoldDB.
 * It handles schema loading, queries, and mutations.
 */

const fs = require('fs');
const path = require('path');

// Import FoldDB (in a real implementation, this would be a proper import)
// For now, we'll simulate the FoldDB API
class FoldDBClient {
  constructor(dbPath) {
    this.dbPath = dbPath;
    this.schemas = {};
    this.initialized = false;
    
    // Create data directory if it doesn't exist
    if (!fs.existsSync(dbPath)) {
      fs.mkdirSync(dbPath, { recursive: true });
    }
    
    // Initialize storage files
    this.postsFile = path.join(dbPath, 'posts.json');
    this.profilesFile = path.join(dbPath, 'profiles.json');
    
    if (!fs.existsSync(this.postsFile)) {
      fs.writeFileSync(this.postsFile, JSON.stringify([], null, 2));
    }
    
    if (!fs.existsSync(this.profilesFile)) {
      fs.writeFileSync(this.profilesFile, JSON.stringify([], null, 2));
    }
  }
  
  /**
   * Initialize the FoldDB client
   * @param {string} schemasDir - Directory containing schema files
   * @returns {Promise<void>}
   */
  async initialize(schemasDir) {
    console.log(`Initializing FoldDB client with schemas from ${schemasDir}`);
    
    try {
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
          
          console.log(`Loaded schema: ${schemaName}`);
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
      let results = [];
      
      if (operation.schema === 'post') {
        // Read posts from storage
        const postsData = fs.readFileSync(this.postsFile, 'utf8');
        results = JSON.parse(postsData);
        
        // Apply filter if provided
        if (operation.filter) {
          if (operation.filter.id) {
            results = results.filter(post => post.id === operation.filter.id);
          }
          if (operation.filter.author && operation.filter.author.id) {
            results = results.filter(post => post.author.id === operation.filter.author.id);
          }
        }
        
        // Apply sort if provided
        if (operation.sort) {
          const { field, order } = operation.sort;
          results.sort((a, b) => {
            if (order === 'asc') {
              return a[field] > b[field] ? 1 : -1;
            } else {
              return a[field] < b[field] ? 1 : -1;
            }
          });
        }
        
        // Filter fields if specified
        if (operation.fields && operation.fields.length > 0) {
          results = results.map(post => {
            const filteredPost = {};
            operation.fields.forEach(field => {
              filteredPost[field] = post[field];
            });
            return filteredPost;
          });
        }
      } else if (operation.schema === 'user-profile') {
        // Read profiles from storage
        const profilesData = fs.readFileSync(this.profilesFile, 'utf8');
        results = JSON.parse(profilesData);
        
        // Apply filter if provided
        if (operation.filter) {
          if (operation.filter.id) {
            results = results.filter(profile => profile.id === operation.filter.id);
          }
        }
        
        // Filter fields if specified
        if (operation.fields && operation.fields.length > 0) {
          results = results.map(profile => {
            const filteredProfile = {};
            operation.fields.forEach(field => {
              filteredProfile[field] = profile[field];
            });
            return filteredProfile;
          });
        }
      }
      
      return results;
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
      if (operation.schema === 'post') {
        if (operation.mutation_type === 'create') {
          // Read existing posts
          const postsData = fs.readFileSync(this.postsFile, 'utf8');
          const posts = JSON.parse(postsData);
          
          // Create new post
          const newPost = {
            id: Date.now().toString(),
            ...operation.data,
            timestamp: operation.data.timestamp || new Date().toISOString()
          };
          
          // Add to posts array
          posts.unshift(newPost);
          
          // Write back to storage
          fs.writeFileSync(this.postsFile, JSON.stringify(posts, null, 2));
          
          // Log the operation to simulate FoldDB's atom-based storage
          console.log(`[FoldDB] Created atom for post ${newPost.id}`);
          
          return { success: true, data: newPost };
        } else if (operation.mutation_type === 'update') {
          // Read existing posts
          const postsData = fs.readFileSync(this.postsFile, 'utf8');
          const posts = JSON.parse(postsData);
          
          // Find post to update
          const postId = operation.data.id;
          const postIndex = posts.findIndex(post => post.id === postId);
          
          if (postIndex === -1) {
            throw new Error('Post not found');
          }
          
          // Update post
          posts[postIndex] = { ...posts[postIndex], ...operation.data };
          
          // Write back to storage
          fs.writeFileSync(this.postsFile, JSON.stringify(posts, null, 2));
          
          // Log the operation to simulate FoldDB's atom-based storage
          console.log(`[FoldDB] Updated atom for post ${postId}`);
          
          return { success: true, data: posts[postIndex] };
        } else if (operation.mutation_type === 'delete') {
          // Read existing posts
          const postsData = fs.readFileSync(this.postsFile, 'utf8');
          const posts = JSON.parse(postsData);
          
          // Find post to delete
          const postId = operation.data.id;
          const newPosts = posts.filter(post => post.id !== postId);
          
          if (newPosts.length === posts.length) {
            throw new Error('Post not found');
          }
          
          // Write back to storage
          fs.writeFileSync(this.postsFile, JSON.stringify(newPosts, null, 2));
          
          // Log the operation to simulate FoldDB's atom-based storage
          console.log(`[FoldDB] Deleted atom for post ${postId}`);
          
          return { success: true };
        }
      } else if (operation.schema === 'user-profile') {
        // Similar implementation for user profiles
        // ...
      }
      
      throw new Error(`Unsupported mutation: ${operation.schema}/${operation.mutation_type}`);
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
const foldDBClient = new FoldDBClient(path.join(__dirname, 'data'));
module.exports = foldDBClient;
