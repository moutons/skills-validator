---
name: openapi
description: OpenAPI 3.x specification guidance for creating agent-ready APIs. Use when writing OpenAPI specs.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# OpenAPI

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

Reference: [OpenAPI Spec](https://spec.openapis.org/oas/latest.html)

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use vague operationIds like "get_1"
- NEVER skip examples in schemas

**ALWAYS** do the following:

- ALWAYS use descriptive operationIds (used as tool names)
- ALWAYS include description for AI context

## Required Fields Per Endpoint

- `operationId`: Descriptive (used as tool name)
- `summary`: Short description
- `description`: Detailed AI context
- `tags`: Domain grouping
- `parameters`: With descriptions/examples
- `responses`: Documented with examples

## Operation IDs

```yaml
# Good
getUserById
createOrder
listPayments

# Bad
get_1
user_get
```

## AI Extensions

```yaml
x-summary: "Get user details"
x-rate-limit: 1000
x-auth-type: API Key
```

## Schema Examples

```yaml
User:
  type: object
  properties:
    id:
      type: integer
      example: 123
```

## Structure Large Specs

```text
openapi/
├── users.yaml
├── orders.yaml
└── payments.yaml
```

## Validation

```bash
pnpm dlx spectral lint api.yaml
```

## Example: Complete Endpoint

```yaml
paths:
  /users/{id}:
    get:
      operationId: getUserById
      summary: Get a user by ID
      description: |
        Retrieves a user by their unique identifier.
        Returns the user's profile information including name and email.
      tags:
        - users
      parameters:
        - name: id
          in: path
          required: true
          description: The user's unique ID
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
              example:
                id: 123
                name: John Doe
                email: john@example.com
        "404":
          description: User not found
```

## See Also

- `skill oak` - OAK principles
- `skill arazzo` - Workflows
