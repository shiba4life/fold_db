// Global variables
let users = [];
let posts = [];
let comments = [];
let remoteNodes = [];

// API endpoint
const API_ENDPOINT = '/api';

// Document ready function
document.addEventListener('DOMContentLoaded', function() {
    // Tab switching
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            // Remove active class from all tabs and tab contents
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            // Add active class to clicked tab and corresponding content
            tab.classList.add('active');
            document.getElementById(tab.dataset.tab).classList.add('active');
        });
    });
    
    // Fetch data from the backend
    fetchData();
    
    // Set up event listeners
    document.getElementById('create-user-form').addEventListener('submit', createUser);
    document.getElementById('create-post-form').addEventListener('submit', createPost);
    document.getElementById('add-comment-form').addEventListener('submit', addComment);
    document.getElementById('discover-nodes-btn').addEventListener('click', discoverNodes);
});

// Fetch data from the backend
async function fetchData() {
    try {
        // Fetch users
        const usersResponse = await fetch(`${API_ENDPOINT}/users`);
        if (usersResponse.ok) {
            users = await usersResponse.json();
        }
        
        // Fetch posts
        const postsResponse = await fetch(`${API_ENDPOINT}/posts`);
        if (postsResponse.ok) {
            posts = await postsResponse.json();
        }
        
        // Fetch comments
        const commentsResponse = await fetch(`${API_ENDPOINT}/comments`);
        if (commentsResponse.ok) {
            comments = await commentsResponse.json();
        }
        
        // Update UI
        displayUsers();
        displayPosts();
        populateUserDropdowns();
        populatePostDropdown();
    } catch (error) {
        console.error('Error fetching data:', error);
    }
}

// Display users
function displayUsers() {
    const usersList = document.getElementById('users-list');
    usersList.innerHTML = '';
    
    if (users.length === 0) {
        usersList.innerHTML = '<p>No users found. Create a user first.</p>';
        return;
    }
    
    users.forEach(user => {
        const userCard = document.createElement('div');
        userCard.className = 'card';
        userCard.innerHTML = `
            <div class="card-title">${user.username}</div>
            <div class="card-subtitle">${user.full_name || ''}</div>
            <div class="card-content">${user.bio || 'No bio provided'}</div>
            <div class="card-footer">ID: ${user.id}</div>
        `;
        usersList.appendChild(userCard);
    });
}

// Display posts with comments
function displayPosts() {
    const postsList = document.getElementById('posts-list');
    postsList.innerHTML = '';
    
    if (posts.length === 0) {
        postsList.innerHTML = '<p>No posts found. Create a post first.</p>';
        return;
    }
    
    posts.forEach(post => {
        const postCard = document.createElement('div');
        postCard.className = 'card';
        
        // Find the author
        const author = users.find(user => user.id === post.author_id);
        const authorName = author ? author.username : 'Unknown';
        
        // Find comments for this post
        const postComments = comments.filter(comment => comment.post_id === post.id);
        
        let commentsHtml = '';
        if (postComments.length > 0) {
            commentsHtml = '<h4>Comments:</h4>';
            postComments.forEach(comment => {
                const commentAuthor = users.find(user => user.id === comment.author_id);
                const commentAuthorName = commentAuthor ? commentAuthor.username : 'Unknown';
                
                commentsHtml += `
                    <div class="comment">
                        <div class="comment-author">${commentAuthorName}</div>
                        <div class="comment-content">${comment.content}</div>
                        <div class="comment-date">${comment.created_at}</div>
                    </div>
                `;
            });
        }
        
        postCard.innerHTML = `
            <div class="card-title">${post.title}</div>
            <div class="card-subtitle">By: ${authorName}</div>
            <div class="card-content">${post.content}</div>
            <div class="card-footer">ID: ${post.id} | Created: ${post.created_at}</div>
            ${commentsHtml}
        `;
        postsList.appendChild(postCard);
    });
}

// Populate user dropdowns
function populateUserDropdowns() {
    const postAuthorSelect = document.getElementById('post-author');
    const commentAuthorSelect = document.getElementById('comment-author');
    
    // Clear existing options
    postAuthorSelect.innerHTML = '<option value="">Select an author</option>';
    commentAuthorSelect.innerHTML = '<option value="">Select an author</option>';
    
    // Add user options
    users.forEach(user => {
        const option = document.createElement('option');
        option.value = user.id;
        option.textContent = `${user.username} (${user.full_name || ''})`;
        
        postAuthorSelect.appendChild(option.cloneNode(true));
        commentAuthorSelect.appendChild(option.cloneNode(true));
    });
}

// Populate post dropdown
function populatePostDropdown() {
    const commentPostSelect = document.getElementById('comment-post');
    
    // Clear existing options
    commentPostSelect.innerHTML = '<option value="">Select a post</option>';
    
    // Add post options
    posts.forEach(post => {
        const option = document.createElement('option');
        option.value = post.id;
        option.textContent = post.title;
        
        commentPostSelect.appendChild(option);
    });
}

// Create user
async function createUser(e) {
    e.preventDefault();
    
    const username = document.getElementById('username').value;
    const fullName = document.getElementById('full-name').value;
    const bio = document.getElementById('bio').value;
    
    try {
        const response = await fetch(`${API_ENDPOINT}/users`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                username,
                full_name: fullName,
                bio
            })
        });
        
        if (response.ok) {
            const result = await response.json();
            document.getElementById('create-user-success').textContent = `User created successfully with ID: ${result.id}`;
            document.getElementById('create-user-error').textContent = '';
            
            // Refresh data
            await fetchData();
            
            // Reset form
            document.getElementById('create-user-form').reset();
        } else {
            const error = await response.json();
            document.getElementById('create-user-error').textContent = `Error: ${error.error}`;
            document.getElementById('create-user-success').textContent = '';
        }
    } catch (error) {
        console.error('Error creating user:', error);
        document.getElementById('create-user-error').textContent = `Error: ${error.message}`;
        document.getElementById('create-user-success').textContent = '';
    }
}

// Create post
async function createPost(e) {
    e.preventDefault();
    
    const authorId = document.getElementById('post-author').value;
    const title = document.getElementById('post-title').value;
    const content = document.getElementById('post-content').value;
    
    if (!authorId) {
        document.getElementById('create-post-error').textContent = 'Please select an author.';
        document.getElementById('create-post-success').textContent = '';
        return;
    }
    
    try {
        const response = await fetch(`${API_ENDPOINT}/posts`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                title,
                content,
                author_id: authorId
            })
        });
        
        if (response.ok) {
            const result = await response.json();
            document.getElementById('create-post-success').textContent = `Post created successfully with ID: ${result.id}`;
            document.getElementById('create-post-error').textContent = '';
            
            // Refresh data
            await fetchData();
            
            // Reset form
            document.getElementById('create-post-form').reset();
        } else {
            const error = await response.json();
            document.getElementById('create-post-error').textContent = `Error: ${error.error}`;
            document.getElementById('create-post-success').textContent = '';
        }
    } catch (error) {
        console.error('Error creating post:', error);
        document.getElementById('create-post-error').textContent = `Error: ${error.message}`;
        document.getElementById('create-post-success').textContent = '';
    }
}

// Add comment
async function addComment(e) {
    e.preventDefault();
    
    const postId = document.getElementById('comment-post').value;
    const authorId = document.getElementById('comment-author').value;
    const content = document.getElementById('comment-content').value;
    
    if (!postId) {
        document.getElementById('add-comment-error').textContent = 'Please select a post.';
        document.getElementById('add-comment-success').textContent = '';
        return;
    }
    
    if (!authorId) {
        document.getElementById('add-comment-error').textContent = 'Please select an author.';
        document.getElementById('add-comment-success').textContent = '';
        return;
    }
    
    try {
        const response = await fetch(`${API_ENDPOINT}/comments`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                content,
                author_id: authorId,
                post_id: postId
            })
        });
        
        if (response.ok) {
            const result = await response.json();
            document.getElementById('add-comment-success').textContent = `Comment added successfully with ID: ${result.id}`;
            document.getElementById('add-comment-error').textContent = '';
            
            // Refresh data
            await fetchData();
            
            // Reset form
            document.getElementById('add-comment-form').reset();
        } else {
            const error = await response.json();
            document.getElementById('add-comment-error').textContent = `Error: ${error.error}`;
            document.getElementById('add-comment-success').textContent = '';
        }
    } catch (error) {
        console.error('Error adding comment:', error);
        document.getElementById('add-comment-error').textContent = `Error: ${error.message}`;
        document.getElementById('add-comment-success').textContent = '';
    }
}

// Discover remote nodes
async function discoverNodes() {
    try {
        const response = await fetch(`${API_ENDPOINT}/discover-nodes`);
        if (response.ok) {
            remoteNodes = await response.json();
            
            const remoteNodesList = document.getElementById('remote-nodes-list');
            remoteNodesList.innerHTML = '<h3>Discovered Nodes:</h3>';
            
            remoteNodes.forEach(node => {
                const nodeItem = document.createElement('div');
                nodeItem.className = 'card';
                nodeItem.innerHTML = `
                    <div class="card-title">Node ID: ${node.id}</div>
                    <div class="card-content">Address: ${node.address}</div>
                `;
                remoteNodesList.appendChild(nodeItem);
            });
            
            document.getElementById('remote-nodes-success').textContent = 'Remote nodes discovered successfully!';
            document.getElementById('remote-nodes-error').textContent = '';
        } else {
            const error = await response.json();
            document.getElementById('remote-nodes-error').textContent = `Error: ${error.error}`;
            document.getElementById('remote-nodes-success').textContent = '';
        }
    } catch (error) {
        console.error('Error discovering nodes:', error);
        document.getElementById('remote-nodes-error').textContent = `Error: ${error.message}`;
        document.getElementById('remote-nodes-success').textContent = '';
    }
}
