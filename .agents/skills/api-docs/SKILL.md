---
name: api-docs
description: Open Agentic Knowledge principles for building AI-ready API documentation. Use when creating specs or documentation for AI agents.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# API Docs

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

Reference: [OAK Manifesto](https://github.com/jentic/jentic-public-apis/blob/main/OAK.md)

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER write sparse descriptions that assume context
- NEVER skip examples in schemas
- NEVER ignore error responses

**ALWAYS** do the following:

- ALWAYS be verbose in descriptions for AI consumption
- ALWAYS include descriptive operationId (used as tool names)
- ALWAYS use x- extensions for AI context

## Core Principles

1. **Docs Is Infrastructure**: AI needs complete docs, not optional
2. **AI Doesn't Need Middlemen**: Just needs good documentation
3. **Build on Open Standards**: OpenAPI, Arazzo
4. **Community Driven**: Open knowledge layer for APIs

## For Agent-Ready Specs

- Be verbose in descriptions
- Use descriptive operationId (tool names)
- Include examples in schemas
- Use x- extensions for AI context

## OAK Repository Structure

```text
project/
├── OAK.md           # Manifesto
├── LICENSE.md       # CC0-1.0
├── STRUCTURE.md     # Directory structure
├── apis/
│   └── openapi/    # API specs
└── workflows/       # Arazzo workflows
```

## Example: x- extensions

```yaml
paths:
  /users/{id}:
    get:
      operationId: getUserById
      summary: Get user by ID
      description: |
        Retrieves a user by their unique identifier.
        Returns 404 if user not found.
      x-rate-limit: 1000
      x-auth-type: API Key
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: integer
            example: 123
      responses:
        "200":
          description: User found
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"
        "404":
          description: User not found
```

## See Also

- `skill openapi` - OpenAPI specifics
- `skill arazzo` - Workflow composition
- `specs.md` - Full spec guidance
