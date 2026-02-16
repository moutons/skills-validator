---
name: python
description: Python project guidance including src layout, domain-based packaging, pixi/uv package management, separation of concerns, and strict type safety. Use when working on Python projects.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Python

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use pip, poetry, or conda directly
- NEVER use `Any` type
- NEVER put business logic in utils modules

**ALWAYS** do the following:

- ALWAYS use pixi or uv for package management
- ALWAYS use mypy --strict
- ALWAYS group by domain

## Package Manager

- **pixi**: GPU/native deps
- **uv**: Most projects
- **NEVER**: `pip`, `poetry`, `conda`

```bash
pixi run pytest    # with pixi
uv run pytest      # with uv
```

## Package Structure: src Layout

```text
project/
├── src/
│   └── package/
│       ├── __init__.py
│       └── ...
├── tests/
└── docs/
    └── design.md      # Required!
├── pyproject.toml
└── README.md
```

## Group by Domain

`src/users/`, `src/orders/`, `src/payments/` - NOT `utils.py`

## Separate Logic from Infrastructure

- Business logic: pure Python functions
- Infrastructure: DB, APIs, file I/O in separate layers
- Makes testing easier

## Define **init**.py

Explicit public API:

```python
__all__ = ["UserService", "create_user", "User"]
```

## Error Handling

Handle errors, not just happy path. Validate inputs, use custom exceptions.

## Strict Typing

- `mypy --strict`
- Explicit return types
- No `Any`

## Example: Domain Package

```python
# src/users/models.py
from pydantic import BaseModel

class User(BaseModel):
    id: int
    email: str
    name: str

# src/users/service.py
class UserService:
    def __init__(self, repo: UserRepository):
        self.repo = repo

    def get_user(self, id: int) -> User | None:
        return self.repo.get_by_id(id)

# src/users/__init__.py
from .models import User
from .service import UserService

__all__ = ["User", "UserService"]
```
