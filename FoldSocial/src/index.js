const express = require('express');
const bodyParser = require('body-parser');
const path = require('path');
const { DataFoldClient } = require('datafold-client');

// Initialize Express app
const app = express();
const PORT = process.env.PORT || 3000;

// Configure middleware
app.use(bodyParser.urlencoded({ extended: true }));
app.use(bodyParser.json());
app.use(express.static(path.join(__dirname, '../public')));

// Set up view engine
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'views'));

// Initialize DataFold client
const client = new DataFoldClient({
  baseUrl: process.env.DATAFOLD_API_URL || 'http://localhost:8080',
});

// Check if DataFold node is running
async function checkDataFoldNode() {
  try {
    await client.listSchemas();
    return true;
  } catch (error) {
    console.error('Error connecting to DataFold node:', error.message);
    return false;
  }
}

// Define the Post schema if it doesn't exist
async function ensurePostSchema() {
  try {
    const schemas = await client.listSchemas();
    
    if (!schemas.includes('Post')) {
      console.log('Creating Post schema...');
      
      const postSchema = {
        name: 'Post',
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
          },
          content: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          },
          author: {
            permission_policy: {
              read_policy: { NoRequirement: null },
              write_policy: { NoRequirement: null }
            },
            payment_config: {
              base_multiplier: 1.0
            },
            field_mappers: {}
          },
          timestamp: {
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
      
      await client.createSchema(postSchema);
      console.log('Post schema created successfully');
    } else {
      console.log('Post schema already exists');
    }
  } catch (error) {
    console.error('Error ensuring Post schema:', error);
  }
}

// Routes
app.get('/', async (req, res) => {
  try {
    const posts = await client.find('Post', ['id', 'content', 'author', 'timestamp'], {});
    
    // Sort posts by timestamp in descending order (newest first)
    posts.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
    
    res.render('index', { posts });
  } catch (error) {
    console.error('Error fetching posts:', error);
    res.render('index', { posts: [], error: 'Failed to fetch posts' });
  }
});

app.post('/posts', async (req, res) => {
  try {
    const { content, author } = req.body;
    
    if (!content || !author) {
      return res.status(400).json({ error: 'Content and author are required' });
    }
    
    const postData = {
      id: Date.now().toString(), // Simple ID generation
      content,
      author,
      timestamp: new Date().toISOString()
    };
    
    const result = await client.create('Post', postData);
    
    if (result.success) {
      res.redirect('/');
    } else {
      res.status(500).json({ error: result.error || 'Failed to create post' });
    }
  } catch (error) {
    console.error('Error creating post:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Start the server
app.listen(PORT, async () => {
  console.log(`FoldSocial app listening at http://localhost:${PORT}`);
  
  // Check if DataFold node is running
  const isNodeRunning = await checkDataFoldNode();
  
  if (!isNodeRunning) {
    console.error('\x1b[31m%s\x1b[0m', '⚠️  DataFold node is not running!');
    console.error('\x1b[31m%s\x1b[0m', 'Please start the DataFold node using:');
    console.error('\x1b[33m%s\x1b[0m', './setup_sandbox_local.sh');
    console.error('\x1b[31m%s\x1b[0m', 'The application will continue running, but functionality will be limited.');
    return;
  }
  
  console.log('\x1b[32m%s\x1b[0m', '✅ Connected to DataFold node successfully!');
  
  // Ensure the Post schema exists
  await ensurePostSchema();
});
