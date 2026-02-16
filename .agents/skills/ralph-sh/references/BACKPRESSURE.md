---
description: |
  Backpressure configuration template for Ralph Wiggum building loop.
  Defines feedback loops that run after every iteration.
  Critical: DO NOT commit if any checks fail.
---

# Backpressure Configuration

These checks run after every iteration. **DO NOT commit if any fail.**

## Feedback Loops

### Tests (Critical)

- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Test command: `npm test` / `pytest` / [project test command]

### Static Analysis

- [ ] Type checks passing
- [ ] Linter passing
- [ ] Build succeeds

## Project-Specific Checks

[Add based on requirements discussion]

## Commit Blocking Rule

The agent MUST NOT commit unless ALL feedback loops pass. Fix issues first.

## Completion Signal

The building loop exits when:

- All IMPLEMENTATION_PLAN.md tasks are checked complete
- All feedback loops green

## Iteration Guidance

- Start with 10-20 max iterations, not 50
- If stuck after 5 iterations on same task, stop and reassess
