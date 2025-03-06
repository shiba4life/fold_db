/**
 * Secure Sandbox Example for FoldDB
 * 
 * This example demonstrates how to run third-party code securely in a sandboxed
 * environment within DataFold Nodes, preventing data exfiltration and ensuring
 * data sovereignty.
 */

// Sandbox configuration
const sandboxConfig = {
    // Resource limits
    limits: {
        memory: '128MB',
        cpu: 0.5, // 50% of one CPU core
        execution_time: '5s',
        network: false, // No network access
    },
    
    // Allowed APIs
    allowed_apis: [
        'data.read',
        'data.transform',
        'crypto.hash',
    ],
    
    // Data access permissions
    permissions: {
        schemas: ['user-profile', 'analytics'],
        fields: {
            'user-profile': ['username', 'preferences'],
            'analytics': ['event_type', 'timestamp', 'anonymous_id'],
        },
        operations: ['read', 'aggregate'],
    },
};

/**
 * Example of a sentiment analysis function that runs in the sandbox
 * 
 * This function analyzes user messages for sentiment without exposing
 * the raw message content outside the sandbox.
 * 
 * @param {Object} context - Sandbox execution context
 * @param {Object} input - Input data for the function
 * @returns {Object} - Analysis results
 */
async function analyzeSentiment(context, input) {
    // Get the message to analyze
    const { message, userId } = input;
    
    // Simple sentiment analysis (in a real implementation, this would use a more sophisticated algorithm)
    const positiveWords = ['good', 'great', 'excellent', 'happy', 'love', 'like', 'best'];
    const negativeWords = ['bad', 'terrible', 'awful', 'sad', 'hate', 'dislike', 'worst'];
    
    const words = message.toLowerCase().split(/\W+/);
    
    let positiveCount = 0;
    let negativeCount = 0;
    
    for (const word of words) {
        if (positiveWords.includes(word)) {
            positiveCount++;
        } else if (negativeWords.includes(word)) {
            negativeCount++;
        }
    }
    
    const totalSentimentWords = positiveCount + negativeCount;
    let sentiment = 'neutral';
    let score = 0;
    
    if (totalSentimentWords > 0) {
        score = (positiveCount - negativeCount) / totalSentimentWords;
        
        if (score > 0.3) {
            sentiment = 'positive';
        } else if (score < -0.3) {
            sentiment = 'negative';
        }
    }
    
    // Store the analysis result (but not the original message)
    if (context.permissions.canWrite('analytics')) {
        await context.data.create({
            schema: 'analytics',
            data: {
                user_id: userId,
                event_type: 'sentiment_analysis',
                sentiment,
                score,
                timestamp: new Date().toISOString(),
            },
        });
    }
    
    // Return only the analysis result, not the original message
    return {
        sentiment,
        score,
        confidence: totalSentimentWords > 5 ? 'high' : 'low',
    };
}

/**
 * Example of a user preferences analyzer that runs in the sandbox
 * 
 * This function analyzes user preferences without exposing the full
 * user profile outside the sandbox.
 * 
 * @param {Object} context - Sandbox execution context
 * @param {Object} input - Input data for the function
 * @returns {Object} - Analysis results
 */
async function analyzeUserPreferences(context, input) {
    const { userId } = input;
    
    // Get user profile data
    const userProfile = await context.data.query({
        schema: 'user-profile',
        fields: ['username', 'preferences'],
        where: { id: userId },
    });
    
    if (!userProfile || !userProfile.preferences) {
        return { error: 'User not found or no preferences available' };
    }
    
    // Analyze preferences
    const preferences = userProfile.preferences;
    const categories = Object.keys(preferences);
    
    // Find top categories
    const topCategories = categories
        .map(category => ({ category, score: preferences[category] }))
        .sort((a, b) => b.score - a.score)
        .slice(0, 3);
    
    // Store the analysis result
    if (context.permissions.canWrite('analytics')) {
        await context.data.create({
            schema: 'analytics',
            data: {
                user_id: userId,
                event_type: 'preference_analysis',
                top_categories: topCategories.map(c => c.category),
                timestamp: new Date().toISOString(),
            },
        });
    }
    
    // Return only the analysis result, not the full user profile
    return {
        username: userProfile.username,
        top_preferences: topCategories,
        categories_count: categories.length,
    };
}

/**
 * Example of a data aggregation function that runs in the sandbox
 * 
 * This function aggregates user data without exposing individual
 * user records outside the sandbox.
 * 
 * @param {Object} context - Sandbox execution context
 * @param {Object} input - Input data for the function
 * @returns {Object} - Aggregated results
 */
async function aggregateUserData(context, input) {
    const { timeRange } = input;
    
    // Get analytics data
    const analyticsData = await context.data.query({
        schema: 'analytics',
        fields: ['event_type', 'timestamp', 'sentiment', 'score'],
        where: {
            timestamp: {
                gte: timeRange.start,
                lte: timeRange.end,
            },
        },
    });
    
    if (!analyticsData || !Array.isArray(analyticsData)) {
        return { error: 'No analytics data available' };
    }
    
    // Aggregate by event type
    const eventCounts = {};
    const sentimentScores = [];
    
    for (const event of analyticsData) {
        // Count events by type
        eventCounts[event.event_type] = (eventCounts[event.event_type] || 0) + 1;
        
        // Collect sentiment scores
        if (event.event_type === 'sentiment_analysis' && typeof event.score === 'number') {
            sentimentScores.push(event.score);
        }
    }
    
    // Calculate average sentiment score
    let averageSentiment = 0;
    if (sentimentScores.length > 0) {
        averageSentiment = sentimentScores.reduce((sum, score) => sum + score, 0) / sentimentScores.length;
    }
    
    // Return aggregated data only
    return {
        total_events: analyticsData.length,
        event_distribution: eventCounts,
        time_range: timeRange,
        sentiment_analysis: {
            count: sentimentScores.length,
            average_score: averageSentiment,
        },
    };
}

// Export the functions for use in the sandbox
module.exports = {
    config: sandboxConfig,
    functions: {
        analyzeSentiment,
        analyzeUserPreferences,
        aggregateUserData,
    },
};
