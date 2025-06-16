# Configuration Traits Examples

Comprehensive examples demonstrating the DataFold configuration traits system in real-world scenarios.

## Overview

These examples showcase different aspects of the trait-based configuration system, from basic implementations to advanced cross-platform scenarios. Each example includes complete working code, configuration files, tests, and detailed explanations.

## Available Examples

### [Basic Configuration](basic-config.md)
**Difficulty**: Beginner  
**Topics**: [`BaseConfig`](../../../../src/config/traits/base.rs:68), [`ConfigLifecycle`](../../../../src/config/traits/base.rs:95), Environment Variables

A complete implementation of a web application configuration demonstrating:
- Basic trait implementation
- Multi-format configuration files (TOML, JSON, YAML)
- Environment variable overrides
- Comprehensive validation
- Event reporting and metrics
- Error handling patterns

**Key Learning Points**:
- How to implement the foundational [`BaseConfig`](../../../../src/config/traits/base.rs:68) trait
- Multi-format serialization support
- Environment variable integration
- Testing strategies for configuration code

### [Database Configuration](database-config.md)
**Difficulty**: Intermediate  
**Topics**: [`DatabaseConfig`](../../../../src/config/traits/database.rs:18), Domain-Specific Traits, Performance Tuning

A production-ready database configuration showcasing:
- Domain-specific trait implementation
- Connection pooling and management
- Backup and encryption configuration
- Performance optimization
- Connectivity testing
- Advanced validation patterns

**Key Learning Points**:
- Using domain-specific traits for specialized functionality
- Implementing complex validation logic
- Integrating with external systems (databases)
- Performance monitoring and optimization
- Security best practices

### [Cross-Platform Configuration](cross-platform-config.md)
**Difficulty**: Advanced  
**Topics**: [`CrossPlatformConfig`](../../../../src/config/traits/integration.rs:15), Platform Abstraction, Performance Optimization

A comprehensive cross-platform application configuration demonstrating:
- Platform-specific adaptations (Windows, macOS, Linux)
- Automatic directory detection and creation
- Platform-specific feature detection
- Performance optimization per platform
- Container and cgroup awareness
- Security and permission handling

**Key Learning Points**:
- Writing truly cross-platform configurations
- Platform-specific optimization strategies
- Container and cloud-native considerations
- Resource management and performance tuning

## Usage Patterns

### Simple Configuration
For basic application settings with minimal requirements:
```rust
// See: basic-config.md
use datafold::config::traits::{BaseConfig, ConfigLifecycle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfig {
    pub name: String,
    pub port: u16,
}

impl BaseConfig for SimpleConfig {
    // Implementation...
}
```

### Domain-Specific Configuration
For specialized configurations with domain expertise:
```rust
// See: database-config.md
use datafold::config::traits::DatabaseConfig;

#[derive(Debug, Clone)]
pub struct MyDatabaseConfig {
    // Specialized fields...
}

impl DatabaseConfig for MyDatabaseConfig {
    // Domain-specific implementation...
}
```

### Cross-Platform Configuration
For applications that need to run on multiple operating systems:
```rust
// See: cross-platform-config.md
use datafold::config::traits::CrossPlatformConfig;

#[derive(Debug, Clone)]
pub struct PlatformAwareConfig {
    // Platform-specific adaptations...
}

impl CrossPlatformConfig for PlatformAwareConfig {
    // Platform optimization implementation...
}
```

## Configuration File Examples

### TOML Configuration
```toml
# Basic application configuration
app_name = "MyApplication"
bind_address = "127.0.0.1"
port = 8080
debug_mode = false

[features]
enable_metrics = true
enable_tracing = false
```

### JSON Configuration
```json
{
  "app_name": "MyApplication",
  "bind_address": "127.0.0.1",
  "port": 8080,
  "debug_mode": false,
  "features": {
    "enable_metrics": true,
    "enable_tracing": false
  }
}
```

### YAML Configuration
```yaml
app_name: MyApplication
bind_address: 127.0.0.1
port: 8080
debug_mode: false
features:
  enable_metrics: true
  enable_tracing: false
```

## Testing Examples

### Unit Testing
```rust
#[tokio::test]
async fn test_config_loading() {
    let config = MyConfig::load(Path::new("test.toml")).await.unwrap();
    assert!(config.validate().is_ok());
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_config_lifecycle() {
    let mut config = MyConfig::default();
    let temp_path = temp_file_path();
    
    config.save(&temp_path).await.unwrap();
    config.reload(&temp_path).await.unwrap();
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_config_serialization_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9]*",
        port in 1024u16..65535
    ) {
        let config = MyConfig { name, port, ..Default::default() };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: MyConfig = toml::from_str(&serialized).unwrap();
        prop_assert_eq!(config, deserialized);
    }
}
```

## Performance Considerations

### Memory Efficiency
- Use appropriate data structures for configuration storage
- Implement lazy loading for large configurations
- Optimize trait object usage to minimize memory overhead

### I/O Optimization
- Prefer async operations for all file I/O
- Implement caching for frequently accessed configurations
- Use atomic operations for configuration updates

### Platform-Specific Optimizations
- Leverage platform-specific APIs when available
- Adjust thread pools and resource allocation per platform
- Use platform-appropriate file system features

## Best Practices

### Configuration Design
1. **Keep it Simple**: Start with basic traits and add complexity as needed
2. **Validate Early**: Implement comprehensive validation at load time
3. **Document Everything**: Provide clear documentation for all configuration options
4. **Use Types**: Leverage Rust's type system for configuration safety

### Error Handling
1. **Rich Context**: Provide detailed error messages with field-specific information
2. **Recovery Strategies**: Implement fallback mechanisms for common failures
3. **User-Friendly Messages**: Make error messages actionable for end users
4. **Logging Integration**: Integrate with your logging system for debugging

### Testing Strategy
1. **Test All Paths**: Test both success and failure scenarios
2. **Use Trait Helpers**: Leverage the built-in testing infrastructure
3. **Integration Tests**: Test the complete configuration lifecycle
4. **Performance Tests**: Validate configuration performance characteristics

### Security Considerations
1. **Sensitive Data**: Never log sensitive configuration values
2. **Validation**: Validate all input from configuration files
3. **Permissions**: Use appropriate file system permissions
4. **Environment Variables**: Use environment variables for sensitive overrides

## Getting Started

1. **Choose an Example**: Start with the example that best matches your use case
2. **Follow the Guide**: Work through the [Usage Guide](../usage-guide.md) for detailed implementation instructions
3. **Run the Code**: All examples include complete working code and tests
4. **Adapt and Extend**: Modify the examples to fit your specific requirements

## Additional Resources

- **[Usage Guide](../usage-guide.md)**: Comprehensive developer guide
- **[Migration Guide](../migration-guide.md)**: Step-by-step migration from legacy configurations
- **[Architecture Guide](../architecture.md)**: Technical architecture and design patterns
- **[API Reference](../../../api-reference.md)**: Complete API documentation

## Contributing

When adding new examples:

1. **Complete Implementation**: Provide full working code with tests
2. **Documentation**: Include detailed explanations and usage instructions
3. **Configuration Files**: Provide example configuration files in multiple formats
4. **Error Scenarios**: Show how to handle common error conditions
5. **Performance Notes**: Include performance considerations and optimizations

---

These examples provide a solid foundation for implementing production-ready configuration management using the DataFold traits system. Each example builds on the previous ones, demonstrating progressively more advanced concepts and patterns.