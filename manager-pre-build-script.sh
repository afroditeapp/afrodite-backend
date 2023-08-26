#!/bin/bash -eux

# Pre-build script for building the backend with app-manager service.
# Deletes the current database and creates a new one.

git submodule init
git submodule update

# Note: Make sure that libsqlite3-dev is installed before running this script.

if ! command -v diesel &> /dev/null; then
    echo "diesel is not installed. Installing..."
    cargo install diesel_cli@2.1.0 --no-default-features --features sqlite
fi

DATABASE_FILE=database/current/current.db

mkdir -p database/current

if [ -f "$DATABASE_FILE" ]; then
    echo "Deleting previous database..."
    rm "$DATABASE_FILE"
fi

DATABASE_URL="database/current/current.db" diesel database reset

echo "Script completed successfully!"
