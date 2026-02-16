# Rust Rules

See skill: `/skill rust` for behavior guidance.

## Package Structure

Follow Cargo conventions:

```text
src/
├── main.rs      # Binary entry
├── lib.rs       # Library entry
└── <module>/
    ├── mod.rs
    ├── models.rs
    └── service.rs
```

## Module Organization

Group by feature/domain:

```text
src/
├── users/
├── orders/
└── payments/
```

## Tools

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

See `~/.agents/skills/rust/SKILL.md` for full guidance. See `~/.agents/shared/architecture.md` for architecture principles.
