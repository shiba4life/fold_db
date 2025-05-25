#!/usr/bin/env python3

import socket
import json
import time

def send_request(sock, request):
    """Send a request and get response"""
    message = json.dumps(request) + '\n'
    sock.send(message.encode('utf-8'))
    
    response = sock.recv(4096).decode('utf-8').strip()
    return json.loads(response)

def main():
    print("Testing simple Range field retrieval...")
    
    # Connect to DataFold node
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', 9000))
    print("Connected to DataFold node on port 9000")
    
    try:
        # 1. Create schema (using the same format as the working test)
        schema_request = {
            "operation": "create_schema",
            "params": {
                "schema": {
                    "name": "SimpleTest",
                    "fields": {
                        "name": {
                            "field_type": "Single",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0}
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "min_payment": None,
                                "trust_distance_scaling": {"None": None}
                            },
                            "field_mappers": {}
                        },
                        "data": {
                            "field_type": "Range",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0}
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "min_payment": None,
                                "trust_distance_scaling": {"None": None}
                            },
                            "field_mappers": {}
                        }
                    },
                    "payment_config": {
                        "base_multiplier": 1.2,
                        "min_payment_threshold": 300
                    }
                }
            }
        }
        
        print("\n1. Creating SimpleTest schema...")
        response = send_request(sock, schema_request)
        print(f"Schema creation response: {json.dumps(response, indent=2)}")
        
        # 2. Create data
        mutation_request = {
            "operation": "mutation",
            "params": {
                "schema": "SimpleTest",
                "mutation_type": "create",
                "data": {
                    "name": "Test Item",
                    "data": {
                        "key1": "value1",
                        "key2": "value2",
                        "warehouse:north": "100"
                    }
                }
            }
        }
        
        print("\n2. Creating test data...")
        response = send_request(sock, mutation_request)
        print(f"Mutation response: {json.dumps(response, indent=2)}")
        
        # 3. Query just the Range field without filter
        query_request = {
            "operation": "query",
            "params": {
                "schema": "SimpleTest",
                "fields": ["data"]
            }
        }
        
        print("\n3. Querying Range field without filter...")
        response = send_request(sock, query_request)
        print(f"Query response: {json.dumps(response, indent=2)}")
        
        # 4. Query the Range field with filter
        filtered_query_request = {
            "operation": "query",
            "params": {
                "schema": "SimpleTest",
                "fields": ["data"],
                "filter": {
                    "field": "data",
                    "range_filter": {
                        "KeyPrefix": "warehouse:"
                    }
                }
            }
        }
        
        print("\n4. Querying Range field with filter...")
        response = send_request(sock, filtered_query_request)
        print(f"Filtered query response: {json.dumps(response, indent=2)}")
        
    finally:
        sock.close()
    
    print("\n✅ Simple Range field test completed!")

if __name__ == "__main__":
    main()