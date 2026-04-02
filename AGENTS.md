# Agent Conventions

This document outlines project conventions and rules for AI agents working on this codebase.

## Code Style

- **Rust**: Follow standard Rust idioms and conventions
- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy` - fix warnings before committing
- **Testing**: All features require tests - run `cargo test` before committing

## Documentation

- **README.md**: User-facing CLI/API documentation
- **docs/**: Technical documentation (architecture, API reference, etc.)
- **docs/plans/**: Implementation plans - move completed plans to `docs/plans/completed/`
- Update documentation when adding new features

## Dependencies

- Keep dependencies minimal
- Use `cargo outdated` to check for updates
- Verify new dependencies don't introduce vulnerabilities

## Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers when applicable
- Squash related commits before merging

## Commands

```bash
# Development
cargo build --release
cargo test
cargo clippy

# Installation
cargo install --locked

# Code quality
cargo fmt
cargo clippy -- -D warnings
```

## Project Structure

```
src/
├── cli.rs      # CLI argument parsing
├── lib.rs      # Public API exports
├── scan.rs     # Scan orchestration
├── discovery.rs # Skill discovery
├── git.rs      # Git repository detection
├── paths.rs    # Path configuration
├── parser.rs   # YAML parsing
├── validator.rs # Validation logic
├── prompt.rs   # XML generation
├── models.rs   # Data structures
└── error.rs    # Error types
```

## Key Files

- `paths.jsonc`: Tool directory configurations (embedded at compile time)
- `Cargo.toml`: Dependencies and project metadata
- `docs/architecture.md`: System architecture documentation
