#!/usr/bin/env python3
import requests
import json

# Get list of schemas
try:
    response = requests.get('http://localhost:9001/api/schemas')
    print(f"Schemas list status: {response.status_code}")
    
    if response.status_code == 200:
        schemas = response.json()
        print(f"Available schemas: {json.dumps(schemas, indent=2)}")
        
        # Get ProductCatalog schema details
        if 'ProductCatalog' in [s['name'] for s in schemas]:
            schema_response = requests.get('http://localhost:9001/api/schema/ProductCatalog')
            print(f"\nProductCatalog schema status: {schema_response.status_code}")
            if schema_response.status_code == 200:
                schema_data = schema_response.json()
                print(f"ProductCatalog schema: {json.dumps(schema_data, indent=2)}")
        else:
            print("ProductCatalog schema not found")
    
except Exception as e:
    print(f"Error: {e}")