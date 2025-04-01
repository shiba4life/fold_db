// Social App API Server
const express = require('express');
const bodyParser = require('body-parser');
const { v4: uuidv4 } = require('uuid');

// Create Express router
const router = express.Router();
router.use(bodyParser.json());

// In-memory data store (in a real implementation, this would use the DataFold SDK)
const users = new Map();
const posts = [];
const comments = new Map();
const sessions = new Map();

// Initialize with some sample data
function initSampleData() {
    // Sample users
    const user1 = {
        id: 'user1',
        username: 'alice',
        fullName: 'Alice Johnson',
        bio: 'Software engineer and blockchain enthusiast',
        created_at: new Date().toISOString()
    };
    
    const user2 = {
        id: 'user2',
        username: 'bob',
        fullName: 'Bob Smith',
        bio: 'Decentralized systems researcher',
        created_at: new Date().toISOString()
    };
    
    users.set('alice', user1);
    users.set('bob', user2);
    
    // Sample posts
    posts.push({
        id: 'post1',
        title: 'Introduction to DataFold',
        content: 'DataFold is a decentralized database system that allows for secure, private data sharing across a network of nodes.',
        authorId: 'alice',
        created_at: new Date().toISOString()
    });
    
    posts.push({
        id: 'post2',
        title: 'Building Social Apps on DataFold',
        content: 'The sandboxed architecture allows for secure social applications that respect user privacy and data ownership.',
        authorId: 'bob',
        created_at: new Date().toISOString()
    });
    
    // Sample comments
    const post1Comments = [
        {
            id: 'comment1',
            content: 'Great introduction! Looking forward to learning more.',
            authorId: 'bob',
            postId: 'post1',
            created_at: new Date().toISOString()
        }
    ];
    
    const post2Comments = [
        {
            id: 'comment2',
            content: 'This architecture looks promising for privacy-focused applications.',
            authorId: 'alice',
            postId: 'post2',
            created_at: new Date().toISOString()
        }
    ];
    
    comments.set('post1', post1Comments);
    comments.set('post2', post2Comments);
}

// Initialize sample data
initSampleData();

// API Routes

// Connect to DataFold node
router.post('/api/connect', (req, res) => {
    const { appId, username, nodeId } = req.body;
    
    if (!appId || !username || !nodeId) {
        return res.status(400).json({ error: 'Missing required fields' });
    }
    
    // Check if user exists, create if not
    if (!users.has(username)) {
        const newUser = {
            id: uuidv4(),
            username,
            fullName: '',
            bio: '',
            created_at: new Date().toISOString()
        };
        users.set(username, newUser);
    }
    
    // Create a session
    const sessionId = uuidv4();
    sessions.set(sessionId, { appId, username, nodeId });
    
    res.json({
        success: true,
        sessionId,
        message: `Connected to node ${nodeId} as ${username}`
    });
});

// Create a post
router.post('/api/posts', (req, res) => {
    const { title, content, authorId, nodeId } = req.body;
    
    if (!title || !content || !authorId) {
        return res.status(400).json({ error: 'Missing required fields' });
    }
    
    const newPost = {
        id: uuidv4(),
        title,
        content,
        authorId,
        created_at: new Date().toISOString()
    };
    
    posts.push(newPost);
    
    res.json({
        success: true,
        post: newPost
    });
});

// Get all posts
router.get('/api/posts', (req, res) => {
    // Sort posts by creation date (newest first)
    const sortedPosts = [...posts].sort((a, b) => 
        new Date(b.created_at) - new Date(a.created_at)
    );
    
    res.json(sortedPosts);
});

// Update user profile
router.put('/api/profile', (req, res) => {
    const { username, fullName, bio, nodeId } = req.body;
    
    if (!username) {
        return res.status(400).json({ error: 'Missing username' });
    }
    
    if (!users.has(username)) {
        return res.status(404).json({ error: 'User not found' });
    }
    
    const user = users.get(username);
    
    if (fullName !== undefined) {
        user.fullName = fullName;
    }
    
    if (bio !== undefined) {
        user.bio = bio;
    }
    
    users.set(username, user);
    
    res.json({
        success: true,
        profile: user
    });
});

// Get user profile
router.get('/api/profile/:username', (req, res) => {
    const { username } = req.params;
    
    if (!users.has(username)) {
        return res.status(404).json({ error: 'User not found' });
    }
    
    res.json(users.get(username));
});

// Add a comment to a post
router.post('/api/comments', (req, res) => {
    const { postId, content, authorId, nodeId } = req.body;
    
    if (!postId || !content || !authorId) {
        return res.status(400).json({ error: 'Missing required fields' });
    }
    
    // Find the post
    const post = posts.find(p => p.id === postId);
    if (!post) {
        return res.status(404).json({ error: 'Post not found' });
    }
    
    const newComment = {
        id: uuidv4(),
        content,
        authorId,
        postId,
        created_at: new Date().toISOString()
    };
    
    // Add comment to the comments map
    if (!comments.has(postId)) {
        comments.set(postId, []);
    }
    
    comments.get(postId).push(newComment);
    
    res.json({
        success: true,
        comment: newComment
    });
});

// Get comments for a post
router.get('/api/comments/:postId', (req, res) => {
    const { postId } = req.params;
    
    if (!comments.has(postId)) {
        return res.json([]);
    }
    
    // Sort comments by creation date (newest first)
    const sortedComments = [...comments.get(postId)].sort((a, b) => 
        new Date(b.created_at) - new Date(a.created_at)
    );
    
    res.json(sortedComments);
});

module.exports = router;
