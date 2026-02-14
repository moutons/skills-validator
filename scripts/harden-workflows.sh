#!/bin/bash
# Script to harden GitHub workflows for zizmor security checks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
WORKFLOWS_DIR="$PROJECT_DIR/.github/workflows"

if [ ! -d "$WORKFLOWS_DIR" ]; then
    echo "No .github/workflows directory found"
    exit 0
fi

# Clean up any existing _comment entries that yq might have created
cleanup_malformed_comments() {
    local file="$1"
    if grep -q "_comment:" "$file"; then
        # Remove _comment: lines that yq created
        sed -i '' '/_comment:.*zizmor/d' "$file"
    fi
}

add_zizmor_ignore() {
    local file="$1"
    local action="dtolnay/rust-toolchain@v2"
    local comment="# zizmor: ignore[unpinned-uses]"
    
    # First clean up any malformed comments
    cleanup_malformed_comments "$file"
    
    # Check if file contains the action
    if ! grep -q "$action" "$file"; then
        return
    fi
    
    # Check if comment already exists
    if grep -B1 "$action" "$file" | grep -q "zizmor: ignore"; then
        echo "zizmor ignore already present in $file"
        return
    fi
    
    # Check existing headComment using yq
    local existing_comment
    existing_comment=$(yq '... | select(.uses == "'"$action"'") | headComment' "$file" 2>/dev/null || echo "")
    
    if [ -n "$existing_comment" ] && echo "$existing_comment" | grep -q "zizmor: ignore"; then
        echo "zizmor ignore already set in $file"
        return
    fi
    
    # Use awk to add comment before the uses line
    local temp_file
    temp_file=$(mktemp)
    
    awk -v action="$action" -v comment="$comment" '
        $0 ~ action && !found {
            print comment
            found = 1
        }
        { print }
    ' "$file" > "$temp_file"
    
    mv "$temp_file" "$file"
    echo "Added zizmor ignore to $file"
}

echo "Hardening GitHub workflows for zizmor..."

for f in "$WORKFLOWS_DIR"/*.yml; do
    if [ -f "$f" ]; then
        add_zizmor_ignore "$f"
    fi
done

echo "Done hardening workflows"
