# Parallel Validation

## Goal

Validate discovered skills in parallel using `rayon`, collecting all results without failing fast.

## Context

The scanner validates each discovered `SKILL.md` by calling the existing validation logic. Rayon provides CPU-bound parallelism via work-stealing thread pool.

**Function signature:**

```rust
pub fn validate_skills(
    skills: &[DiscoveredSkill],
) -> Vec<ValidationResult>;

pub struct ValidationResult {
    pub skill: DiscoveredSkill,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub skill_name: Option<String>,  // from frontmatter
}
```

**Behavior:**

- Use `par_iter()` from rayon for parallel processing
- Collect ALL results before returning (no early termination)
- Individual validation failures do not stop other validations

## User Stories

**US-001:** Validate in parallel As a scanner, I want to validate 100+ skills efficiently using all CPU cores.

**US-002:** Collect all errors As a user, I want to see every validation issue, not just the first failure.

**US-003:** Handle malformed skills gracefully As a scanner, if one skill has unparseable TOML, I want it logged but other validations continue.

## Acceptance Criteria

- [ ] `validate_skills()` processes all input skills
- [ ] Uses `rayon::prelude::*` for parallel iteration
- [ ] Malformed skills produce `ValidationResult` with `valid: false` and errors, not panics
- [ ] Empty input returns empty output (no crash)
- [ ] Integration test: validate 100+ skills, verify all processed

## Completion Signal
