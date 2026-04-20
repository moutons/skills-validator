# Decision 0001: Layered Analysis Pipeline

**Date:** 2026-04-05 **Status:** Accepted

## Context

The validator needs to grow from ~6 basic checks to ~20-30 checks spanning content quality, structural integrity, referential integrity, and optional security analysis. The current single-pass architecture in `validator.rs` (411 lines) doesn't
support checks that depend on each other's results, and there's no way to classify skill sizeyness before applying severity escalation.

Three approaches were considered:

1. **Layered Analysis Pipeline** — multi-pass architecture where each pass builds on the previous
2. **Bolt-on Checks** — add new check functions to the existing single-pass flow
3. **Plugin Architecture** — trait-based check registration with dependency declarations

## Decision

Adopt the Layered Analysis Pipeline (Approach 1).

## Architecture

The validator becomes a five-pass pipeline:

| Pass | Name                    | Inputs                                | Outputs                                                                                             |
| ---- | ----------------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------- |
| 1    | **Parse**               | Raw SKILL.md                          | `pulldown-cmark` AST, frontmatter fields                                                            |
| 2    | **Structure**           | Skill directory                       | File inventory, sizeyness tier (simple/moderate/hefty), subdirectory map, binary detection          |
| 3    | **Content**             | AST + sizeyness tier                  | Heading analysis, keyword checks, description quality, content diagnostics                          |
| 4    | **References**          | AST + file inventory + sizeyness tier | Reference chain validation, orphan detection, extension field validation (hook scripts exist, etc.) |
| 5    | **Security** (optional) | File inventory + detected scripts     | Semgrep analysis if available, otherwise advisory warnings                                          |

Each pass produces typed `Diagnostic` values with four severity tiers (info/suggestion/warning/error). Sizeyness classification from pass 2 feeds into severity escalation for passes 3-5.

## Rationale

- The pipeline matches natural data dependencies — you need the file inventory before checking references, the AST before content analysis, sizeyness before severity assignment
- Each pass is independently testable
- Clean extension points: new checks slot into the appropriate pass
- Natural refactor of what exists — pass 1 is essentially today's parser

## Alternatives rejected

**Bolt-on Checks:** Faster to ship but `validator.rs` would grow to 800+ lines with no clear structure. Checks can't depend on each other's results without threading context through every function. Likely requires refactoring to the pipeline model
within months.

**Plugin Architecture (`Check` trait):** Most extensible, but over-engineered for ~20-30 checks with a small contributor base. Adds indirection without near-term payoff. Better suited if third-party check plugins become a goal.

## Inflection point: migrate to Plugin Architecture

Revisit this decision and migrate to Approach 3 (trait-based `Check` plugin architecture) when **any one** of these conditions is met:

- **Check count exceeds 50** — at this scale, the pipeline passes become grab-bags and the trait boundary pays for itself in organization alone
- **External contributors are writing checks** — if anyone outside the core team wants to add custom checks (corporate policy rules, framework-specific checks, etc.), they need a stable interface, not knowledge of pipeline internals
- **A check needs to run in multiple passes** — if the same logic applies at both the content and integrity level, the trait model handles this cleanly while the pipeline model forces duplication

When migrating, each pipeline pass becomes a check _category_ (a grouping mechanism), and individual checks within it implement the `Check` trait. The pipeline ordering is preserved as declared dependencies between categories, not between individual
checks.
