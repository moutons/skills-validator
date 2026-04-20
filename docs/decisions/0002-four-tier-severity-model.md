# Decision 0002: Four-Tier Severity Model with Sizeyness Escalation

**Date:** 2026-04-05 **Status:** Accepted

## Context

The original validator used a two-tier model (warnings and errors). As the check count grew from ~6 to ~30+, this binary classification became too coarse. Positive reinforcement ("your skill has a gotchas section") and gentle nudges ("consider adding examples") were both flattened into "warning," giving skill authors no signal about what matters most.

Three approaches were considered:

1. **Four-tier model** (Info/Suggestion/Warning/Error) with sizeyness-aware escalation
2. **Three-tier model** (Info/Warning/Error) matching common linter conventions
3. **Keep two tiers** and add a separate "praise" output channel

## Decision

Adopt the four-tier model with sizeyness escalation.

## Severity Tiers

| Tier       | Purpose                                              | Exit code             |
| ---------- | ---------------------------------------------------- | --------------------- |
| Info       | Positive reinforcement for good practices            | 0                     |
| Suggestion | Gentle nudge to consider adding something            | 0 (1 with `--strict`) |
| Warning    | Real quality concern affecting agent behavior        | 0 (1 with `--strict`) |
| Error      | Broken, spec-violating, or dangerous                 | 1 always              |

`Severity` derives `Ord` with variants ordered Info < Suggestion < Warning < Error, enabling `>=` filtering.

## Sizeyness Escalation

Skills are classified as Simple, Moderate, or Hefty based on file count, subdirectory count, and orchestration frontmatter fields (see `Sizeyness::from_counts`). Certain checks have a `base_severity` that gets escalated for larger skills via `escalate(base, levels)`, which adds `levels` steps capped at Error.

A missing-examples check might be Suggestion for a simple skill but Warning for a hefty one. This means larger, more complex skills are held to a higher standard — proportional to the impact they have on agent workflows.

Not all checks escalate. Binary detection is always Error regardless of size. Parse failures are always Error. Only content-quality and structural checks use escalation.

## Diagnostic Dual Messages

Each `Diagnostic` carries both `human_message` (warm, encouraging tone with emoji) and `machine_message` (spare, factual). The human formatter uses `human_message`; the JSON formatter uses `machine_message`. Both share the same `severity` and `check_name`.

Each diagnostic also stores `base_severity` alongside the (possibly escalated) `severity`. The exit code is computed from the escalated severity using all diagnostics — even those filtered from display by `--severity`. This prevents a user filtering to Warning level from accidentally hiding an Error that should fail CI.

## Rationale

- **Four tiers over three**: The Suggestion tier fills a real gap. "Consider adding examples" is not a warning — it won't break anything — but it's not just informational praise either. Collapsing it into Warning dilutes the signal; collapsing it into Info loses the nudge.
- **Info for praise, not neutrality**: Most linters use Info for neutral messages. We use it exclusively for positive reinforcement ("you have this and it's valuable"), which makes the validator feel encouraging rather than punitive.
- **Escalation over fixed severity**: A missing description is a minor gap in a 1-file skill but a real problem in a 20-file orchestrated skill. Fixed severity can't express this.
- **base_severity tracking**: Without it, the exit code would need to re-derive escalation or trust the filtered view, both error-prone.

## Alternatives Rejected

**Three-tier model**: Standard in most linters but loses the Suggestion/Warning distinction that matters for skill authors who are learning best practices. The validator's goal is education, not just gatekeeping.

**Two tiers with separate praise channel**: Keeps the warn/error model but routes praise elsewhere. Adds complexity (two output paths) without the graduated nudging that Suggestion provides.
