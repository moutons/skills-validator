---
name: specification-writing
description: Write exhaustive, product-ready specifications. Use when creating detailed technical docs.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.2.0"
---

# Specification Writing

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER leave ambiguous requirements
- NEVER skip error cases

**ALWAYS** do the following:

- ALWAYS include examples
- ALWAYS define acceptance criteria

## Goal

Specs should be complete enough that:

- Engineers implement without asking questions
- QA writes tests from it
- Maintainers understand intent

## Principles

### Exhaustive

Cover all edge cases, error conditions, validation rules.

### Precise

No ambiguity, specific types, include examples.

### Actionable

Clear acceptance criteria, known limitations documented.

## Spec Template

````markdown
# [Feature]

## Overview

[2-3 sentences]

## API: METHOD /path

### Request

| Field  | Type   | Required | Description |
| ------ | ------ | -------- | ----------- |
| field1 | string | yes      | Description |

### Response: 200

```json
{ "id": "uuid" }
```
````

### Errors

| Code | Condition     |
| ---- | ------------- |
| 400  | Invalid input |
| 404  | Not found     |

### Edge Cases

- Scenario 1
- Scenario 2

## Acceptance Criteria

- [ ] Criterion 1

````text

## Keep Modular

Many focused specs, not one massive file:

```text

spec/
├── requirements.md
├── architecture.md
├── api/
│ ├── users.yaml
│ └── orders.yaml

````

## Cross-Reference

Link between specs: See `api/users.yaml`

## Review Checklist

- [ ] Endpoints documented
- [ ] Error codes defined
- [ ] Edge cases considered
- [ ] Examples provided
