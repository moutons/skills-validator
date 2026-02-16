---
name: ralph-sh
description: Execute the Ralph Wiggum building loop (Phase 3). Run implementation tasks from IMPLEMENTATION_PLAN.md, following BACKPRESSURE.md feedback loops. Use ralph-one.sh for HITL or ralph.sh for AFK autonomous execution.
argument-hint: [task name from specs/ folder]
allowed-tools: Read Write Glob Grep WebFetch WebSearch Bash
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.2.0"
---

# Ralph-sh

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure. See [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for tool integration.

You are executing Phase 3 of the Ralph Wiggum methodology. Your job is to implement tasks from IMPLEMENTATION_PLAN.md, run backpressure checks after each task, and iterate until complete.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER skip backpressure checks
- NEVER commit failing code

**ALWAYS** do the following:

- ALWAYS run one task at a time
- ALWAYS exit after completing one task

## Starting Context

The task to implement: $ARGUMENTS

If no task was provided, use the script selector to choose from available specs in `specs/`.

## Prerequisites

Before running, verify:

1. Phase 1 & 2 completed (`specs/[task-name]/` exists with PRD.md, IMPLEMENTATION_PLAN.md, BACKPRESSURE.md, AGENTS.md)
2. BUILD_PROMPT.md in project root
3. `scripts/ralph.sh` and `scripts/ralph-one.sh` from this skill copied to project root
4. You're on a feature branch (not main)

Copy scripts from `~/.agents/skills/ralph-sh/scripts/` to your project.

## Running the Loop

### Option 1: ralph-one.sh (Recommended First)

```bash
./ralph-one.sh
```

- Runs ONE task at a time
- Stops after each task for observation
- Use when: testing the pattern, debugging, or want HITL

### Option 2: ralph.sh (AFK Mode)

```bash
./ralph.sh
```

- Runs full loop until done or stuck
- Go AFK while it works
- Use when: confident, want autonomous execution

## Execution Workflow

### Step 1: Select Task

The script lists available tasks from `specs/[task-name]/IMPLEMENTATION_PLAN.md`

### Step 2: Read Context Files

Before each task, read these files:

- `specs/[task-name]/IMPLEMENTATION_PLAN.md` - current task details
- `specs/[task-name]/BACKPRESSURE.md` - feedback loop checks
- `specs/[task-name]/AGENTS.md` - agent context
- `BUILD_PROMPT.md` - building loop instructions

### Step 3: Execute Task

Implement the specific steps from IMPLEMENTATION_PLAN.md

### Step 4: Run Backpressure Checks

After each task, run ALL BACKPRESSURE.md checks. Critical checks:

- All tests pass
- Type checks passing
- Linter passing
- Build succeeds

See `references/BACKPRESSURE.md` for full template. **DO NOT commit if any checks fail**

### Step 5: Mark Complete

Update `IMPLEMENTATION_PLAN.md`:

- Check off completed items
- Note any blockers or notes

### Step 6: Loop

Return to Step 1 for next task, or exit if all done

## Stuck Detection

Watch for "STUCK:" markers - same task 5+ attempts, repeated failures. If stuck: stop, analyze, update specs, break into smaller pieces.

**Iteration Guidance:**

- Start with 10-20 max iterations, not 50
- If stuck after 5 iterations on same task, stop and reassess

## Exit Signals

Loop completes when ALL of the following are true:

- All IMPLEMENTATION_PLAN.md tasks checked complete
- All BACKPRESSURE.md checks green

## Templates

See these files for detailed templates:

- `references/IMPLEMENTATION_PLAN.md` - Task structure template
- `references/BACKPRESSURE.md` - Feedback loop template
- `references/AGENTS.md` - Agent context template
