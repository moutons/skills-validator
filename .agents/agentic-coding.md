# Agentic Coding Guidelines

See skill: `/skill agentic-coding` for behavior guidance.

## Key Principles

1. **Human-AI Collaboration**: Humans design, AI implements
2. **Design First**: Always create `docs/design.md` before coding
3. **8-Step Process**: Requirements → Flow → Utilities → Data → Node → Implementation → Optimization → Reliability

## Design Patterns

- **Agent**: Autonomous decisions
- **Workflow**: Sequential pipelines
- **RAG**: Retrieval + Generation
- **MapReduce**: Split/combine
- **Structured Output**: Pydantic/JSON Schema

## docs/design.md

Must include:

- Requirements (user stories)
- Flow with mermaid diagram
- Design patterns used
- Utility functions (input/output)
- Node design (prep/exec/post)

See `~/.agents/skills/agentic/SKILL.md` for full guidance.
