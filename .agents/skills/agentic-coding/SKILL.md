---
name: agentic-coding
description: LLM/AI development guidance including human-AI collaboration, 8-step process, design patterns. Use when building LLM/AI applications.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.3.0"
---

# Agentic Coding

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

See `~/.agents/shared/safety.md` for safety rules. See `~/.agents/shared/architecture.md` for architecture principles.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER skip the design document phase
- NEVER implement without clear requirements
- NEVER mix concerns (keep utilities separate from flow logic)

**ALWAYS** do the following:

- ALWAYS create docs/design.md before coding
- ALWAYS use structured output (Pydantic/JSON Schema)
- ALWAYS separate API utilities from business logic

## Key Principles

1. **Human-AI Collaboration**: Humans design, AI implements
2. **Design First**: Create `docs/design.md` before coding
3. **KISS**: Keep it simple

## 8-Step Process

1. Requirements (Human)
2. Flow Design (Human + AI)
3. Utilities (Human + AI)
4. Data Design (AI)
5. Node Design (AI)
6. Implementation (AI)
7. Optimization (Human + AI)
8. Reliability (AI)

## docs/design.md

Must include:

- Requirements (user stories)
- Flow with mermaid diagram
- Applicable patterns (Agent, RAG, MapReduce, etc.)
- Utility functions (input/output)
- Node design (prep/exec/post)

## Design Patterns

- **Agent**: Autonomous decisions
- **Workflow**: Sequential pipelines
- **RAG**: Retrieval + Generation
- **MapReduce**: Split and combine
- **Structured Output**: Pydantic/JSON Schema

## Project Structure

```text
my_project/
├── main.py
├── nodes.py
├── flow.py
├── utils/           # One file per API
│   └── call_llm.py
└── docs/
    └── design.md
```

## Node Pattern

```python
class MyNode(Node):
    def prep(self, shared):
        return shared["input"]
    def exec(self, data):
        return process(data)
    def post(self, shared, data, result):
        shared["output"] = result
```

## Utility Functions

- One file per external API
- Include `main()` for testing
- Avoid vendor lock-in

## Example: Simple RAG Pipeline

```python
# utils/embeddings.py
from openai import AsyncOpenAI

client = AsyncOpenAI()

async def get_embedding(text: str) -> list[float]:
    response = await client.embeddings.create(
        model="text-embedding-3-small",
        input=text
    )
    return response.data[0].embedding

# nodes/retrieve.py
class RetrieveNode(Node):
    def prep(self, shared):
        return shared["query"]

    def exec(self, query):
        embedding = get_embedding(query)
        results = vector_store.search(embedding, k=5)
        return results

    def post(self, shared, query, results):
        shared["context"] = results
```
