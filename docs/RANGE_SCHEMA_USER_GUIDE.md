# Range Schema User Guide

## Overview

Range schemas provide efficient querying and data organization by using a designated range key. This guide explains how to use range schemas through the DataFold web interface.

## What is a Range Schema?

A range schema is a special type of schema where:
- All fields are of type "Range"
- A designated field serves as the range_key
- Data is organized and indexed by the range key for efficient querying

### Example Range Schema Structure
```json
{
  "name": "UserScores",
  "range_key": "user_id",
  "fields": {
    "user_id": {
      "field_type": "Range",
      "writable": true
    },
    "game_scores": {
      "field_type": "Range", 
      "writable": true
    }
  }
}
```

## Using Range Schemas in the Web Interface

### 1. Loading Range Schemas

1. Navigate to the **Schemas** tab
2. Expand the **Available Schemas** section
3. Look for schemas with the purple "Range Schema" badge
4. Click **Approve** to load the schema

Range schemas will display:
- Purple "Range Schema" badge
- Range key information in the expanded view
- Special range schema information panel

### 2. Querying Range Schemas

#### Basic Range Queries

1. Go to the **Query** tab
2. Select your range schema from the dropdown
3. Select the fields you want to query
4. Use the simple range filter for basic filtering:
   ```
   Filter by user_id value: user123
   ```

#### Advanced Range Queries

Enable **Advanced Range Query Mode** for powerful filtering options:

##### Key Filter (Exact Match)
```
Key: user123
```
Matches exactly the key "user123"

##### Key Prefix Filter
```
Key Prefix: user:
```
Matches all keys starting with "user:" (e.g., "user:123", "user:456")

##### Key Range Filter
```
Start key: user100
End key: user200
```
Matches keys between "user100" (inclusive) and "user200" (exclusive)

##### Multiple Keys Filter
```
Keys: user123, user456, user789
```
Matches any of the specified keys (comma-separated)

##### Value Filter
```
Value: {"score": 100}
```
Matches records with the specified value

##### Pattern Filter
```
Key Pattern: user.*test
```
Matches keys using pattern matching

### 3. Range Schema Mutations

#### Create/Update Operations

1. Go to the **Mutation** tab
2. Select your range schema
3. Choose operation type (Create/Update)
4. **Required**: Enter the range key value
   ```
   user_id (Range Key): user123 [Required]
   ```
5. Fill in other field data
6. Click **Execute Mutation**

#### Delete Operations

1. Select **Delete** operation type
2. Range key is optional for Delete operations
3. If provided, targets specific records for deletion
4. Click **Execute Mutation**

## Range Query Examples

### Example 1: Gaming Leaderboard Query

**Scenario**: Query user scores for users in a specific range

```yaml
Schema: UserScores
Range Key: user_id
Query Type: Advanced Range Query
Filter: Key Range
  Start: user_1000
  End: user_2000
Fields: [game_scores, achievements]
```

**Result**: Returns game scores and achievements for users 1000-1999

### Example 2: User Activity Query

**Scenario**: Get all activity for a specific user

```yaml
Schema: UserActivity  
Range Key: user_id
Query Type: Basic Range Query
Filter: user_12345
Fields: [login_times, page_views, actions]
```

**Result**: Returns all activity data for user_12345

### Example 3: Prefix-based Organization Query

**Scenario**: Query all admin users

```yaml
Schema: UserProfiles
Range Key: user_id  
Query Type: Advanced Range Query
Filter: Key Prefix: admin_
Fields: [profile_data, permissions]
```

**Result**: Returns data for all users with IDs starting with "admin_"

## Range Mutation Examples

### Example 1: Creating User Score Record

```yaml
Schema: UserScores
Operation: Create
Range Key: user_12345
Data:
  game_scores: {"level1": 850, "level2": 920}
  achievements: ["first_win", "perfect_score"]
```

### Example 2: Updating User Statistics

```yaml
Schema: UserStats
Operation: Update  
Range Key: player_999
Data:
  player_statistics: {"wins": 45, "losses": 12}
  ranking_data: {"current_rank": 15}
```

### Example 3: Targeted Deletion

```yaml
Schema: UserSessions
Operation: Delete
Range Key: user_456  # Optional - targets specific user
```

## Validation Rules

### Range Key Validation

#### For Create/Update Operations
- Range key is **required**
- Cannot be empty or null
- Must be a valid string value

#### For Delete Operations  
- Range key is **optional**
- If provided, targets specific records
- If omitted, broader deletion scope

### Query Parameter Validation

#### Key Range Validation
- Start key must be less than end key
- Both start and end values required
- Proper string comparison used

#### Multiple Keys Validation
- Must be a non-empty array
- All keys must be valid strings
- Comma-separated input converted to array

## Error Handling

### Common Error Messages

#### Range Key Errors
```
Error: Range key is required for range schema mutations
Solution: Provide a valid range key value
```

#### Query Validation Errors
```
Error: KeyRange requires both start and end values
Solution: Provide both start and end values for range queries
```

```
Error: KeyRange start must be less than end  
Solution: Ensure start value comes before end value
```

#### Schema Loading Errors
```
Error: Failed to approve schema: Invalid field
Solution: Check schema definition for proper field types
```

### Troubleshooting Steps

#### 1. Schema Won't Load
- Check if all fields are Range type
- Verify range_key field exists
- Ensure schema JSON is valid
- Check server logs for detailed errors

#### 2. Query Returns No Results
- Verify range key values exist in data
- Check filter parameters are correct
- Ensure schema has data
- Try broader query parameters

#### 3. Mutation Fails
- Confirm range key is provided for Create/Update
- Check field data format
- Verify schema permissions
- Review validation error messages

#### 4. UI Not Showing Range Features
- Refresh the page
- Verify schema is properly loaded
- Check browser console for errors
- Ensure schema has range_key defined

## Best Practices

### Range Key Design
- Use meaningful, sortable keys
- Consider query patterns when designing keys
- Use consistent key formats
- Plan for scalability

### Query Optimization
- Use specific filters when possible
- Prefer range queries over pattern matching
- Limit result sets appropriately
- Use efficient key structures

### Data Organization
- Group related data by range key
- Use hierarchical key structures when beneficial
- Consider data access patterns
- Plan for future query needs

## Advanced Features

### Transform Integration
Range schemas support transforms on range fields:
```json
{
  "field_type": "Range",
  "transform": {
    "inputs": ["game_scores"],
    "logic": "calculate_total_score(game_scores)",
    "output": "UserScores.total_score"
  }
}
```

### Permission Policies
Range fields support granular permissions:
```json
{
  "permission_policy": {
    "read_policy": {"Distance": 0},
    "write_policy": {"Distance": 2}
  }
}
```

### Payment Configuration
Range fields can have payment requirements:
```json
{
  "payment_config": {
    "base_multiplier": 1.5,
    "min_payment": 10
  }
}
```

## API Integration

### REST API Endpoints

#### Range Query
```http
POST /api/query
Content-Type: application/json

{
  "type": "query",
  "schema": "UserScores", 
  "fields": ["game_scores"],
  "range_filter": {
    "KeyRange": {
      "start": "user100",
      "end": "user200"
    }
  }
}
```

#### Range Mutation
```http
POST /api/mutation
Content-Type: application/json

{
  "type": "mutation",
  "schema": "UserScores",
  "mutation_type": "create", 
  "data": {
    "range_key": "user123",
    "game_scores": {"level1": 850}
  }
}
```

## Support and Resources

### Getting Help
- Check this user guide for common scenarios
- Review error messages carefully
- Check server logs for detailed information
- Consult the troubleshooting section

### Additional Documentation
- [Range Schema Testing Report](./RANGE_SCHEMA_TESTING_REPORT.md)
- [Schema Input Guide](./SCHEMA_INPUT_GUIDE.md)
- [API Documentation](../README.md)

### Sample Schemas
Range schema examples are available in:
- `fold_node/available_schemas/UserScores.json`
- Additional samples in the `/available_schemas` directory