# Skills Configuration

See [agentskills.io](https://agentskills.io) and [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for skill specifications.

## Progressive Disclosure

Skills follow progressive disclosure per the spec:

1. **Metadata** (~100 tokens): `name` and `description` loaded at startup
2. **Instructions** (<5000 tokens recommended): Full SKILL.md loaded when activated
3. **Resources** (as needed): Files in `scripts/`, `references/`, `assets/` loaded on-demand

## Skill Structure

```text
skill-name/
├── SKILL.md              # Required: main instructions
├── scripts/              # Optional: executable code
├── references/           # Optional: additional documentation
│   ├── REFERENCE.md
│   └── FORMS.md
└── assets/               # Optional: templates, images, data
```

## Skill Frontmatter

```yaml
---
name: skill-name
description: What this skill does and when to use it.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.1.0"
---
```

Optional fields per spec:

- `compatibility` - environment requirements
- `allowed-tools` - pre-approved tools (experimental)

## Limits (Non-Skill Files)

All non-skill files in `~/.agents/` should be:

- Under ~3KB
- Focused and scoped

This prevents context bloat and keeps agents efficient.

## Available Skills

| Skill                          | Description                                                |
| ------------------------------ | ---------------------------------------------------------- |
| `/skill python`                | Python: pixi/uv, src layout, domain packaging              |
| `/skill javascript-typescript` | JS/TS: pnpm, features, barrel files                        |
| `/skill rust`                  | Rust: Cargo, modules                                       |
| `/skill golang`                | Go: standard layout, interfaces                            |
| `/skill testing`               | Testing: pytest, Playwright                                |
| `/skill agentic-coding`        | LLM/AI: 8-step process, design patterns                    |
| `/skill project-planning`      | Requirements gathering via flipped interaction             |
| `/skill architecture-design`   | Microservices architecture design                          |
| `/skill specification-writing` | Exhaustive, product-ready specs                            |
| `/skill openapi`               | OpenAPI 3.x: agent-ready specs                             |
| `/skill arazzo`                | Arazzo: workflow composition                               |
| `/skill api-docs`              | OAK: AI-ready API documentation                            |
| `/skill project-config`        | Project-level agent configuration                          |
| `/skill build-skill`           | Create new skills following agentskills.io spec            |
| `/skill ralph-sh`              | Ralph Wiggum building loop: execute IMPLEMENTATION_PLAN.md |
| `/skill github-actions`        | GitHub Actions: workflow creation, linting, security       |

## More

See `~/.agents/skills-rules.md` for detailed skill creation guidance.
