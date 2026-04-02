# Multi-Location Skills

Place skills with identical `name` fields here to test duplicate detection.

## Purpose

Tests the duplicate skill warning feature. Two SKILL.md files with the same frontmatter `name` should both be validated but trigger a warning about tool precedence.

## Subdirectories

### `skill-a/` and `skill-b/`
Both should have SKILL.md files with identical `name` field in frontmatter, but different content or paths.

## Example

```
multi-location/
├── skill-a/
│   └── SKILL.md     # name = "my-skill", description = "Version A"
└── skill-b/
    └── SKILL.md     # name = "my-skill", description = "Version B"
```

Both should validate successfully but produce a duplicate warning.