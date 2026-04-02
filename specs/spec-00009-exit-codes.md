# Exit Codes

## Goal
Define and implement granular exit codes for the scan command to support CI/CD decision-making.

## Context
Exit codes must be bounded (≤10 total including 0). They should distinguish between success, validation failures, and system/scan errors.

**Exit codes:**
| Code | Meaning |
|------|---------|
| 0 | All skills valid, no errors |
| 1 | Some skills invalid (validation errors) |
| 2 | Configuration error (invalid tool, missing config) |
| 3 | Git error (not in repo with `--repo`) |
| 4 | No skills found (empty scan) |
| 5 | I/O error (permission denied, unreadable files) |

**Behavior:**
- `--dry-run` uses codes 2, 3, 5 for system errors (not validation codes)
- Partial failures use the most severe applicable code
- Exit codes are per-command, not cumulative

## User Stories

**US-001:** Success returns 0
As a CI pipeline, I want exit code 0 when all skills are valid.

**US-002:** Invalid skills return 1
As a CI pipeline, I want exit code 1 when any skill has validation errors.

**US-003:** System errors return appropriate codes
As a CI pipeline, I want different codes for "not in git repo" vs "validation failed".

**US-004:** Dry-run respects system codes
As a user, `--dry-run` with bad config returns error codes, not 0.

## Acceptance Criteria
- [ ] All valid skills → exit 0
- [ ] Any invalid skill → exit 1
- [ ] Unknown tool → exit 2
- [ ] `--repo` outside git → exit 3
- [ ] Zero skills discovered → exit 4
- [ ] Permission denied on directory → exit 5
- [ ] `--dry-run` + bad config → exit 2 (not 0)

## Completion Signal
<promise>DONE</promise>