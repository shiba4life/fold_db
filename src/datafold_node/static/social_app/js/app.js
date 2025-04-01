document.addEventListener('DOMContentLoaded', () => {
    const client = window.datafoldClient;
    
    // UI Elements
    const loginSection = document.getElementById('loginSection');
    const socialFeed = document.getElementById('socialFeed');
    const userProfile = document.getElementById('userProfile');
    const userStatus = document.getElementById('userStatus');
    const loginForm = document.getElementById('loginForm');
    const createPostForm = document.getElementById('createPostForm');
    const profileForm = document.getElementById('profileForm');
    const postsContainer = document.getElementById('postsContainer');
    const refreshFeedBtn = document.getElementById('refreshFeed');

    // Helper Functions
    const updateUI = (connected) => {
        loginSection.style.display = connected ? 'none' : 'block';
        socialFeed.style.display = connected ? 'block' : 'none';
        userProfile.style.display = connected ? 'block' : 'none';
        userStatus.innerHTML = connected ? 
            `<span class="badge badge-success">Connected as ${client.currentUser}</span>` :
            `<span class="badge badge-primary">Not Connected</span>`;
    };

    const createPostElement = (post) => {
        const postElement = document.createElement('div');
        postElement.className = 'post';
        postElement.innerHTML = `
            <div class="post-header">
                <h3 class="post-title">${post.title}</h3>
                <div class="post-meta">
                    Posted by ${post.authorId}
                    ${post.created_at ? `on ${new Date(post.created_at).toLocaleDateString()}` : ''}
                </div>
            </div>
            <div class="post-content">${post.content}</div>
            <div class="post-actions">
                <button class="post-action-btn show-comments-btn" data-post-id="${post.id}">
                    <span class="icon">${icons.message()}</span>
                    Comments
                </button>
            </div>
            <div class="comments-section" id="comments-${post.id}" style="display: none;">
                <form class="comment-form" data-post-id="${post.id}">
                    <div class="form-group">
                        <textarea class="comment-input" placeholder="Write a comment..." required></textarea>
                    </div>
                    <button type="submit" class="btn btn-primary">Add Comment</button>
                </form>
                <div class="comments-container"></div>
            </div>
        `;

        // Add comment functionality
        const commentSection = postElement.querySelector(`#comments-${post.id}`);
        const showCommentsBtn = postElement.querySelector('.show-comments-btn');
        const commentForm = postElement.querySelector('.comment-form');
        const commentsContainer = postElement.querySelector('.comments-container');

        showCommentsBtn.addEventListener('click', async () => {
            const isHidden = commentSection.style.display === 'none';
            commentSection.style.display = isHidden ? 'block' : 'none';
            
            if (isHidden) {
                try {
                    const comments = await client.getComments(post.id);
                    commentsContainer.innerHTML = comments.map(comment => `
                        <div class="comment">
                            <div class="comment-header">
                                <span>${comment.authorId}</span>
                                <span>${new Date(comment.created_at).toLocaleDateString()}</span>
                            </div>
                            <div class="comment-content">${comment.content}</div>
                        </div>
                    `).join('');
                } catch (error) {
                    console.error('Error fetching comments:', error);
                }
            }
        });

        commentForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const content = e.target.querySelector('.comment-input').value;
            
            try {
                await client.addComment(post.id, content);
                e.target.reset();
                showCommentsBtn.click(); // Refresh comments
            } catch (error) {
                console.error('Error adding comment:', error);
            }
        });

        return postElement;
    };

    const refreshPosts = async () => {
        try {
            const posts = await client.getPosts();
            postsContainer.innerHTML = '';
            posts.forEach(post => {
                postsContainer.appendChild(createPostElement(post));
            });
        } catch (error) {
            console.error('Error refreshing posts:', error);
        }
    };

    // Event Listeners
    loginForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        const username = e.target.username.value;
        const nodeId = e.target.nodeId.value;

        try {
            await client.connect(username, nodeId);
            updateUI(true);
            await refreshPosts();
            
            // Load user profile
            const profile = await client.getProfile();
            if (profile) {
                profileForm.fullName.value = profile.fullName || '';
                profileForm.bio.value = profile.bio || '';
            }
        } catch (error) {
            console.error('Login error:', error);
        }
    });

    createPostForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        const title = e.target.postTitle.value;
        const content = e.target.postContent.value;

        try {
            await client.createPost(title, content);
            e.target.reset();
            await refreshPosts();
        } catch (error) {
            console.error('Create post error:', error);
        }
    });

    profileForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        const fullName = e.target.fullName.value;
        const bio = e.target.bio.value;

        try {
            await client.updateProfile(fullName, bio);
        } catch (error) {
            console.error('Update profile error:', error);
        }
    });

    refreshFeedBtn.addEventListener('click', refreshPosts);

    // Initialize UI
    updateUI(false);
});
