#!/bin/bash
# Script to fix all deprecated API calls in test files

set -e

echo "Fixing deprecated API calls in netabase_store tests..."

# Function to replace deprecated calls in a file
fix_file() {
    local file="$1"
    echo "Processing: $file"
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Replace deprecated calls
    sed -i 's/\.create_redb(/.create(/g' "$file"
    sed -i 's/\.update_redb(/.update(/g' "$file"
    sed -i 's/\.delete_redb(/.delete(/g' "$file"
    sed -i 's/\.read_redb(/.read(/g' "$file"
    
    # Also fix documentation comments
    sed -i 's/create_redb/create/g' "$file"
    sed -i 's/update_redb/update/g' "$file"
    sed -i 's/delete_redb/delete/g' "$file"
    sed -i 's/read_redb/read/g' "$file"
}

# Find and fix all test files in main workspace
echo "Fixing main workspace tests..."
find tests -name "*.rs" -type f | while read file; do
    if grep -q "create_redb\|update_redb\|delete_redb\|read_redb" "$file"; then
        fix_file "$file"
    fi
done

# Find and fix all test files in boilerplate
echo "Fixing boilerplate tests..."
find boilerplate/tests -name "*.rs" -type f 2>/dev/null | while read file; do
    if grep -q "create_redb\|update_redb\|delete_redb\|read_redb" "$file"; then
        fix_file "$file"
    fi
done || true

echo "Done! Backup files created with .bak extension"
echo ""
echo "To verify changes:"
echo "  grep -r 'create_redb\\|update_redb\\|delete_redb' tests/ boilerplate/tests/"
echo ""
echo "To remove backups:"
echo "  find . -name '*.rs.bak' -delete"
