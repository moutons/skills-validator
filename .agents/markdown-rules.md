# Markdown Rules

Use markdownlint. Fix all lint issues.

## Installation

```bash
pnpm add -D markdownlint-cli
pnpm dlx markdownlint "**/*.md"
```

## File Naming

Use kebab-case: `getting-started.md`, `api-reference.md`

## Headings

- ATX-style (`#`, `##`, `###`)
- Start with H1, don't skip levels
- Consistent capitalization

## Code Blocks

Always specify language:

````markdown
```python
def hello():
    print("world")
```
````

## Links

Descriptive text, not "click here":

```markdown
[Python Setup](./python-setup.md) # Good [Click here](./python-setup.md) # Bad
```

## Tables

```markdown
| Col 1 | Col 2 |
| ----- | ----- |
| Val   | Val   |
```

## Keep Under ~100 Lines

If longer, split into multiple files.
