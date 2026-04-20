# Decision 0004: Dual Output Personality

**Date:** 2026-04-05 **Status:** Accepted

## Context

The validator produces diagnostics consumed by two very different audiences: humans learning to write better skills, and CI pipelines deciding whether to pass or fail. A single message format serves neither well — encouraging tone feels noisy in JSON; terse machine codes feel hostile to humans.

Two approaches were considered:

1. **Dual personality** — separate human and machine messages per diagnostic, with dedicated formatters
2. **Single message with format hints** — one message string, formatters add/strip decoration

## Decision

Each `Diagnostic` carries two messages: `human_message` (warm, encouraging, with context) and `machine_message` (spare, factual, machine-parseable). Two independent formatters (`format_human`, `format_json`) select the appropriate field.

## Human Formatter

- Groups diagnostics by severity: Info → Suggestion → Warning → Error
- Uses emoji markers: ✅ (Info), 💡 (Suggestion), ⚠️ (Warning), ❌ (Error)
- Shows `human_message` with doc URLs as `→ url` links
- Header includes skill name, sizeyness tier, and reasons
- Summary line with counts per tier
- Falls back to directory name when skill name is unavailable

## JSON Formatter

- Flat diagnostic array using `machine_message` (never `human_message`)
- Kebab-case `check` names via serde serialization of `CheckName` enum
- File paths relative to skill directory (stripped via `strip_prefix`)
- `schema_version: 2` for consumer compatibility detection
- `sizeyness_reasons` array explaining classification
- `exit_code` field computed from **all** diagnostics (unfiltered), not just displayed ones

## Exit Code Semantics

The exit code is computed by `pipeline::exit_code()` using the full unfiltered diagnostic set. If a user passes `--severity warning` to hide Info and Suggestion, the exit code still reflects any Errors present. This prevents CI from accidentally passing when errors are hidden by a severity filter.

In `--strict` mode, Suggestion and Warning also produce exit code 1.

## Rationale

- **Two messages over one**: A message like "Nice — your skill includes a gotchas section, which is one of the highest-value things you can add" is great for humans but useless in JSON. The machine equivalent is "Skill includes gotchas section with content." Trying to derive one from the other (stripping emoji, adding warmth) is fragile and produces mediocre results for both audiences.
- **Parallel formatters over templates**: The two formatters share no logic except severity counting. Human output groups by severity; JSON output is a flat list. Human output uses human_message; JSON uses machine_message. Attempting to share code would create coupling without reducing complexity.
- **Unfiltered exit codes**: A filtered view is a display preference, not a contract change. If errors exist, the process should exit 1 regardless of what the user chose to see. This matches the principle that `--severity` affects what you see, not what the validator finds.

## Alternatives Rejected

**Single message with format hints**: Would require each diagnostic author to write one message that reads well both in a terminal with emoji and in a JSON array. In practice, messages optimized for humans are too verbose for JSON, and messages optimized for JSON feel terse and cold to humans. The dual-message approach lets each diagnostic nail both audiences independently.
