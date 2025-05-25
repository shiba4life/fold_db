#!/usr/bin/env python3

import socket
import json
import struct

def send_request(sock, request):
    # Serialize the request to JSON
    request_json = json.dumps(request).encode('utf-8')
    
    # Send the length prefix
    sock.sendall(struct.pack('!I', len(request_json)))
    
    # Send the JSON payload
    sock.sendall(request_json)
    
    # Receive the response length
    response_len_bytes = sock.recv(4)
    response_len = struct.unpack('!I', response_len_bytes)[0]
    
    # Receive the response
    response_json = sock.recv(response_len)
    
    # Deserialize the response
    return json.loads(response_json.decode('utf-8'))

def test_range_fields():
    print("Testing Range Field functionality...")
    
    # Connect to the node
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.connect(('localhost', 9000))
        print("Connected to DataFold node on port 9000")
        
        # 1. Create ProductCatalog schema
        print("\n1. Creating ProductCatalog schema...")
        schema_request = {
            "operation": "create_schema",
            "params": {
                "schema": {
                    "name": "ProductCatalog",
                    "fields": {
                        "name": {
                            "field_type": "Single",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0},
                                "explicit_read_policy": None,
                                "explicit_write_policy": None
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "trust_distance_scaling": {"None": None},
                                "min_payment": None
                            },
                            "field_mappers": {}
                        },
                        "category": {
                            "field_type": "Single",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0},
                                "explicit_read_policy": None,
                                "explicit_write_policy": None
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "trust_distance_scaling": {"None": None},
                                "min_payment": None
                            },
                            "field_mappers": {}
                        },
                        "inventory_by_location": {
                            "field_type": "Range",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0},
                                "explicit_read_policy": None,
                                "explicit_write_policy": None
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "trust_distance_scaling": {"None": None},
                                "min_payment": None
                            },
                            "field_mappers": {}
                        },
                        "attributes": {
                            "field_type": "Range",
                            "permission_policy": {
                                "read_policy": {"NoRequirement": None},
                                "write_policy": {"Distance": 0},
                                "explicit_read_policy": None,
                                "explicit_write_policy": None
                            },
                            "payment_config": {
                                "base_multiplier": 1.0,
                                "trust_distance_scaling": {"None": None},
                                "min_payment": None
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
        
        response = send_request(sock, schema_request)
        print("Schema creation response:", json.dumps(response, indent=2))
        
        # 2. Create a product with Range fields
        print("\n2. Creating a product with Range fields...")
        mutation_request = {
            "operation": "mutation",
            "params": {
                "schema": "ProductCatalog",
                "data": {
                    "name": "Test Gaming Laptop",
                    "category": "Electronics",
                    "inventory_by_location": {
                        "warehouse:north": "25",
                        "warehouse:south": "18",
                        "store:downtown": "5",
                        "store:mall": "8"
                    },
                    "attributes": {
                        "brand": "TechCorp",
                        "model": "GX-2024",
                        "cpu": "Intel i7-13700H",
                        "gpu": "RTX 4060",
                        "warranty": "2 years"
                    }
                },
                "mutation_type": "create"
            }
        }
        
        response = send_request(sock, mutation_request)
        print("Mutation response:", json.dumps(response, indent=2))
        
        # 3. Query inventory for warehouse locations
        print("\n3. Querying inventory for warehouse locations...")
        query_request = {
            "operation": "query",
            "params": {
                "schema": "ProductCatalog",
                "fields": ["name", "inventory_by_location"],
                "filter": {
                    "field": "inventory_by_location",
                    "range_filter": {"KeyPrefix": "warehouse:"}
                }
            }
        }
        
        response = send_request(sock, query_request)
        print("Warehouse inventory query response:", json.dumps(response, indent=2))
        
        # 4. Query product attributes
        print("\n4. Querying product brand attribute...")
        query_request = {
            "operation": "query",
            "params": {
                "schema": "ProductCatalog",
                "fields": ["name", "attributes"],
                "filter": {
                    "field": "attributes",
                    "range_filter": {"Key": "brand"}
                }
            }
        }
        
        response = send_request(sock, query_request)
        print("Brand attribute query response:", json.dumps(response, indent=2))
        
        # 5. Query using pattern matching
        print("\n5. Querying using pattern matching for store locations...")
        query_request = {
            "operation": "query",
            "params": {
                "schema": "ProductCatalog",
                "fields": ["name", "inventory_by_location"],
                "filter": {
                    "field": "inventory_by_location",
                    "range_filter": {"KeyPattern": "store:*"}
                }
            }
        }
        
        response = send_request(sock, query_request)
        print("Store pattern query response:", json.dumps(response, indent=2))
        
        print("\n✅ Range field test completed successfully!")
        
    except Exception as e:
        print(f"❌ Error during testing: {e}")
    finally:
        sock.close()

if __name__ == "__main__":
    test_range_fields()