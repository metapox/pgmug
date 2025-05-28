#!/bin/bash

# Generate a test JWT token for development purposes
# This creates a mock JWT that can be used for testing the proxy
# Note: This is for development only - do not use in production!

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if Python3 is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python3 is not installed."
    exit 1
fi

# Generate a simple test JWT
print_info "Generating test JWT token..."

# Create a temporary Python script to generate JWT
cat > /tmp/generate_jwt.py << 'EOF'
import json
import base64
import hmac
import hashlib
import time
import os

# Mock JWT payload
payload = {
    "sub": "test-user-123",
    "name": "Test User",
    "email": "test@example.com",
    "aud": os.getenv("AUDIENCE", "your-api-identifier"),
    "iss": os.getenv("ISSUER", "https://dev-12345.auth0.com/"),
    "exp": int(time.time()) + (60 * 60 * 24),  # 24 hours
    "iat": int(time.time()),
    "scope": "read:users write:users"
}

# JWT header
header = {
    "alg": "HS256",
    "typ": "JWT"
}

# Secret key (for development only!)
secret = os.getenv("JWT_SECRET", "your-development-secret-key")

def base64url_encode(data):
    """Base64URL encode"""
    if isinstance(data, str):
        data = data.encode('utf-8')
    elif isinstance(data, dict):
        data = json.dumps(data, separators=(',', ':')).encode('utf-8')
    
    encoded = base64.urlsafe_b64encode(data)
    return encoded.rstrip(b'=').decode('utf-8')

def generate_signature(message, secret):
    """Generate HMAC-SHA256 signature"""
    return base64url_encode(hmac.new(
        secret.encode('utf-8'), 
        message.encode('utf-8'), 
        hashlib.sha256
    ).digest())

# Encode header and payload
header_encoded = base64url_encode(header)
payload_encoded = base64url_encode(payload)

# Create message to sign
message = f"{header_encoded}.{payload_encoded}"

# Generate signature
signature = generate_signature(message, secret)

# Create final JWT
token = f"{message}.{signature}"

print("Test JWT Token:")
print(token)
print("")
print("Usage example:")
print(f'export JWT_TOKEN="{token}"')
print('curl -H "Authorization: Bearer $JWT_TOKEN" -H "Content-Type: application/json" http://localhost:8080/query \\')
print('  -d \'{"sql": "SELECT version()}\'')
EOF

# Run the script
python3 /tmp/generate_jwt.py

# Clean up
rm /tmp/generate_jwt.py

print_warning "This token is for development only and uses a mock secret!"
print_warning "For production, use proper OIDC tokens from your identity provider."
