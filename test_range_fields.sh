#!/bin/bash

echo "Testing Range Field functionality..."

# Wait for server to be ready
sleep 2

echo "1. Creating ProductCatalog schema..."
curl -X POST http://localhost:9001/schema \
  -H "Content-Type: application/json" \
  -d @fold_node/src/datafold_node/examples/product_catalog_schema.json

echo -e "\n\n2. Creating a product with Range fields..."
curl -X POST http://localhost:9001/mutation \
  -H "Content-Type: application/json" \
  -d '{
    "type": "mutation",
    "schema": "ProductCatalog",
    "mutation_type": "create",
    "data": {
      "name": "Test Gaming Laptop",
      "category": "Electronics",
      "price": "1299.99",
      "inventory_by_location": {
        "warehouse:north": "25",
        "warehouse:south": "18",
        "store:downtown": "5"
      },
      "attributes": {
        "brand": "TechCorp",
        "model": "GX-2024",
        "cpu": "Intel i7-13700H"
      }
    }
  }'

echo -e "\n\n3. Querying inventory for warehouse locations..."
curl -X POST http://localhost:9001/query \
  -H "Content-Type: application/json" \
  -d '{
    "type": "query",
    "schema": "ProductCatalog",
    "fields": ["name", "inventory_by_location"],
    "filter": {
      "field": "inventory_by_location",
      "range_filter": {
        "KeyPrefix": "warehouse:"
      }
    }
  }'

echo -e "\n\n4. Querying product attributes..."
curl -X POST http://localhost:9001/query \
  -H "Content-Type: application/json" \
  -d '{
    "type": "query",
    "schema": "ProductCatalog",
    "fields": ["name", "attributes"],
    "filter": {
      "field": "attributes",
      "range_filter": {
        "Key": "brand"
      }
    }
  }'

echo -e "\n\nRange field test completed!"