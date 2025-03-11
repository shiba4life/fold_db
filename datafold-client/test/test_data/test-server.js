
      const express = require('express');
      const bodyParser = require('body-parser');
      const fs = require('fs');
      const path = require('path');

      // Initialize Express app for the DataFold Node
      const app = express();
      // Use the port passed as a command-line argument or default to 8082
      const PORT = process.argv[2] ? parseInt(process.argv[2]) : 8082;

      // Configure middleware
      app.use(bodyParser.json());
      app.use(bodyParser.urlencoded({ extended: true }));

      // In-memory storage for schemas and data
      const db = {
        schemas: {},
        data: {}
      };

      // Ensure data directory exists
      const dataDir = '/Users/tomtang/projects/engram/fold_db/datafold-client/test/test_data';
      if (!fs.existsSync(dataDir)) {
        fs.mkdirSync(dataDir, { recursive: true });
      }

      // Load existing data if available
      try {
        if (fs.existsSync(path.join(dataDir, 'schemas.json'))) {
          db.schemas = JSON.parse(fs.readFileSync(path.join(dataDir, 'schemas.json'), 'utf8'));
        }
        if (fs.existsSync(path.join(dataDir, 'data.json'))) {
          db.data = JSON.parse(fs.readFileSync(path.join(dataDir, 'data.json'), 'utf8'));
        }
        console.log('Loaded existing data');
      } catch (error) {
        console.error('Error loading existing data:', error);
      }

      // Save data to disk
      function saveData() {
        try {
          fs.writeFileSync(path.join(dataDir, 'schemas.json'), JSON.stringify(db.schemas, null, 2));
          fs.writeFileSync(path.join(dataDir, 'data.json'), JSON.stringify(db.data, null, 2));
        } catch (error) {
          console.error('Error saving data:', error);
        }
      }

      // List all schemas
      app.get('/api/schemas', (req, res) => {
        res.json({ schemas: Object.keys(db.schemas) });
      });

      // Create or update a schema
      app.post('/api/schema', (req, res) => {
        const schema = req.body;
        
        if (!schema || !schema.name || !schema.fields) {
          return res.status(400).json({ 
            success: false, 
            error: 'Invalid schema format. Schema must have name and fields.' 
          });
        }
        
        db.schemas[schema.name] = schema;
        
        // Initialize data collection for this schema if it doesn't exist
        if (!db.data[schema.name]) {
          db.data[schema.name] = [];
        }
        
        saveData();
        
        res.json({ success: true });
      });

      // Delete a schema
      app.delete('/api/schema/:name', (req, res) => {
        const { name } = req.params;
        
        if (db.schemas[name]) {
          delete db.schemas[name];
          delete db.data[name];
          saveData();
          res.json({ success: true });
        } else {
          res.status(404).json({ success: false, error: 'Schema not found' });
        }
      });

      // Execute operations (query/mutation)
      app.post('/api/execute', (req, res) => {
        const { operation } = req.body;
        
        if (!operation) {
          return res.status(400).json({ success: false, error: 'Operation is required' });
        }
        
        let parsedOperation;
        try {
          parsedOperation = typeof operation === 'string' ? JSON.parse(operation) : operation;
        } catch (error) {
          return res.status(400).json({ success: false, error: 'Invalid operation format' });
        }
        
        const { type, schema } = parsedOperation;
        
        if (!type || !schema) {
          return res.status(400).json({ success: false, error: 'Operation type and schema are required' });
        }
        
        if (!db.schemas[schema]) {
          return res.status(404).json({ success: false, error: 'Schema not found' });
        }
        
        // Handle query operation
        if (type === 'query') {
          const { fields, filter } = parsedOperation;
          
          if (!fields || !Array.isArray(fields)) {
            return res.status(400).json({ success: false, error: 'Fields must be an array' });
          }
          
          let results = db.data[schema] || [];
          
          // Apply filter if provided
          if (filter && Object.keys(filter).length > 0) {
            results = results.filter(item => {
              return Object.entries(filter).every(([key, value]) => item[key] === value);
            });
          }
          
          // Project only requested fields
          const projectedResults = results.map(item => {
            const result = {};
            fields.forEach(field => {
              if (item[field] !== undefined) {
                result[field] = item[field];
              }
            });
            return result;
          });
          
          return res.json({ results: projectedResults, count: projectedResults.length });
        }
        
        // Handle mutation operation
        if (type === 'mutation') {
          const { operation: mutationType, data, filter } = parsedOperation;
          
          if (!mutationType) {
            return res.status(400).json({ success: false, error: 'Mutation operation type is required' });
          }
          
          // Handle create operation
          if (mutationType === 'create') {
            if (!data) {
              return res.status(400).json({ success: false, error: 'Data is required for create operation' });
            }
            
            db.data[schema].push(data);
            saveData();
            
            return res.json({ success: true, affected_count: 1 });
          }
          
          // Handle update operation
          if (mutationType === 'update') {
            if (!data) {
              return res.status(400).json({ success: false, error: 'Data is required for update operation' });
            }
            
            if (!filter || Object.keys(filter).length === 0) {
              return res.status(400).json({ success: false, error: 'Filter is required for update operation' });
            }
            
            let affectedCount = 0;
            
            db.data[schema] = db.data[schema].map(item => {
              const matches = Object.entries(filter).every(([key, value]) => item[key] === value);
              
              if (matches) {
                affectedCount++;
                return { ...item, ...data };
              }
              
              return item;
            });
            
            saveData();
            
            return res.json({ success: true, affected_count: affectedCount });
          }
          
          // Handle delete operation
          if (mutationType === 'delete') {
            if (!filter || Object.keys(filter).length === 0) {
              // Delete all records
              const affectedCount = db.data[schema].length;
              db.data[schema] = [];
              saveData();
              
              return res.json({ success: true, affected_count: affectedCount });
            }
            
            const originalLength = db.data[schema].length;
            
            db.data[schema] = db.data[schema].filter(item => {
              return !Object.entries(filter).every(([key, value]) => item[key] === value);
            });
            
            const affectedCount = originalLength - db.data[schema].length;
            saveData();
            
            return res.json({ success: true, affected_count: affectedCount });
          }
          
          return res.status(400).json({ success: false, error: 'Unsupported mutation operation' });
        }
        
        return res.status(400).json({ success: false, error: 'Unsupported operation type' });
      });

      // Load schema from file
      app.post('/api/schema/load/file', (req, res) => {
        const { file_path } = req.body;
        
        if (!file_path) {
          return res.status(400).json({ 
            success: false, 
            error: 'File path is required' 
          });
        }
        
        try {
          // For testing purposes, we'll just create a dummy schema
          const schemaName = path.basename(file_path, '.json');
          const dummySchema = {
            name: schemaName,
            fields: {
              id: {
                permission_policy: {
                  read_policy: { NoRequirement: null },
                  write_policy: { NoRequirement: null }
                },
                payment_config: {
                  base_multiplier: 1.0
                },
                field_mappers: {}
              }
            }
          };
          
          db.schemas[schemaName] = dummySchema;
          
          // Initialize data collection for this schema if it doesn't exist
          if (!db.data[schemaName]) {
            db.data[schemaName] = [];
          }
          
          saveData();
          
          return res.json({ 
            data: {
              schema_name: schemaName,
              message: 'Schema loaded successfully'
            }
          });
        } catch (error) {
          return res.status(500).json({ 
            success: false, 
            error: `Failed to load schema: ${error.message}` 
          });
        }
      });
      
      // Load schema from JSON
      app.post('/api/schema/load/json', (req, res) => {
        const { schema_json } = req.body;
        
        if (!schema_json || !schema_json.name || !schema_json.fields) {
          return res.status(400).json({ 
            success: false, 
            error: 'Invalid schema format. Schema must have name and fields.' 
          });
        }
        
        try {
          const schemaName = schema_json.name;
          
          db.schemas[schemaName] = schema_json;
          
          // Initialize data collection for this schema if it doesn't exist
          if (!db.data[schemaName]) {
            db.data[schemaName] = [];
          }
          
          saveData();
          
          return res.json({ 
            data: {
              schema_name: schemaName,
              message: 'Schema loaded successfully'
            }
          });
        } catch (error) {
          return res.status(500).json({ 
            success: false, 
            error: `Failed to load schema: ${error.message}` 
          });
        }
      });

      // Network status endpoint (mock)
      app.get('/api/network/status', (req, res) => {
        res.json({
          running: true,
          node_count: 1,
          connection_count: 0,
          local_node_id: 'test-node'
        });
      });

      // Start the server
      app.listen(PORT, () => {
        console.log(`DataFold Node test server running at http://localhost:${PORT}`);
      });
