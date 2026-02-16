---
name: project-config
description: Project-level agent configuration guidance including openspec. Use when setting up project-specific agent behavior.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Project Config

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER assume project-local config without checking
- NEVER skip the openspec check

**ALWAYS** do the following:

- ALWAYS check for project-local openspec first
- ALWAYS warn user if openspec is missing

## What is openspec?

OpenSpec defines project-local agent configuration. It tells agents how to behave in a specific project.

## Finding openspec

Always check for project-local openspec:

1. Look in project root
2. Check `~/src/` subdirectories
3. Look for `.openspec.json`, `openspec.json`
4. Check `package.json`, `pyproject.toml`

## Warn User If Missing

If no openspec found, warn immediately:

> "No project-local openspec found. Project may lack proper configuration."

## Creating openspec

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "skills": ["python", "testing"],
  "rules": ["../.agents/python-rules.md"],
  "structure": {
    "src": "src",
    "tests": "tests"
  }
}
```

## Key Fields

- `name`: Project identifier
- `version`: For compatibility
- `skills`: Required skills for this project
- `rules`: Additional rule files
- `structure`: Source/test paths

## Project AGENTS.md

For project-specific guidance, create `AGENTS.md` in project root:

```markdown
# Project AGENTS.md

See ~/.agents/ for global rules.

## Project-Specific

[Override or add project rules here]
```

## Example: Python Project openspec

```json
{
  "name": "data-processor",
  "version": "1.0.0",
  "skills": ["python", "testing"],
  "structure": {
    "src": "src",
    "tests": "tests"
  },
  "lint": "ruff",
  "test": "pytest"
}
```

## See Also

- `skills-rules.md` - Creating skills
- `build-skill` - Building new skills
