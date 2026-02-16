# Skills Rules

## Philosophy

Skills should be small, focused, and loadable on-demand. Rules files should point to skills and shared resources.

## Creating Skills

### When to Create a Skill

Create a new skill when:

- A domain needs dedicated behavior guidance
- Context would bloat other skills or rules
- The skill would be used independently

### Skill Structure

```text
~/.agents/skills/<name>/
└── SKILL.md
```

### Frontmatter Required

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

### Content Guidelines

- Keep under ~80 lines
- Reference shared rules: `~/.agents/shared/safety.md`, `shared/architecture.md`
- Include practical examples
- Point to detailed rules files for expansion

### Naming

- Lowercase, hyphenated: `python`, `javascript`, `build-skill`
- Max 64 characters
- Match directory name

## Maintaining Skills

### Deduplicate

Extract common content to `~/.agents/shared/`:

- Safety rules → `shared/safety.md`
- Architecture → `shared/architecture.md`

### Cross-Reference

Skills should reference:

- `~/.agents/shared/safety.md`
- `~/.agents/shared/architecture.md`
- Other relevant skills

### Keep Small

If a skill or rule file approaches 100 lines:

1. Extract to shared/ if common
2. Create dedicated rule file pointing to skill
3. Keep skill focused on behavior, not reference

## Validation

- All files under 100 lines
- All files under ~2KB
- Skills loadable via `/skill <name>`
