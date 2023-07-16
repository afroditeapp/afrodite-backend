#!/bin/bash -eux

# Pre-build script for the app-manager service.
# Deletes the current database and creates a new one.

git submodule init
git submodule update

if ! command -v sqlx &> /dev/null; then
    echo "sqxl is not installed. Installing..."
    cargo install sqlx-cli@0.6.3 --no-default-features --features sqlite,sqlite,rustls
fi

DATABASE_FILE=database/current/current.db

mkdir -p database/current

if [ -f "$DATABASE_FILE" ]; then
    echo "Deleting previous database..."
    rm "$DATABASE_FILE"
fi

sqlx database setup

echo "Script completed successfully!"
