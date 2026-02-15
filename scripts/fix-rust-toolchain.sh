#!/bin/bash
# Script to fix rust-toolchain references in GitHub/Gitea workflows for zizmor

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

CORRECT_COMMENT="        # zizmor: ignore[unpinned-uses]"
CORRECT_ACTION="dtolnay/rust-toolchain@stable"

# Find all workflow directories
WORKFLOW_DIRS=()
if [ -d "$PROJECT_DIR/.github/workflows" ]; then
    WORKFLOW_DIRS+=("$PROJECT_DIR/.github/workflows")
fi
if [ -d "$PROJECT_DIR/.gitea/workflows" ]; then
    WORKFLOW_DIRS+=("$PROJECT_DIR/.gitea/workflows")
fi

if [ ${#WORKFLOW_DIRS[@]} -eq 0 ]; then
    echo "No workflow directories found"
    exit 0
fi

fix_workflow_file() {
    local file="$1"
    local temp_file
    temp_file=$(mktemp)
    
    # Step 1: Remove any comment lines containing "zizmor" 
    # (we'll re-add the correct one if needed)
    sed '/#.*zizmor/d' "$file" > "$temp_file"
    
    # Step 2: Process the file to fix rust-toolchain uses lines
    awk -v action="$CORRECT_ACTION" -v correct_comment="$CORRECT_COMMENT" '
    /uses:.*dtolnay\/rust-toolchain/ {
        # Check if this line has the correct action
        if ($0 !~ action) {
            # Fix the version to @stable
            gsub(/uses:.*dtolnay\/rust-toolchain@[^ ]*/, "uses: " action)
        }
        
        # Check if preceded by correct comment (on previous line)
        if (prev_line !~ correct_comment) {
            # Insert the correct comment before this line
            print correct_comment
        }
    }
    
    { 
        prev_line = $0
        print 
    }
    ' "$temp_file" > "$file"
    
    rm -f "$temp_file"
    echo "Fixed: $file"
}

echo "Fixing rust-toolchain references in workflows..."

for dir in "${WORKFLOW_DIRS[@]}"; do
    for f in "$dir"/*.yml "$dir"/*.yaml; do
        if [ -f "$f" ]; then
            fix_workflow_file "$f"
        fi
    done
done

echo "Done fixing workflows"
