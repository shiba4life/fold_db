use std::collections::HashMap;
use serde_json::Value;

use crate::error::{AppSdkError, AppSdkResult};
use crate::schema::{
    Schema, SchemaField, FieldType, PermissionsPolicy, 
    TrustDistance, FieldPaymentConfig, SchemaPaymentConfig, TrustDistanceScaling
};

/// Builder for creating and configuring Schema instances.
/// 
/// SchemaBuilder provides a fluent API for constructing schemas with:
/// - Field definitions with types and constraints
/// - Permission policies for access control
/// - Payment configurations for data access
/// - Field mappings for schema transformation
/// 
/// This builder simplifies schema creation by:
/// - Providing sensible defaults
/// - Offering a chainable API for configuration
/// - Handling validation during construction
/// - Supporting incremental field addition
/// 
/// # Examples
/// 
/// ```
/// use datafold_sdk::schema_builder::SchemaBuilder;
/// 
/// // Create a simple user profile schema
/// let schema = SchemaBuilder::new("user_profile")
///     .add_field("username", |field| {
///         field.field_type(FieldType::Single)
///             .required(true)
///             .description("User's unique username")
///     })
///     .add_field("email", |field| {
///         field.field_type(FieldType::Single)
///             .required(true)
///             .description("User's email address")
///     })
///     .add_field("posts", |field| {
///         field.field_type(FieldType::Collection)
///             .required(false)
///             .description("User's posts")
///     })
///     .build();
/// ```
#[derive(Debug)]
pub struct SchemaBuilder {
    /// Name of the schema being built
    name: String,
    
    /// Fields being added to the schema
    fields: HashMap<String, SchemaField>,
    
    /// Payment configuration for the schema
    payment_config: SchemaPaymentConfig,
    
    /// Field definitions with additional metadata
    field_definitions: HashMap<String, FieldDefinition>,
}

/// Definition of a field with metadata for schema creation
#[derive(Debug, Clone)]
struct FieldDefinition {
    /// Field type (single or collection)
    field_type: FieldType,
    
    /// Whether the field is required
    required: bool,
    
    /// Description of the field
    description: Option<String>,
    
    /// JSON Schema validation rules
    validation: Option<Value>,
    
    /// Permission policy for the field
    permission_policy: PermissionsPolicy,
    
    /// Payment configuration for the field
    payment_config: FieldPaymentConfig,
    
    /// Field mappings for schema transformation
    field_mappers: HashMap<String, String>,
}

impl Default for FieldDefinition {
    fn default() -> Self {
        Self {
            field_type: FieldType::Single,
            required: false,
            description: None,
            validation: None,
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(0),
                write_policy: TrustDistance::Distance(0),
            },
            payment_config: FieldPaymentConfig {
                base_multiplier: 1.0,
                trust_scaling: TrustDistanceScaling::None,
                payment_threshold: None,
            },
            field_mappers: HashMap::new(),
        }
    }
}

/// Builder for configuring a single field within a schema
#[derive(Debug)]
pub struct FieldBuilder<'a> {
    /// Reference to the parent schema builder
    schema_builder: &'a mut SchemaBuilder,
    
    /// Name of the field being built
    field_name: String,
    
    /// Definition of the field
    definition: FieldDefinition,
}

impl SchemaBuilder {
    /// Creates a new SchemaBuilder with the specified name.
    /// 
    /// Initializes a builder with:
    /// - Empty field collection
    /// - Default payment configuration
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name of the schema to build
    /// 
    /// # Returns
    /// 
    /// A new SchemaBuilder instance
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: HashMap::new(),
            payment_config: SchemaPaymentConfig {
                base_multiplier: 1.0,
                trust_scaling: TrustDistanceScaling::None,
                payment_threshold: None,
            },
            field_definitions: HashMap::new(),
        }
    }

    /// Adds a field to the schema with the specified configuration.
    /// 
    /// This method allows adding a field with a custom configuration
    /// using a closure that receives a FieldBuilder.
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - Name of the field to add
    /// * `config_fn` - Closure that configures the field
    /// 
    /// # Returns
    /// 
    /// The SchemaBuilder instance for method chaining
    pub fn add_field<F>(mut self, field_name: &str, config_fn: F) -> Self
    where
        F: FnOnce(FieldBuilder) -> FieldBuilder,
    {
        let definition = {
            let field_builder = FieldBuilder {
                schema_builder: &mut self,
                field_name: field_name.to_string(),
                definition: FieldDefinition::default(),
            };
            
            let configured_builder = config_fn(field_builder);
            configured_builder.definition
        };
        
        self.field_definitions.insert(field_name.to_string(), definition);
        self
    }

    /// Sets the payment configuration for the schema.
    /// 
    /// # Arguments
    /// 
    /// * `base_multiplier` - Base multiplier for all operations
    /// * `trust_scaling` - Trust distance scaling factor
    /// * `payment_threshold` - Optional payment threshold
    /// 
    /// # Returns
    /// 
    /// The SchemaBuilder instance for method chaining
    pub fn payment_config(
        mut self,
        base_multiplier: f64,
        trust_scaling: TrustDistanceScaling,
        payment_threshold: Option<f64>,
    ) -> Self {
        self.payment_config = SchemaPaymentConfig {
            base_multiplier,
            trust_scaling,
            payment_threshold,
        };
        
        self
    }

    /// Builds the Schema instance with the configured settings.
    /// 
    /// This method:
    /// - Creates SchemaField instances from field definitions
    /// - Applies permission policies and payment configurations
    /// - Constructs the final Schema with all settings
    /// 
    /// # Returns
    /// 
    /// A Result containing the built Schema or an error
    pub fn build(self) -> AppSdkResult<Schema> {
        let mut fields = HashMap::new();
        
        // Convert field definitions to SchemaField instances
        for (field_name, definition) in self.field_definitions {
            let field = SchemaField::new(
                definition.permission_policy,
                definition.payment_config,
                definition.field_mappers,
                Some(definition.field_type),
            );
            
            fields.insert(field_name, field);
        }
        
        // Create the schema
        let schema = Schema {
            name: self.name,
            fields,
            payment_config: self.payment_config,
        };
        
        Ok(schema)
    }
}

impl<'a> FieldBuilder<'a> {
    /// Sets the field type (single or collection).
    /// 
    /// # Arguments
    /// 
    /// * `field_type` - Type of the field
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn field_type(mut self, field_type: FieldType) -> Self {
        self.definition.field_type = field_type;
        self
    }

    /// Sets whether the field is required.
    /// 
    /// # Arguments
    /// 
    /// * `required` - Whether the field is required
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn required(mut self, required: bool) -> Self {
        self.definition.required = required;
        self
    }

    /// Sets the description of the field.
    /// 
    /// # Arguments
    /// 
    /// * `description` - Description of the field
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn description(mut self, description: &str) -> Self {
        self.definition.description = Some(description.to_string());
        self
    }

    /// Sets the validation rules for the field.
    /// 
    /// # Arguments
    /// 
    /// * `validation` - JSON Schema validation rules
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn validation(mut self, validation: Value) -> Self {
        self.definition.validation = Some(validation);
        self
    }

    /// Sets the permission policy for the field.
    /// 
    /// # Arguments
    /// 
    /// * `read_policy` - Policy for read access
    /// * `write_policy` - Policy for write access
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn permissions(
        mut self,
        read_policy: TrustDistance,
        write_policy: TrustDistance,
    ) -> Self {
        self.definition.permission_policy = PermissionsPolicy {
            read_policy,
            write_policy,
        };
        self
    }

    /// Sets the payment configuration for the field.
    /// 
    /// # Arguments
    /// 
    /// * `base_multiplier` - Base multiplier for field operations
    /// * `trust_scaling` - Trust distance scaling factor
    /// * `payment_threshold` - Optional payment threshold
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn payment_config(
        mut self,
        base_multiplier: f64,
        trust_scaling: TrustDistanceScaling,
        payment_threshold: Option<f64>,
    ) -> Self {
        self.definition.payment_config = FieldPaymentConfig {
            base_multiplier,
            trust_scaling,
            payment_threshold,
        };
        self
    }

    /// Adds a field mapping for schema transformation.
    /// 
    /// # Arguments
    /// 
    /// * `source_schema` - Name of the source schema
    /// * `source_field` - Name of the source field
    /// 
    /// # Returns
    /// 
    /// The FieldBuilder instance for method chaining
    pub fn add_mapping(mut self, source_schema: &str, source_field: &str) -> Self {
        self.definition.field_mappers.insert(
            source_schema.to_string(),
            source_field.to_string(),
        );
        self
    }
}
