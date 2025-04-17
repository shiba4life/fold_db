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

# Connect to the node
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('localhost', 9000))

# Get schema details
schema_name = "user"
request = {
    "operation": "get_schema",
    "params": {
        "schema_name": schema_name
    }
}
print(f"Sending request to get schema '{schema_name}':")
print(json.dumps(request, indent=2))

response = send_request(sock, request)
print("Response received:")
print(json.dumps(response, indent=2))

# Close the connection
sock.close()
