# Scan CLI Subcommand

## Goal
Add `scan` subcommand to the CLI with mutually exclusive flags `--all`, `--user`, `--repo`, and `--tool <tools>`.

## Context
The CLI uses `clap` for argument parsing. The `scan` command is mutually exclusive with the existing `validate` command.

**Command structure:**
```bash
skills-validator scan [OPTIONS]
skills-validator validate <skill_path>  # existing

# Options for scan:
--all           Scan $CWD→repo root + $HOME
--user          Scan $HOME for all tool directories
--repo          Scan $CWD→repo root (requires git repo)
--tool <TOOLS>  Comma-separated tool names to scan
--dry-run       Discover paths without validating
--verbose       Show detailed output per skill
--json          JSON output format
```

**Edge cases:**
- `--all`, `--user`, `--repo`, `--tool` are mutually exclusive
- No flag defaults to `--all` behavior
- `--tool unknown-tool` → ERROR + emit helptext with available tools

## User Stories

**US-001:** Parse scan subcommand
As a user, I want `skills-validator scan` to be recognized as a valid command.

**US-002:** Enforce mutual exclusivity
As a user, I want an error if I pass `--all --user` together so I know only one scope is allowed.

**US-003:** Default to --all
As a user, I want `skills-validator scan` (no flags) to behave like `--all`.

**US-004:** Show help on unknown tool
As a user, I want `--tool unknown` to show available tools so I can correct my input.

## Acceptance Criteria
- [ ] `cargo run -- scan` parses successfully
- [ ] `cargo run -- scan --all --user` returns error describing mutual exclusivity
- [ ] `cargo run -- scan --tool invalid-tool` returns exit code 2 and shows help with tool list
- [ ] `--dry-run`, `--verbose`, `--json` flags are accepted
- [ ] Help text (`--help`) documents all options

## Completion Signal
<promise>DONE</promise>