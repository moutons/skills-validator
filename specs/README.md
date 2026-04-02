# Scan Feature Specifications

This directory contains implementation specifications for the scan feature. Each spec is designed to be implemented in a single 5-10 minute autonomous iteration.

## Spec Index

| Spec | Description | Priority |
|------|-------------|----------|
| [spec-00001-paths-config](spec-00001-paths-config.md) | Embed and parse paths.jsonc configuration | High |
| [spec-00002-scan-cli](spec-00002-scan-cli.md) | Implement scan subcommand with mutual-exclusive flags | High |
| [spec-00003-path-expansion](spec-00003-path-expansion.md) | Expand ~ and $HOME in paths using dirs crate | High |
| [spec-00004-git-detection](spec-00004-git-detection.md) | Detect git repository root using git2 | High |
| [spec-00005-skill-discovery](spec-00005-skill-discovery.md) | Recursively discover SKILL.md files | High |
| [spec-00006-parallel-validation](spec-00006-parallel-validation.md) | Parallel skill validation with rayon | High |
| [spec-00007-duplicate-detection](spec-00007-duplicate-detection.md) | Detect duplicate skill names and warn | Medium |
| [spec-00008-output-formatters](spec-00008-output-formatters.md) | Human-readable and JSON output formatters | Medium |
| [spec-00009-exit-codes](spec-00009-exit-codes.md) | Granular exit codes for CI integration | Medium |
| [spec-00010-test-fixtures](spec-00010-test-fixtures.md) | Create test fixture directory with skill examples | Medium |
| [spec-00011-integration-tests](spec-00011-integration-tests.md) | Integration tests using tempfile and git repos | Medium |

## Test Fixtures

Add real skill examples to `tests/fixtures/skills/`:
- `valid/` - Skills that pass validation
- `invalid/` - Skills with errors
- `edge-cases/` - Malformed frontmatter, missing fields, etc.
- `multi-location/` - Skills for duplicate detection testing