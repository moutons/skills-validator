---
name: golang
description: Go project guidance including standard Go layout, domain-based packages, interfaces, error handling. Use when working on Go projects.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Golang

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use generic error handling (just `return err`)
- NEVER put business logic in handlers
- NEVER use internal packages from outside

**ALWAYS** do the following:

- ALWAYS use interfaces for dependencies
- ALWAYS wrap errors with context
- ALWAYS group by domain, not by type

## Package Structure

```text
cmd/
├── myapp/
│   └── main.go
internal/           # Private (not importable)
├── user/
│   ├── handler.go
│   ├── service.go
│   └── repository.go
pkg/               # Public libraries
go.mod
```

## Domain Grouping

```text
internal/
├── user/
├── order/
└── payment/
```

NOT `internal/utils/`, `internal/helpers/`

## Interfaces

Define dependencies as interfaces:

```go
type UserRepository interface {
    GetByID(ctx context.Context, id int64) (*User, error)
}
```

## Error Handling

Explicit errors, not generic:

```go
if err != nil {
    return nil, fmt.Errorf("getting user %d: %w", id, err)
}
```

## Tools

```bash
go fmt ./...
go vet ./...
golangci-lint run
go test
```

## Example: Complete Service

```go
// internal/user/service.go
type UserService struct {
    repo UserRepository
}

func NewUserService(repo UserRepository) *UserService {
    return &UserService{repo: repo}
}

func (s *UserService) GetUser(ctx context.Context, id int64) (*User, error) {
    user, err := s.repo.GetByID(ctx, id)
    if err != nil {
        return nil, fmt.Errorf("getting user %d: %w", id, err)
    }
    return user, nil
}

// internal/user/handler.go
type Handler struct {
    svc *UserService
}

func (h *Handler) GetUser(w http.ResponseWriter, r *http.Request) {
    id, err := strconv.ParseInt(chi.URLParam(r, "id"), 10, 64)
    if err != nil {
        http.Error(w, "invalid id", http.StatusBadRequest)
        return
    }

    user, err := h.svc.GetUser(r.Context(), id)
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }

    json.NewEncoder(w).Encode(user)
}
```
