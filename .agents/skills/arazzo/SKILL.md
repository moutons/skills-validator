---
name: arazzo
description: Arazzo specification for composing complex API workflows. Use when defining multi-step agent workflows.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Arazzo

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

Reference: [Arazzo Spec](https://spec.openapis.org/arazzo/latest.html)

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER reference non-existent OpenAPI operations
- NEVER skip error handling in workflows

**ALWAYS** do the following:

- ALWAYS define outputs for steps that will be chained
- ALWAYS include onFailure handlers for critical steps

## Overview

Arazzo defines composable workflows from OpenAPI operations. Essential for multi-step agentic workflows.

## Workflow Structure

```yaml
workflows:
  - workflowId: user-onboarding
    summary: Complete user onboarding
    steps:
      - stepId: create-user
        operationRef: "#/paths/~1users/post"
        parameters:
          body:
            email: "{$inputs.userEmail}"
        outputs:
          userId: "$steps.create-user.response.body.id"
```

## Step Design

Every step needs:

- `stepId`: Unique identifier
- `description`: Clear purpose
- `operationRef`: OpenAPI operation
- `parameters`: Input mapping
- `outputs`: For chaining

## Chaining Steps

```yaml
# Step 2 uses Step 1 output
- stepId: update-profile
  parameters:
    userId: "$steps.create-user.outputs.userId"
```

## Error Handling

```yaml
onFailure:
  - stepId: notify-admin
    operationRef: "#/paths/~1admin/notify/post"
```

## Structure

```text
arazzo/
├── workflows/
│   ├── user-onboarding.yaml
│   └── payment-processing.yaml
└── main.yaml
```

## Example: Complete Workflow

```yaml
workflows:
  - workflowId: order-processing
    summary: Process a customer order
    inputs:
      properties:
        orderId:
          type: integer
          description: The order ID to process
    steps:
      - stepId: validate-order
        operationRef: "#/paths/~1orders~1{orderId}~1validate/post"
        parameters:
          orderId: "$inputs.orderId"
        outputs:
          validated: "$steps.validate-order.response.body.valid"

      - stepId: charge-payment
        operationRef: "#/paths/~1payments/post"
        parameters:
          orderId: "$inputs.orderId"
        onSuccess:
          - stepId: fulfill-order
        onFailure:
          - stepId: notify-failure
```

## See Also

- `skill openapi` - OpenAPI operations
- `skill oak` - OAK principles
