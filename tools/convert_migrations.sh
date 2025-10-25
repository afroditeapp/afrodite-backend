#!/bin/sh

# Script to convert PostgreSQL migrations to SQLite migrations

set -e

POSTGRES_DIR="migrations/postgres"
SQLITE_DIR="migrations/sqlite"

# Find all .sql files in postgres directory
find "$POSTGRES_DIR" -name "*.sql" ! -path "*/00000000000000_diesel_initial_setup/*" | while read -r file; do
    # Get the relative path from postgres dir
    relative_path="${file#$POSTGRES_DIR/}"

    # Destination file
    dest_file="$SQLITE_DIR/$relative_path"

    # Create destination directory if needed
    mkdir -p "$(dirname "$dest_file")"

    # Copy and convert
    sed -e 's/BIGSERIAL PRIMARY KEY             /INTEGER PRIMARY KEY AUTOINCREMENT /g' \
        -e 's/BIGSERIAL PRIMARY KEY/INTEGER PRIMARY KEY AUTOINCREMENT/g' \
        -e 's/BIGINT PRIMARY KEY  /INTEGER PRIMARY KEY /g' \
        -e 's/BIGINT PRIMARY KEY/INTEGER PRIMARY KEY/g' \
        -e 's/BYTEA/BLOB /g' \
        "$file" > "$dest_file"
done
