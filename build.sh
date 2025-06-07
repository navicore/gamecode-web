#!/bin/bash
set -e

echo "Building GameCode Web..."

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "trunk is not installed. Installing..."
    cargo install trunk
fi

# Build the client
echo "Building client..."
cd client
trunk build --release
cd ..

# Build the server
echo "Building server..."
cargo build --release --manifest-path server/Cargo.toml

echo "Build complete! Run ./run.sh to start the server"