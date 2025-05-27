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

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    print_warning "Node.js is not installed. Installing jsonwebtoken package globally..."
    npm install -g jsonwebtoken
fi

# Generate a simple test JWT
print_info "Generating test JWT token..."

# Create a temporary Node.js script to generate JWT
cat > /tmp/generate_jwt.js << 'EOF'
const jwt = require('jsonwebtoken');

// Mock JWT payload
const payload = {
  sub: "test-user-123",
  name: "Test User",
  email: "test@example.com",
  aud: process.env.AUDIENCE || "your-api-identifier",
  iss: process.env.ISSUER || "https://dev-12345.auth0.com/",
  exp: Math.floor(Date.now() / 1000) + (60 * 60 * 24), // 24 hours
  iat: Math.floor(Date.now() / 1000),
  scope: "read:users write:users"
};

// Secret key (for development only!)
const secret = process.env.JWT_SECRET || 'your-development-secret-key';

const token = jwt.sign(payload, secret, { algorithm: 'HS256' });

console.log('Test JWT Token:');
console.log(token);
console.log('');
console.log('Usage example:');
console.log(`export JWT_TOKEN="${token}"`);
console.log('curl -H "Authorization: Bearer $JWT_TOKEN" http://localhost:8080/query \\');
console.log('  -d \'{"query": "SELECT version()"}\'');
EOF

# Run the script
node /tmp/generate_jwt.js

# Clean up
rm /tmp/generate_jwt.js

print_warning "This token is for development only and uses a mock secret!"
print_warning "For production, use proper OIDC tokens from your identity provider."
