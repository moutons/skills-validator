# Claude Code Configuration

Read `AGENTS.md` first -- it contains the development methodology for this repository. This file adds Claude-specific configuration.

## Model Preferences for Subagents

When using the `model` parameter on Agent tool calls:

- `haiku` -- file lookups, formatting checks, simple grep/glob searches, running `just` recipes
- `sonnet` -- code review, standard implementations, test writing, debugging
- `opus` -- architectural decisions, complex refactors, spec writing, ambiguous problems

Default to `sonnet` when unsure. Escalate to `opus` only when the task requires deep reasoning across multiple files or domains.

## Tool Permissions

Tool permissions are defined in `.claude/settings.json`. Key restrictions:

- All formatting, linting, and testing must go through `just` recipes
- Direct invocation of `npm`, `yarn`, `pnpm`, and similar tools is not allowed
- Use `just fmt`, `just clippy`, `just test` instead

## Plugins

This project uses:

- `superpowers@claude-plugins-official` -- spec-driven development workflow
- `elements-of-style@superpowers-marketplace` -- clear, concise writing
