#!/usr/bin/env python3

import socket
import json

def send_request(sock, request):
    """Send a JSON request and receive response using length-prefixed protocol"""
    import struct
    
    request_str = json.dumps(request)
    request_bytes = request_str.encode('utf-8')
    
    # Send length prefix (4 bytes, big-endian)
    length = len(request_bytes)
    sock.send(struct.pack('>I', length))
    
    # Send the actual request
    sock.send(request_bytes)
    
    # Read response length prefix
    length_bytes = sock.recv(4)
    if len(length_bytes) != 4:
        raise Exception("Failed to read response length")
    
    response_length = struct.unpack('>I', length_bytes)[0]
    
    # Read the actual response
    response_data = b''
    while len(response_data) < response_length:
        chunk = sock.recv(response_length - len(response_data))
        if not chunk:
            raise Exception("Connection closed while reading response")
        response_data += chunk
    
    return json.loads(response_data.decode('utf-8'))

def test_range_debug():
    """Test Range field data storage and retrieval"""
    try:
        print("Debugging Range Field data storage...")
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(('localhost', 9000))
        print("Connected to DataFold node on port 9000")
        
        # 1. Create schema
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
        
        # 2. Create a product
        print("\n2. Creating a product with Range fields...")
        mutation_request = {
            "operation": "mutation",
            "params": {
                "schema": "ProductCatalog",
                "data": {
                    "name": "Test Gaming Laptop",
                    "inventory_by_location": {
                        "warehouse:north": "25",
                        "warehouse:south": "18",
                        "store:downtown": "5",
                        "store:mall": "8"
                    }
                },
                "mutation_type": "create"
            }
        }
        
        response = send_request(sock, mutation_request)
        print("Mutation response:", json.dumps(response, indent=2))
        
        # 3. Query without filter to see stored data
        print("\n3. Querying all data (no filter)...")
        query_request = {
            "operation": "query",
            "params": {
                "schema": "ProductCatalog",
                "fields": ["name", "inventory_by_location"]
            }
        }
        
        response = send_request(sock, query_request)
        print("All data query response:", json.dumps(response, indent=2))
        
        # 4. Query with filter
        print("\n4. Querying with warehouse filter...")
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
        print("Filtered query response:", json.dumps(response, indent=2))
        
        print("\n✅ Range field debug completed!")
        
    except Exception as e:
        print(f"❌ Error during debugging: {e}")
    finally:
        sock.close()

if __name__ == "__main__":
    test_range_debug()