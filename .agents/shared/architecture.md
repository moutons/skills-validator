# Architecture Guidelines

## Feature-Driven Structure

Organize code by **feature/domain**, not file type:

```text
src/
├── features/
│   ├── users/
│   │   └── ...
│   └── orders/
└── shared/
```

Avoid: `components/`, `utils/`, `helpers/` directories.

## Hexagonal Architecture

Separate core from infrastructure:

- **Core/Domain**: Pure business logic, no external deps
- **Ports**: Interfaces defining interactions
- **Adapters**: Implementations (DB, API, file)

## Syntax: Target Modern Standards

Use latest stable language features. Configure tooling to fail on outdated syntax.

## Strict Type Safety

- Enable strict mode in all languages
- Never use `any`, `Any`, `interface{}`
- Use explicit return types on public functions

## Document Functions

All public APIs need:

- Description of purpose
- Parameter documentation with types
- Return type and meaning
- Error conditions

## Atomic Functions

- One function, one responsibility
- Compose larger behavior from smaller functions
- Avoid "god functions"

## Favor Smaller Files

- Split large modules
- Keep files under ~200-300 lines
- Use exports/index files to organize
