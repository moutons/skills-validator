---
description: |
  Implementation plan template for Ralph Wiggum methodology.
  Contains ordered tasks with specific steps, verification criteria, and file lists.
  Each task should be small enough to implement in a single iteration.
---

# Implementation Plan: [Task Name]

## Overview

[Brief summary of what will be built]

**Estimated iterations:** [X-Y]

---

## Task 1: [Task Name]

**Goal:** [One sentence]

**Steps:**

1. [Specific step with file paths]
2. [Specific step]
3. [Specific step]

**Verification:**

- [ ] `npm run build` passes
- [ ] [Other checks from BACKPRESSURE.md]

**Files touched:**

- `path/to/file.ts` (new/modified)
- `path/to/other.ts`

---

## Task 2: [Task Name]

[Same structure...]

---

## Build Order Summary

```text
Task 1 → Task 2 → Task 3 (sequential)
              ↓
Task 4 ←→ Task 5 (can parallelize)
```

**Critical path:** [list]

---

## Notes for Implementing Agent

- [Important reminders]
- [Patterns to follow]
- [Things to avoid]
