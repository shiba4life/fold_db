#!/usr/bin/env python3

import socket
import json

def send_request(sock, request):
    """Send a request and get response"""
    message = json.dumps(request) + '\n'
    sock.send(message.encode('utf-8'))
    
    response = sock.recv(4096).decode('utf-8').strip()
    return json.loads(response)

def main():
    print("Testing basic Range field retrieval...")
    
    # Connect to DataFold node
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', 9000))
    print("Connected to DataFold node on port 9000")
    
    try:
        # Query just the Range field without any filter
        query_request = {
            "operation": "query",
            "params": {
                "schema": "ProductCatalog",
                "fields": ["inventory_by_location"]
            }
        }
        
        print("\nQuerying Range field without filter...")
        response = send_request(sock, query_request)
        print(f"Query response: {json.dumps(response, indent=2)}")
        
    finally:
        sock.close()
    
    print("\n✅ Basic Range field test completed!")

if __name__ == "__main__":
    main()