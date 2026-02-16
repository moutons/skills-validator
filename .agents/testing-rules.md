# Testing Rules

See skill: `/skill testing` for behavior guidance.

## Test Location

```text
project/
├── src/
└── tests/ or test/
    ├── unit/
    ├── integration/
    └── fixtures/
```

## Test Style

- **Python**: pytest functions (not unittest classes)
- **JS/TS**: Vitest/Jest
- **Rust**: Built-in or rstest
- **Go**: testing package

## HTTP/Web Testing

Use **Playwright** for web interfaces (language-agnostic).

Every published endpoint must have tests.

## Running Tests

- Python: `uv run pytest` or `pixi run pytest`
- JS/TS: `pnpm test`
- Rust: `cargo test`
- Go: `go test`

See `~/.agents/skills/testing/SKILL.md` for full guidance.
