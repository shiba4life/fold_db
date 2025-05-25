#!/usr/bin/env python3
import requests
import json

def test_mutation(name, description):
    print(f"\n=== Testing {name} ===")
    try:
        # Get the mutation data
        response = requests.get(f'http://localhost:9001/api/samples/mutation/{name}')
        print(f"Get mutation status: {response.status_code}")
        
        if response.status_code == 200:
            mutation_data = response.json()
            print(f"Mutation data: {json.dumps(mutation_data, indent=2)}")
            
            # Execute the mutation
            exec_response = requests.post(
                'http://localhost:9001/api/mutation',
                headers={'Content-Type': 'application/json'},
                json=mutation_data
            )
            print(f"Execution status: {exec_response.status_code}")
            print(f"Execution result: {exec_response.text}")
            return exec_response.status_code == 200
        else:
            print(f"Failed to get mutation: {response.text}")
            return False
    except Exception as e:
        print(f"Error: {e}")
        return False

# First, create a product
success = test_mutation("CreateProduct", "Create a product first")

if success:
    print("\n" + "="*50)
    print("Product created successfully! Now testing update...")
    # Then update the product inventory
    test_mutation("UpdateProductInventory", "Update the product inventory")
else:
    print("Failed to create product, skipping update test")