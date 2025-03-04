/**
 * API Client for Social App
 * 
 * This module provides a clean interface for interacting with the backend API.
 * It handles all API requests and responses, providing a consistent interface
 * for the app to use.
 */

class ApiClient {
  constructor(baseUrl = '/api') {
    this.baseUrl = baseUrl;
  }

  /**
   * Execute an operation against the API
   * @param {Object} operation - The operation to execute
   * @returns {Promise<Object>} - The API response
   */
  async executeOperation(operation) {
    try {
      const response = await fetch(`${this.baseUrl}/execute`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ operation: JSON.stringify(operation) })
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || `API request failed with status ${response.status}`);
      }

      return response.json();
    } catch (error) {
      console.error('API request failed:', error);
      throw error;
    }
  }

  /**
   * Get all posts
   * @returns {Promise<Array>} - Array of posts
   */
  async getPosts() {
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "content", "timestamp", "likes", "comments", "author"],
      sort: { field: "timestamp", order: "desc" }
    };

    try {
      const result = await this.executeOperation(queryOperation);
      if (result && result.data && Array.isArray(result.data.results)) {
        return result.data.results;
      }
      throw new Error('Invalid response format from API');
    } catch (error) {
      console.error('Error getting posts:', error);
      throw error;
    }
  }

  /**
   * Get posts by a specific user
   * @param {string} userId - The user ID
   * @returns {Promise<Array>} - Array of posts
   */
  async getUserPosts(userId) {
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "content", "timestamp", "likes", "comments", "author"],
      filter: { author: { id: userId } },
      sort: { field: "timestamp", order: "desc" }
    };

    try {
      const result = await this.executeOperation(queryOperation);
      if (result && result.data && Array.isArray(result.data.results)) {
        return result.data.results;
      }
      throw new Error('Invalid response format from API');
    } catch (error) {
      console.error('Error getting user posts:', error);
      throw error;
    }
  }

  /**
   * Create a new post
   * @param {Object} postData - The post data
   * @returns {Promise<Object>} - The created post
   */
  async createPost(postData) {
    const createOperation = {
      type: "mutation",
      schema: "post",
      data: postData,
      mutation_type: "create"
    };

    try {
      const result = await this.executeOperation(createOperation);
      if (result && result.success && result.data) {
        return result.data;
      }
      throw new Error('Failed to create post');
    } catch (error) {
      console.error('Error creating post:', error);
      throw error;
    }
  }

  /**
   * Update a post
   * @param {string} postId - The post ID
   * @param {Object} updateData - The data to update
   * @returns {Promise<Object>} - The updated post
   */
  async updatePost(postId, updateData) {
    const updateOperation = {
      type: "mutation",
      schema: "post",
      data: {
        id: postId,
        ...updateData
      },
      mutation_type: "update"
    };

    try {
      const result = await this.executeOperation(updateOperation);
      if (result && result.success) {
        return result.data;
      }
      throw new Error('Failed to update post');
    } catch (error) {
      console.error('Error updating post:', error);
      throw error;
    }
  }

  /**
   * Get user profile
   * @returns {Promise<Object>} - The user profile
   */
  async getProfile() {
    const queryOperation = {
      type: "query",
      schema: "user-profile",
      fields: ["id", "username", "bio"]
    };

    try {
      const result = await this.executeOperation(queryOperation);
      if (result && result.data && Array.isArray(result.data.results) && result.data.results.length > 0) {
        return result.data.results[0];
      }
      throw new Error('Invalid response format from API or no profile found');
    } catch (error) {
      console.error('Error getting profile:', error);
      throw error;
    }
  }

  /**
   * Get friends list
   * @returns {Promise<Array>} - Array of friends
   */
  async getFriends() {
    const queryOperation = {
      type: "query",
      schema: "user-profile",
      fields: ["id", "username", "bio"],
      filter: { isFriend: true }
    };

    try {
      const result = await this.executeOperation(queryOperation);
      if (result && result.data && Array.isArray(result.data.results)) {
        return result.data.results;
      }
      throw new Error('Invalid response format from API');
    } catch (error) {
      console.error('Error getting friends:', error);
      throw error;
    }
  }

  /**
   * Get friend suggestions
   * @returns {Promise<Array>} - Array of friend suggestions
   */
  async getFriendSuggestions() {
    const queryOperation = {
      type: "query",
      schema: "user-profile",
      fields: ["id", "username", "bio"],
      filter: { isFriend: false, isSuggestion: true }
    };

    try {
      const result = await this.executeOperation(queryOperation);
      if (result && result.data && Array.isArray(result.data.results)) {
        return result.data.results;
      }
      throw new Error('Invalid response format from API');
    } catch (error) {
      console.error('Error getting friend suggestions:', error);
      throw error;
    }
  }

  /**
   * Add a friend
   * @param {string} userId - The user ID to add as a friend
   * @returns {Promise<Object>} - The result
   */
  async addFriend(userId) {
    const updateOperation = {
      type: "mutation",
      schema: "user-profile",
      data: {
        id: userId,
        isFriend: true
      },
      mutation_type: "update"
    };

    try {
      const result = await this.executeOperation(updateOperation);
      if (result && result.success) {
        return result;
      }
      throw new Error('Failed to add friend');
    } catch (error) {
      console.error('Error adding friend:', error);
      throw error;
    }
  }

  /**
   * Like a post
   * @param {string} postId - The post ID
   * @param {Object} userInfo - The user info for the like
   * @returns {Promise<Object>} - The result
   */
  async likePost(postId, userInfo) {
    // First get the current post to update its likes
    const queryOperation = {
      type: "query",
      schema: "post",
      fields: ["id", "likes"],
      filter: { id: postId }
    };

    try {
      const queryResult = await this.executeOperation(queryOperation);
      if (!queryResult || !queryResult.data || !Array.isArray(queryResult.data.results) || queryResult.data.results.length === 0) {
        throw new Error('Post not found');
      }

      const post = queryResult.data.results[0];
      const likes = post.likes || [];

      // Check if the user already liked the post
      const alreadyLiked = likes.some(like => like.id === userInfo.id);
      if (alreadyLiked) {
        return { success: true, message: 'Post already liked' };
      }

      // Add the user to the likes array
      likes.push(userInfo);

      // Update the post
      const updateOperation = {
        type: "mutation",
        schema: "post",
        data: {
          id: postId,
          likes: likes
        },
        mutation_type: "update"
      };

      const updateResult = await this.executeOperation(updateOperation);
      if (updateResult && updateResult.success) {
        return updateResult;
      }
      throw new Error('Failed to like post');
    } catch (error) {
      console.error('Error liking post:', error);
      throw error;
    }
  }
}

// Export a singleton instance
const apiClient = new ApiClient();
export default apiClient;
