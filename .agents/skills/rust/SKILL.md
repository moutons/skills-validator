---
name: rust
description: Rust project guidance including Cargo conventions, module organization, separation of concerns, strict safety. Use when working on Rust projects.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Rust

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use unwrap() in production code
- NEVER group by type (models/, utils/)

**ALWAYS** do the following:

- ALWAYS use Result for error handling
- ALWAYS group by feature/domain

## Package Structure

```text
src/
├── main.rs      # Binary
├── lib.rs       # Library
└── <module>/
    ├── mod.rs
    ├── models.rs
    └── service.rs
```

## Workspace

```text
workspace/
├── Cargo.toml
├── crate_a/
└── crate_b/
```

## Group by Feature

`src/users/`, `src/orders/` - NOT `src/models/`, `src/utils/`

## Separate Logic

Pure logic in separate modules from DB/API code.

## Public API

Define in `lib.rs`:

```rust
pub use users::{User, UserService};
```

## Error Handling

Use `Result` with custom error types. Avoid `unwrap()`.

## Tools

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Example: Feature Module

```rust
// src/users/mod.rs
pub mod models;
pub mod service;

pub use models::User;
pub use service::UserService;

// src/users/models.rs
#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
}

// src/users/service.rs
pub struct UserService {
    // dependencies
}

impl UserService {
    pub fn get_user(&self, id: i64) -> Result<Option<User>, Error> {
        // business logic
    }
}
```
