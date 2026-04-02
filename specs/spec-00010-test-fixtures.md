# Test Fixtures

## Goal

Create comprehensive test fixtures in `tests/fixtures/skills/` covering all edge cases for skill validation and scanning.

## Context

Fixtures are organized into subdirectories by category. Real skills from different registries should be added here for realistic testing.

**Directory structure:**

```text
tests/fixtures/skills/
├── valid/
│   ├── minimal/          # Bare minimum valid skill
│   ├── complete/         # All optional fields populated
│   └── multi-file/       # Skills with referenced files
├── invalid/
│   ├── missing-frontmatter/
│   ├── malformed-toml/
│   ├── missing-name/
│   ├── invalid-name/
│   └── unknown-fields/
├── edge-cases/
│   ├── unicode-content/
│   ├── large-file/
│   ├── empty-optional-fields/
│   └── circular-references/
└── multi-location/
    ├── skill-a/          # Same name, different location
    └── skill-b/
```

**Note:** User will populate with real skills from Claude Code, OpenAI, OpenCode, etc.

## User Stories

**US-001:** Valid fixtures pass validation As a test suite, I need valid skills that should always pass.

**US-002:** Invalid fixtures fail predictably As a test suite, I need invalid skills with known error patterns.

**US-003:** Edge cases cover boundaries As a test suite, I need skills that test parsing limits.

**US-004:** Multi-location tests duplicates As a test suite, I need duplicate skills to test warning logic.

## Acceptance Criteria

- [ ] `valid/minimal/SKILL.md` exists with minimal required fields
- [ ] `valid/complete/SKILL.md` exists with all optional fields
- [ ] `invalid/missing-frontmatter/SKILL.md` has no frontmatter
- [ ] `invalid/malformed-toml/SKILL.md` has invalid TOML syntax
- [ ] `edge-cases/unicode-content/SKILL.md` has non-ASCII characters
- [ ] `multi-location/skill-a/SKILL.md` has same name as `skill-b/SKILL.md`
- [ ] Placeholder README in each dir explaining what fixtures should go there

## Completion Signal
