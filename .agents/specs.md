# API Specifications

Reference: [OpenAPI](https://spec.openapis.org/oas/latest.html), [Arazzo](https://spec.openapis.org/arazzo/latest.html), [Overlay](https://spec.openapis.org/overlay/v1.0.0.html)

## General Principles

### Documentation Is Infrastructure

For AI agents, comprehensive docs are required—not optional.

### Be Verbose (in Specs)

Humans understand cryptic names; agents need clear `summary` and `description` on every endpoint and parameter.

### Keep Rules Small

**This file should stay under 100 lines.** For detailed guidance:

- Create project-level `AGENTS.md` directives
- Add rules to `~/.agents/rules/` files
- Use skills for behavior enforcement

Don't bloat this file with exhaustive details—point to references.

### Define Types and Examples

Include examples in schemas:

```yaml
User:
  type: object
  properties:
    id:
      type: integer
      example: 123
```

## Managing Large Specs

- Use OpenAPI Slimmer to strip unused endpoints
- Break into task-specific skills loaded on demand
- Use tags to group by domain

## OpenAPI Requirements

- `operationId`: Descriptive (used as tool names: `getUserById`, NOT `get_1`)
- `summary` + `description`: Clear context
- `tags`: Domain grouping
- `x-` extensions for AI context (`x-summary`, `x-rate-limit`)

## Arazzo (Workflows)

Define multi-step workflows:

```yaml
workflows:
  - workflowId: user-onboarding
    steps:
      - stepId: create-user
        operationRef: "#/paths/~1users/post"
        outputs:
          userId: "$steps.create-user.response.body.id"
```

## Overlay

Add AI annotations without modifying original specs:

```yaml
overlayVersion: "1.0.0"
extends: "./api.yaml"
actions:
  - target: "$.info"
    update:
      x-agent-hint: "Payment API"
```

## Validation

```bash
pnpm dlx @redocly/cli lint api.yaml
pnpm dlx spectral lint api.yaml
```

## OAK Principles

Build on open standards. See [OAK Manifesto](https://github.com/jentic/jentic-public-apis/blob/main/OAK.md).
