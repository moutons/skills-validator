# Duplicate Detection

## Goal
Detect skills with identical `name` fields in frontmatter across discovered files and report them as warnings.

## Context
When multiple SKILL.md files have the same `name` in their frontmatter, tools may implement precedence rules differently. The scanner should warn users about this potential conflict.

**Data structures:**
```rust
pub struct DuplicateSkill {
    pub skill_name: String,
    pub locations: Vec<PathBuf>,
}

pub fn detect_duplicates(
    results: &[ValidationResult],
) -> Vec<DuplicateSkill>;
```

**Behavior:**
- Group by `skill_name` (from frontmatter, not filename)
- If count > 1, it's a duplicate
- Both instances are still validated normally
- Warning message suggests checking tool precedence docs

## User Stories

**US-001:** Detect same-name skills
As a user, I want to know when two skills share the same `name` in frontmatter.

**US-002:** Validate both duplicates
As a scanner, I want to validate both instances of a duplicate skill, not skip one.

**US-003:** Clear warning output
As a user, I want the warning to tell me which files conflict and suggest checking precedence.

## Acceptance Criteria
- [ ] Two skills with same `name` produce a `DuplicateSkill`
- [ ] Skills with different names or missing names are not flagged
- [ ] Warning output lists all conflicting file paths
- [ ] Validation still runs on all instances (no skipping)
- [ ] Unit tests: same name, different names, missing name field

## Completion Signal
<promise>DONE</promise>