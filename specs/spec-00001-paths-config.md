# Paths Config Module

## Goal

Create `src/paths.rs` module that parses and exposes the embedded `paths.jsonc` configuration with tool directory templates.

## Context

The `paths.jsonc` file (JSON with Comments) is embedded via `include_str!` at compile time. It maps tool names (e.g., "claude", "opencode") to their skill directory paths using template variables `$HOME` and `$REPO_ROOT`.

**File location:** `src/paths.jsonc` (create if missing, embed in `src/paths.rs`)

**Data structures:**

```rust
pub struct ToolConfig {
    pub name: String,
    pub documentation: Option<String>,
    pub directories: Vec<String>,  // templates like "$HOME/.claude/skills"
}

pub struct PathsConfig {
    pub tools: HashMap<String, ToolConfig>,  // kebab-case key -> config
}
```

## User Stories

**US-001:** Load paths config from embedded JSONC As a developer, I want the config loaded at startup so that tool paths are immediately available.

**US-002:** Normalize tool names to kebab-case As a developer, I want "Claude Code" normalized to "claude-code" so that lookups are consistent regardless of input casing.

**US-003:** List all available tools As a user, I want to see available tools when I pass an invalid tool name so I know what to use.

## Acceptance Criteria

- [ ] `PathsConfig::load()` returns parsed config or error
- [ ] `PathsConfig::get_tool("claude")` returns correct `ToolConfig`
- [ ] `PathsConfig::get_tool("CLAUDE")` returns same result (case-insensitive)
- [ ] `PathsConfig::tool_names()` returns sorted list of tool names
- [ ] `PathsConfig::has_tool("unknown")` returns `false`
- [ ] Unit tests cover: empty config, single tool, multiple tools, missing file

## Completion Signal
