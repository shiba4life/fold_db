# DataFold Sample Data Design

This document outlines the design for sample data that will be included in the DataFold UI for one-click loading. These samples will provide users with ready-to-use examples of schemas, queries, and mutations.

## 1. Sample Schemas

### 1.1 User Profile Schema

```json
{
  "name": "UserProfile",
  "fields": {
    "username": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "email": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "full_name": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "bio": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "age": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "location": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 2 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

**Description**: A basic user profile schema with common fields like username, email, and bio. Includes various permission policies to demonstrate different access levels.

### 1.2 Product Catalog Schema

```json
{
  "name": "ProductCatalog",
  "fields": {
    "product_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "name": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "description": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "price": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "category": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "inventory_count": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

**Description**: A product catalog schema suitable for e-commerce applications. Includes fields for product details, pricing, and inventory management.

### 1.3 Blog Post Schema

```json
{
  "name": "BlogPost",
  "fields": {
    "title": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "content": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "author": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "publish_date": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "tags": {
      "field_type": "Collection",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "comments": {
      "field_type": "Collection",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 1 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

**Description**: A blog post schema with fields for content, metadata, and comments. Demonstrates the use of collection fields for tags and comments.

### 1.4 Social Media Post Schema

```json
{
  "name": "SocialMediaPost",
  "fields": {
    "user_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "content": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 2 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "media_url": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 2 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "likes": {
      "field_type": "Collection",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 1 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "comments": {
      "field_type": "Collection",
      "permission_policy": {
        "read_policy": { "Distance": 2 },
        "write_policy": { "Distance": 1 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    },
    "timestamp": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "NoRequirement": null },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null }
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 1.0,
    "min_payment_threshold": 0
  }
}
```

**Description**: A social media post schema with fields for content, media, likes, and comments. Demonstrates more complex permission policies for social content.

### 1.5 Financial Transaction Schema

```json
{
  "name": "FinancialTransaction",
  "fields": {
    "transaction_id": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": { "Linear": { "slope": 0.5, "intercept": 1.0, "min_factor": 1.0 } },
        "min_payment": 1000
      },
      "field_mappers": {}
    },
    "amount": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": { "Linear": { "slope": 0.5, "intercept": 1.0, "min_factor": 1.0 } },
        "min_payment": 1000
      },
      "field_mappers": {}
    },
    "sender": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": { "Linear": { "slope": 0.5, "intercept": 1.0, "min_factor": 1.0 } },
        "min_payment": 1000
      },
      "field_mappers": {}
    },
    "recipient": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 2.0,
        "trust_distance_scaling": { "Linear": { "slope": 0.5, "intercept": 1.0, "min_factor": 1.0 } },
        "min_payment": 1000
      },
      "field_mappers": {}
    },
    "timestamp": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null },
        "min_payment": null
      },
      "field_mappers": {}
    },
    "status": {
      "field_type": "Single",
      "permission_policy": {
        "read_policy": { "Distance": 1 },
        "write_policy": { "Distance": 0 }
      },
      "payment_config": {
        "base_multiplier": 1.0,
        "trust_distance_scaling": { "None": null },
        "min_payment": null
      },
      "field_mappers": {}
    }
  },
  "payment_config": {
    "base_multiplier": 2.0,
    "min_payment_threshold": 1000
  }
}
```

**Description**: A financial transaction schema with strict permission policies and payment configurations. Demonstrates more complex payment scaling for sensitive data.

## 2. Sample Queries

### 2.1 Basic User Query

```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username", "email", "full_name"],
  "filter": null
}
```

**Description**: A simple query to retrieve basic user information without any filtering.

### 2.2 Filtered User Query

```json
{
  "type": "query",
  "schema": "UserProfile",
  "fields": ["username", "email", "location"],
  "filter": {
    "field": "age",
    "operator": "gt",
    "value": 25
  }
}
```

**Description**: A query to retrieve user information filtered by age greater than 25.

### 2.3 Product Category Query

```json
{
  "type": "query",
  "schema": "ProductCatalog",
  "fields": ["product_id", "name", "price", "inventory_count"],
  "filter": {
    "field": "category",
    "operator": "eq",
    "value": "Electronics"
  }
}
```

**Description**: A query to retrieve products in the Electronics category.

### 2.4 Recent Blog Posts Query

```json
{
  "type": "query",
  "schema": "BlogPost",
  "fields": ["title", "author", "publish_date"],
  "filter": {
    "field": "publish_date",
    "operator": "gt",
    "value": "2023-01-01"
  }
}
```

**Description**: A query to retrieve blog posts published after January 1, 2023.

### 2.5 Popular Social Posts Query

```json
{
  "type": "query",
  "schema": "SocialMediaPost",
  "fields": ["user_id", "content", "likes", "timestamp"],
  "filter": {
    "operator": "and",
    "conditions": [
      {
        "field": "likes",
        "operator": "gt",
        "value": 100
      },
      {
        "field": "timestamp",
        "operator": "gt",
        "value": "2023-06-01"
      }
    ]
  }
}
```

**Description**: A query to retrieve popular social media posts with more than 100 likes posted after June 1, 2023.

### 2.6 Transaction History Query

```json
{
  "type": "query",
  "schema": "FinancialTransaction",
  "fields": ["transaction_id", "amount", "timestamp", "status"],
  "filter": {
    "operator": "or",
    "conditions": [
      {
        "field": "sender",
        "operator": "eq",
        "value": "user123"
      },
      {
        "field": "recipient",
        "operator": "eq",
        "value": "user123"
      }
    ]
  }
}
```

**Description**: A query to retrieve all transactions where a specific user is either the sender or recipient.

## 3. Sample Mutations

### 3.1 Create User Mutation

```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "mutation_type": "create",
  "data": {
    "username": "johndoe",
    "email": "john.doe@example.com",
    "full_name": "John Doe",
    "bio": "Software developer and tech enthusiast",
    "age": 30,
    "location": "San Francisco, CA"
  }
}
```

**Description**: A mutation to create a new user profile with complete information.

### 3.2 Update User Mutation

```json
{
  "type": "mutation",
  "schema": "UserProfile",
  "mutation_type": "update",
  "data": {
    "username": "johndoe",
    "bio": "Senior software engineer and blockchain enthusiast",
    "location": "New York, NY"
  }
}
```

**Description**: A mutation to update specific fields of an existing user profile.

### 3.3 Create Product Mutation

```json
{
  "type": "mutation",
  "schema": "ProductCatalog",
  "mutation_type": "create",
  "data": {
    "product_id": "PROD-12345",
    "name": "Wireless Headphones",
    "description": "High-quality wireless headphones with noise cancellation",
    "price": 149.99,
    "category": "Electronics",
    "inventory_count": 50
  }
}
```

**Description**: A mutation to create a new product in the catalog.

### 3.4 Update Product Inventory Mutation

```json
{
  "type": "mutation",
  "schema": "ProductCatalog",
  "mutation_type": "update",
  "data": {
    "product_id": "PROD-12345",
    "price": 129.99,
    "inventory_count": 45
  }
}
```

**Description**: A mutation to update the price and inventory count of an existing product.

### 3.5 Create Blog Post Mutation

```json
{
  "type": "mutation",
  "schema": "BlogPost",
  "mutation_type": "create",
  "data": {
    "title": "Getting Started with DataFold",
    "content": "DataFold is a powerful distributed database system...",
    "author": "johndoe",
    "publish_date": "2023-07-15",
    "tags": ["database", "tutorial", "datafold"]
  }
}
```

**Description**: A mutation to create a new blog post with tags.

### 3.6 Delete Blog Post Mutation

```json
{
  "type": "mutation",
  "schema": "BlogPost",
  "mutation_type": "delete",
  "data": {
    "title": "Getting Started with DataFold"
  }
}
```

**Description**: A mutation to delete a blog post by its title.

### 3.7 Create Social Media Post Mutation

```json
{
  "type": "mutation",
  "schema": "SocialMediaPost",
  "mutation_type": "create",
  "data": {
    "user_id": "user123",
    "content": "Just launched a new feature in our app!",
    "media_url": "https://example.com/images/feature.png",
    "timestamp": "2023-07-20T14:30:00Z"
  }
}
```

**Description**: A mutation to create a new social media post with media.

### 3.8 Add Like to Social Media Post Mutation

```json
{
  "type": "mutation",
  "schema": "SocialMediaPost",
  "mutation_type": "add_to_collection:likes",
  "data": {
    "user_id": "user123",
    "timestamp": "2023-07-20T14:30:00Z",
    "likes": "user456"
  }
}
```

**Description**: A mutation to add a like to an existing social media post, demonstrating collection modification.

### 3.9 Create Financial Transaction Mutation

```json
{
  "type": "mutation",
  "schema": "FinancialTransaction",
  "mutation_type": "create",
  "data": {
    "transaction_id": "TRX-789012",
    "amount": 1500.00,
    "sender": "user123",
    "recipient": "user456",
    "timestamp": "2023-07-21T10:15:00Z",
    "status": "pending"
  }
}
```

**Description**: A mutation to create a new financial transaction.

### 3.10 Update Transaction Status Mutation

```json
{
  "type": "mutation",
  "schema": "FinancialTransaction",
  "mutation_type": "update",
  "data": {
    "transaction_id": "TRX-789012",
    "status": "completed"
  }
}
```

**Description**: A mutation to update the status of an existing financial transaction.

## 4. Implementation Notes

These sample data files will be stored in the `fold_node/src/datafold_node/samples/data/` directory and loaded by the sample manager at startup. The UI will provide a way to browse and load these samples with a single click, making it easy for users to get started with DataFold.

Each sample will include:
- A descriptive name
- A brief description of its purpose and use case
- The actual JSON data
- A preview in the UI

The sample data is designed to cover a wide range of use cases and demonstrate various features of the DataFold system, including different field types, permission policies, payment configurations, and operation types.