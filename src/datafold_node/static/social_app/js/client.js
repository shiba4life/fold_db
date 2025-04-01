class DataFoldClient {
    constructor() {
        this.appId = 'social-app';
        this.connected = false;
        this.currentUser = null;
        this.nodeId = null;
    }

    async connect(username, nodeId) {
        try {
            // Initialize connection with the DataFold node
            const response = await fetch('/api/connect', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    appId: this.appId,
                    username,
                    nodeId
                })
            });

            if (!response.ok) {
                throw new Error('Failed to connect to DataFold node');
            }

            const data = await response.json();
            this.connected = true;
            this.currentUser = username;
            this.nodeId = nodeId;

            return data;
        } catch (error) {
            console.error('Connection error:', error);
            throw error;
        }
    }

    async createPost(title, content) {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        try {
            const response = await fetch('/api/posts', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    title,
                    content,
                    authorId: this.currentUser,
                    nodeId: this.nodeId
                })
            });

            if (!response.ok) {
                throw new Error('Failed to create post');
            }

            return await response.json();
        } catch (error) {
            console.error('Create post error:', error);
            throw error;
        }
    }

    async getPosts() {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        try {
            const response = await fetch(`/api/posts?nodeId=${this.nodeId}`);
            
            if (!response.ok) {
                throw new Error('Failed to fetch posts');
            }

            return await response.json();
        } catch (error) {
            console.error('Fetch posts error:', error);
            throw error;
        }
    }

    async updateProfile(fullName, bio) {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        try {
            const response = await fetch('/api/profile', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    username: this.currentUser,
                    fullName,
                    bio,
                    nodeId: this.nodeId
                })
            });

            if (!response.ok) {
                throw new Error('Failed to update profile');
            }

            return await response.json();
        } catch (error) {
            console.error('Update profile error:', error);
            throw error;
        }
    }

    async getProfile(username = null) {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        const targetUser = username || this.currentUser;

        try {
            const response = await fetch(`/api/profile/${targetUser}?nodeId=${this.nodeId}`);
            
            if (!response.ok) {
                throw new Error('Failed to fetch profile');
            }

            return await response.json();
        } catch (error) {
            console.error('Fetch profile error:', error);
            throw error;
        }
    }

    async addComment(postId, content) {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        try {
            const response = await fetch('/api/comments', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    postId,
                    content,
                    authorId: this.currentUser,
                    nodeId: this.nodeId
                })
            });

            if (!response.ok) {
                throw new Error('Failed to add comment');
            }

            return await response.json();
        } catch (error) {
            console.error('Add comment error:', error);
            throw error;
        }
    }

    async getComments(postId) {
        if (!this.connected) throw new Error('Not connected to DataFold node');

        try {
            const response = await fetch(`/api/comments/${postId}?nodeId=${this.nodeId}`);
            
            if (!response.ok) {
                throw new Error('Failed to fetch comments');
            }

            return await response.json();
        } catch (error) {
            console.error('Fetch comments error:', error);
            throw error;
        }
    }

    disconnect() {
        this.connected = false;
        this.currentUser = null;
        this.nodeId = null;
    }
}

// Create a global instance
window.datafoldClient = new DataFoldClient();
