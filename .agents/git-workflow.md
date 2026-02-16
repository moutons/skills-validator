---
description: |
  Git workflow and source control standards for projects.
  Enforces conventional commits and describes branching strategy.
---

# Git Workflow Standards

## Commit Messages

### Format: Conventional Commits

```text
type: brief description
type(scope): detailed description
```

**Structure:**

- Type: lowercase, no punctuation
- Scope: optional, in parentheses
- Description: under 72 characters, imperative mood

### Types

| Type        | Description                     |
| ----------- | ------------------------------- |
| `feat:`     | New features                    |
| `fix:`      | Bug fixes                       |
| `test:`     | Test-related changes            |
| `docs:`     | Documentation updates           |
| `refactor:` | Code refactoring                |
| `chore:`    | Maintenance tasks               |
| `style:`    | Code style changes (formatting) |
| `perf:`     | Performance improvements        |
| `ci:`       | CI/CD changes                   |

### Examples

```text
feat: add user authentication flow
fix: resolve memory leak in data processor
test: add integration tests for payment API
docs: update API endpoint documentation
refactor: extract common utilities into shared module
chore: update dependencies to latest versions
```

### Imperative Mood

**Good**: "add feature", "fix bug", "update documentation"  
**Bad**: "added feature", "fixed bug", "updated documentation"

## Branch Strategy

### Main Branches

- `main` - Production-ready code, protected
- `develop` - Integration branch for next release

### Feature Branches

```text
feature/TICKET-description
fix/TICKET-description
```

Examples:

- `feature/user-authentication`
- `fix/payment-validation`

### Naming

- Use lowercase
- Use hyphens (-) as separators
- Include ticket/issue number when applicable

## Workflow

### Starting Work

1. Update main: `git pull origin main`
2. Create branch: `git checkout -b feature/description`

### While Working

1. Commit frequently with atomic changes
2. Write descriptive commit messages
3. Push regularly: `git push -u origin branch-name`

### Before Merging

1. Rebase on main: `git rebase main`
2. Run tests locally
3. Ensure commit messages follow conventions

### After Merge

1. Delete branch locally: `git branch -d feature-name`
2. Delete branch remotely: `git push origin --delete feature-name`

## Commit Best Practices

### Atomic Commits

Each commit should:

- Be self-contained
- Pass all tests
- Represent one logical change

### Commit Message Body

When needed, add body after blank line:

```text
feat: add user registration

Implement user registration with email verification.
Includes validation and error handling.

Closes #123
```

## Pre-Commit Hooks

Consider using commitlint to enforce conventional commits:

```bash
npm install --save-dev @commitlint/cli @commitlint/config-conventional
```

## Resources

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Conventional Commits Spec](https://www.conventionalcommits.org/en/v1.0.0/)
