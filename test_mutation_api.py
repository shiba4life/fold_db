#!/usr/bin/env python3
import requests
import json

# Test the sample mutations API
try:
    response = requests.get('http://localhost:9001/api/samples/mutations')
    print(f"Status Code: {response.status_code}")
    print(f"Response: {response.text}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"Sample mutations: {data}")
        
        # Test running the UpdateProductInventory mutation
        if 'UpdateProductInventory' in data.get('data', []):
            print("\nTesting UpdateProductInventory mutation...")
            
            # Get the mutation data
            mutation_response = requests.get('http://localhost:9001/api/samples/mutation/UpdateProductInventory')
            print(f"Mutation data status: {mutation_response.status_code}")
            print(f"Mutation data: {mutation_response.text}")
            
            if mutation_response.status_code == 200:
                mutation_data = mutation_response.json()
                
                # Execute the mutation
                exec_response = requests.post(
                    'http://localhost:9001/api/mutation',
                    headers={'Content-Type': 'application/json'},
                    json=mutation_data
                )
                print(f"Execution status: {exec_response.status_code}")
                print(f"Execution result: {exec_response.text}")
        else:
            print("UpdateProductInventory not found in sample mutations")
    
except Exception as e:
    print(f"Error: {e}")