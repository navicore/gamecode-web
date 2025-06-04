#!/bin/bash
# Test script for GameCode Web server

BASE_URL="http://localhost:8080/api"
PASSWORD="gamecode"

echo "🧪 Testing GameCode Web Server..."

# Test health endpoint
echo -e "\n1. Testing health endpoint..."
curl -s "$BASE_URL/health" | jq .

# Test authentication
echo -e "\n2. Testing authentication..."
AUTH_RESPONSE=$(curl -s -X POST "$BASE_URL/auth" \
  -H "Content-Type: application/json" \
  -d "{\"password\": \"$PASSWORD\"}")

TOKEN=$(echo $AUTH_RESPONSE | jq -r .token)
echo "Token received: ${TOKEN:0:20}..."

# Test providers list
echo -e "\n3. Testing providers list..."
curl -s "$BASE_URL/providers" \
  -H "Authorization: Bearer $TOKEN" | jq .

# Test chat
echo -e "\n4. Testing chat (streaming)..."
echo "Question: What do you think about computers?"
curl -N -X POST "$BASE_URL/chat" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "ollama",
    "messages": [
      {"role": "user", "content": "What do you think about computers?"}
    ]
  }'

echo -e "\n\n✅ Tests complete!"