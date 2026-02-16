---
name: project-planning
description: Use flipped interaction pattern to gather requirements exhaustively through questioning, then generate comprehensive project specs. Use when starting new projects.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.2.0"
---

# Project Planning

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER start implementing without clear requirements
- NEVER skip the questioning phase

**ALWAYS** do the following:

- ALWAYS ask questions until you have complete information
- ALWAYS document assumptions

## Flipped Interaction Pattern

Instead of waiting for detailed specs, **ask questions to elicit requirements**. Then synthesize into comprehensive documentation.

### The Pattern

```text
From now on, ask me questions to achieve [goal].
Continue asking until you have enough information.
Then provide [deliverable].
Stop when: [specific criteria]
```

### Questioning Strategy

1. **Start Broad**: What is the core problem?
2. **Users**: Who uses this? What's their background?
3. **Scale**: How many users? What's the growth trajectory?
4. **Integrations**: What external systems?
5. **Data**: What data flows? What's sensitive?
6. **Constraints**: Budget? Timeline? Tech preferences?
7. **Success**: How do you measure success?

### After Questions: Deliver

- Comprehensive requirements document
- Architecture overview
- Suggested tech stack with rationale
- Risks and open questions
- Suggested next steps

## Project Structure in .agent/

Create comprehensive but modular docs:

```text
project/
├── AGENT.md
└── .agent/
    ├── spec/
    │   ├── requirements.md    # What we're building
    │   ├── architecture.md    # System design
    │   └── api/              # Per-service specs
    ├── wiki/
    │   ├── domain.md         # Business logic
    │   └── ops.md            # Operations
    └── links/
        └── resources.md
```

## Requirements Gathering Questions

Ask until you have:

- **Functional**: What does it do? User stories?
- **Non-functional**: Performance? Security? Availability?
- **Integration**: What APIs? External services?
- **Data**: Schema? Migrations? Backup?
- **UI/UX**: Web? Mobile? CLI?
- **Operations**: Deployment? Monitoring? Logging?

## Deliverables

After gathering, create:

1. `spec/requirements.md` - Features, user stories
2. `spec/architecture.md` - High-level design
3. Individual service specs in `spec/api/`
4. `wiki/domain.md` - Business concepts
