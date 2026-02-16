---
name: architecture-design
description: Design scalable microservices architectures with clear service boundaries, data stores, and API contracts. Use when designing system architecture.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Architecture Design

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER create services that share databases
- NEVER skip OpenAPI contracts

**ALWAYS** do the following:

- ALWAYS design service boundaries around business domains
- ALWAYS use domain-appropriate datastores

## Architecture Principles

### Stateless Services

All business logic services are stateless containers.

### API-First

Every service exposes strict, versioned OpenAPI v3.

### CLI for Each Service

Each service includes a CLI wrapping its OpenAPI spec.

### Domain-Appropriate Datastores

- **PostgreSQL**: Relational data
- **Qdrant**: Vector embeddings (RAG)
- **S3**: Blob storage
- **Redis**: Queues, caching

## Service Design Process

### 1. Identify Domain Boundaries

Each service should:

- Have single, well-defined responsibility
- Own its data
- Expose via API
- Be deployable independently

### 2. Define Service Contract

For each service:

```yaml
## Service Name
### Primary Datastore
### API Endpoints
- `POST /v1/resource`: Description
- `GET /v1/resource/{id}`: Description

### CLI Commands
- `cli command --arg`: Description
```

### 3. Document Data Flows

Show how data moves between services:

```text
User → API Gateway → Service A → Service B → Database
```

## Service Template

```text
### Service N: [Name]

[What it does]

* **Primary Datastore**: [PostgreSQL/Redis/Qdrant/S3]
* **API Endpoints**:
  * `POST /v1/...`: ...
  * `GET /v1/...`: ...

* **CLI Implementation**:
  * `cli action --arg`: ...
```

## Create Modular Specs

In `.agent/spec/api/`, create one file per service:

```text
spec/
├── architecture.md      # Overview
└── api/
    ├── service-a.yaml
    ├── service-b.yaml
    └── service-c.yaml
```

## Example: E-commerce Architecture

```text
User Service (PostgreSQL)
  - POST /v1/users
  - GET /v1/users/{id}

Order Service (PostgreSQL)
  - POST /v1/orders
  - GET /v1/orders/{id}
  → calls User Service

Payment Service (PostgreSQL + Redis)
  - POST /v1/payments
  → calls Order Service
  → uses Redis for rate limiting

Inventory Service (PostgreSQL)
  - GET /v1/inventory/{product_id}
```

## Example Structure

See `~/.agents/skills/project-planning` for requirements gathering. See `skill openapi` for OpenAPI spec format.
