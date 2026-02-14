#!/bin/bash
# Pre-commit hook to validate skills in ~/.agents/skills/
# Usage: Add to .git/hooks/pre-commit or run manually before commits

SKILLS_DIR="${1:-~/.agents/skills}"
VALIDATOR="./target/release/skills-validator"

if [ ! -f "$VALIDATOR" ]; then
    echo "Building skills-validator..."
    cargo build --release
fi

echo "Validating skills in $SKILLS_DIR..."

errors=0
for skill in "$SKILLS_DIR"/*/; do
    if [ -d "$skill" ]; then
        echo "Checking: $skill"
        if ! "$VALIDATOR" validate "$skill" 2>&1; then
            echo "FAILED: $skill"
            errors=$((errors + 1))
        fi
    fi
done

if [ $errors -gt 0 ]; then
    echo ""
    echo "Validation failed for $errors skill(s)"
    exit 1
fi

echo "All skills validated successfully!"
exit 0
