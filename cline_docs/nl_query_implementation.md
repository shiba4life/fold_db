# Natural Language Query Interface Implementation Plan

This document outlines a concrete implementation plan for adding a Natural Language Query Interface to FoldDB, one of the AI integration opportunities identified in the ai_integration.md document.

## Overview

The Natural Language Query Interface will allow users to interact with FoldDB using natural language instead of having to construct schema-compliant queries manually. This will significantly improve usability and lower the barrier to entry for new users.

## Architecture

```
                  +-------------------+
                  |                   |
                  |  Web Interface    |
                  |                   |
                  +--------+----------+
                           |
                           v
+-------------+   +--------+----------+   +-----------------+
|             |   |                   |   |                 |
| Schema Info +-->+ NLQueryProcessor +-->+ SchemaInterpreter|
|             |   |                   |   |                 |
+-------------+   +--------+----------+   +-----------------+
                           |
                           v
                  +--------+----------+
                  |                   |
                  |   FoldDB Core     |
                  |                   |
                  +-------------------+
```

## Components

### 1. NLQueryProcessor

This new component will be responsible for:
- Parsing natural language input
- Extracting query intent and parameters
- Mapping to schema entities and fields
- Generating schema-compliant queries

#### Implementation Details

```rust
pub struct NLQueryProcessor {
    schema_manager: Arc<SchemaManager>,
    model_client: NLModelClient,
    query_cache: Cache<String, ProcessedQuery>,
}

impl NLQueryProcessor {
    pub fn new(schema_manager: Arc<SchemaManager>) -> Self {
        // Initialize with schema manager and model client
        // Set up caching for processed queries
    }
    
    pub async fn process_query(&self, natural_language: &str) -> Result<ProcessedQuery, NLQueryError> {
        // Check cache first
        if let Some(cached) = self.query_cache.get(natural_language) {
            return Ok(cached.clone());
        }
        
        // Extract intent and entities from natural language
        let (intent, entities) = self.model_client.extract_intent_and_entities(natural_language).await?;
        
        // Map to schema fields
        let field_mappings = self.map_to_schema_fields(&entities)?;
        
        // Generate query based on intent
        let query = match intent {
            QueryIntent::Read => self.generate_read_query(&field_mappings)?,
            QueryIntent::Create => self.generate_create_query(&field_mappings)?,
            QueryIntent::Update => self.generate_update_query(&field_mappings)?,
            QueryIntent::Delete => self.generate_delete_query(&field_mappings)?,
        };
        
        // Cache the result
        let processed = ProcessedQuery { 
            original: natural_language.to_string(),
            intent,
            field_mappings,
            generated_query: query.clone(),
        };
        
        self.query_cache.insert(natural_language.to_string(), processed.clone());
        
        Ok(processed)
    }
    
    fn map_to_schema_fields(&self, entities: &[Entity]) -> Result<FieldMappings, NLQueryError> {
        // Map extracted entities to schema fields
        // Use schema information to resolve ambiguities
    }
    
    fn generate_read_query(&self, mappings: &FieldMappings) -> Result<Query, NLQueryError> {
        // Generate a read query based on the field mappings
    }
    
    // Similar methods for other query types
}
```

### 2. NLModelClient

This component will handle communication with the NLP model:

```rust
pub struct NLModelClient {
    client: HttpClient,
    model_endpoint: String,
    schema_context: String,
}

impl NLModelClient {
    pub fn new(model_endpoint: &str, schema_manager: &SchemaManager) -> Self {
        // Initialize with model endpoint
        // Generate schema context from schema manager
    }
    
    pub async fn extract_intent_and_entities(&self, text: &str) -> Result<(QueryIntent, Vec<Entity>), NLModelError> {
        // Prepare request with schema context and user text
        // Send to model endpoint
        // Parse response into intent and entities
    }
    
    pub async fn validate_query(&self, query: &Query, original_text: &str) -> Result<ValidationResult, NLModelError> {
        // Validate that the generated query matches the user's intent
    }
    
    fn update_schema_context(&mut self, schema_manager: &SchemaManager) {
        // Update the schema context when schemas change
    }
}
```

### 3. Web Interface Integration

Add a new endpoint to the web server for natural language queries:

```rust
// In src/datafold_node/web_server/handlers/query.rs

pub async fn handle_nl_query(
    State(state): State<AppState>,
    Json(payload): Json<NLQueryRequest>,
) -> Result<Json<NLQueryResponse>, AppError> {
    // Get the NL query processor from state
    let nl_processor = state.nl_processor.clone();
    
    // Process the natural language query
    let processed = nl_processor.process_query(&payload.query).await?;
    
    // Execute the generated query
    let result = state.fold_db.execute_query(&processed.generated_query).await?;
    
    // Return both the result and the processed query information
    Ok(Json(NLQueryResponse {
        result,
        processed_query: processed,
    }))
}
```

## Integration with Existing Components

### SchemaManager Extensions

Add methods to support NL query processing:

```rust
impl SchemaManager {
    // New methods
    
    pub fn get_field_descriptions(&self) -> HashMap<String, String> {
        // Return field descriptions for NL processing
    }
    
    pub fn get_entity_synonyms(&self) -> HashMap<String, Vec<String>> {
        // Return entity synonyms for better NL understanding
    }
    
    pub fn register_nl_query_pattern(&self, pattern: &str, query_template: &str) {
        // Register common query patterns for better matching
    }
}
```

### FoldDB Core Integration

```rust
impl FoldDB {
    // New method
    
    pub async fn execute_nl_query(&self, nl_query: &str) -> Result<QueryResult, FoldDBError> {
        // Get the NL processor
        let nl_processor = self.nl_processor.as_ref()
            .ok_or(FoldDBError::ComponentNotInitialized("NLQueryProcessor"))?;
            
        // Process the query
        let processed = nl_processor.process_query(nl_query).await?;
        
        // Execute the generated query
        self.execute_query(&processed.generated_query).await
    }
}
```

## Model Training and Deployment

### Training Data Generation

1. Create a synthetic dataset of natural language queries paired with FoldDB queries
2. Extract real query patterns from system logs (anonymized)
3. Generate variations using templates and schema information

### Model Architecture

1. Use a fine-tuned language model specialized for database query understanding
2. Incorporate schema information as context during inference
3. Implement a feedback loop to improve from user corrections

### Deployment Options

1. **Local Deployment**: 
   - Embed a smaller model directly in the application
   - Pros: Privacy, no external dependencies
   - Cons: Limited capabilities, higher resource usage

2. **API Service**:
   - Deploy as a separate service with a REST API
   - Pros: More powerful models, centralized updates
   - Cons: Network dependency, potential privacy concerns

3. **Hybrid Approach**:
   - Use local model for common queries
   - Fall back to API for complex cases
   - Pros: Balance of capabilities and reliability
   - Cons: More complex implementation

## User Experience Considerations

### Query Suggestions

As users type, provide suggestions based on:
- Schema entities and fields
- Common query patterns
- User's query history

### Explanations

For each generated query:
- Show a natural language explanation of what the query will do
- Highlight which parts of the input mapped to which schema elements
- Provide confidence scores for ambiguous mappings

### Feedback Loop

Allow users to:
- Correct misinterpreted queries
- Save successful queries as templates
- Rate query accuracy to improve the system

## Implementation Phases

### Phase 1: Foundation (1-2 months)

1. Implement basic NLQueryProcessor structure
2. Create integration points with SchemaManager
3. Set up model client interface
4. Develop simple rule-based query generation for common patterns

### Phase 2: Core Functionality (2-3 months)

1. Integrate with ML model for intent and entity extraction
2. Implement query generation for all CRUD operations
3. Add basic web interface integration
4. Create initial training dataset

### Phase 3: Refinement (2-3 months)

1. Implement query suggestions and explanations
2. Add feedback collection mechanisms
3. Improve model with user feedback
4. Optimize performance and caching

### Phase 4: Advanced Features (3-4 months)

1. Add support for complex queries (joins, aggregations)
2. Implement query validation and safety checks
3. Add conversational context (follow-up queries)
4. Integrate with permission system for context-aware queries

## Testing Strategy

1. **Unit Tests**:
   - Test query parsing and generation components
   - Validate schema mapping logic
   - Test caching mechanisms

2. **Integration Tests**:
   - Verify end-to-end query processing
   - Test integration with SchemaManager
   - Validate web interface integration

3. **Model Evaluation**:
   - Measure accuracy on test dataset
   - Evaluate performance on edge cases
   - Test with schema variations

4. **User Testing**:
   - Collect feedback on query understanding
   - Measure success rate of generated queries
   - Identify common failure patterns

## Performance Considerations

1. **Caching Strategy**:
   - Cache processed queries by text
   - Cache schema mappings
   - Implement LRU eviction policy

2. **Batch Processing**:
   - Group similar queries for efficient processing
   - Pre-compute common query patterns

3. **Model Optimization**:
   - Use quantized models for faster inference
   - Consider model distillation for production

## Security Considerations

1. **Input Validation**:
   - Sanitize all natural language input
   - Validate generated queries against schema

2. **Permission Integration**:
   - Apply same permission checks as manual queries
   - Consider intent when evaluating permissions

3. **Rate Limiting**:
   - Implement rate limiting for NL query processing
   - Monitor for abuse patterns

## Metrics and Monitoring

1. **Performance Metrics**:
   - Query processing time
   - Model inference time
   - Cache hit rate

2. **Quality Metrics**:
   - Query understanding accuracy
   - User correction rate
   - Query success rate

3. **Usage Metrics**:
   - Most common query types
   - Schema elements frequently accessed
   - User engagement patterns
