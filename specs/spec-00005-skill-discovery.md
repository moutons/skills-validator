# Skill Discovery

## Goal

Discover all SKILL.md files within expanded directory paths, collecting their locations and tool associations.

## Context

Once paths are expanded, the scanner walks each directory recursively to find `SKILL.md` files. Uses `walkdir` crate for efficient traversal.

**Data structures:**

```rust
pub struct DiscoveredSkill {
    pub path: PathBuf,
    pub tool_name: String,
    pub directory: PathBuf,  // parent directory that was scanned
}

pub struct DiscoveryResult {
    pub skills: Vec<DiscoveredSkill>,
    pub skipped_dirs: Vec<PathBuf>,  // non-existent or inaccessible
    pub errors: Vec<DiscoveryError>,
}
```

**Function signature:**

```rust
pub fn discover_skills(
    directories: &[(String, PathBuf)],  // (tool_name, expanded_path)
) -> DiscoveryResult;
```

## User Stories

**US-001:** Find all SKILL.md files As a scanner, I want to find every `SKILL.md` in a directory tree.

**US-002:** Handle missing directories silently As a scanner running `--user` mode, I want to skip non-existent directories without error.

**US-003:** Track which tool owns each skill As a scanner, I want each discovered skill to know its source tool for reporting.

**US-004:** Report inaccessible directories As a user, I want to know if a directory exists but cannot be read (permissions).

## Acceptance Criteria

- [ ] Given a directory with nested SKILL.md files, all are discovered
- [ ] Non-existent directories go into `skipped_dirs`, not `errors`
- [ ] Each `DiscoveredSkill` has correct `tool_name` from input
- [ ] Symlinks are followed (not treated as errors)
- [ ] Unit tests cover: empty directory, nested skills, mixed valid/invalid dirs

## Completion Signal
