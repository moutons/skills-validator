# Decision 0003: Compile-Time Path Embedding

**Date:** 2026-04-05 **Status:** Accepted

## Context

The scan system needs to know where each agent tool stores skills on disk. There are 27+ tools (Claude Code, OpenCode, GitHub Copilot, Cursor, etc.) each with 1-3 directory templates using variables like `$HOME` and `$REPO_ROOT`. This configuration needs to be maintained and distributed with the validator.

Three approaches were considered:

1. **Compile-time embedding** via `include_str!` from a JSONC file
2. **Runtime config file** loaded from disk at startup
3. **Hardcoded constants** in Rust source

## Decision

Embed `paths.jsonc` at compile time using `include_str!("../paths.jsonc")`.

## Implementation

The `PathsConfig::load()` method:

1. Reads the embedded JSONC string (already in the binary)
2. Strips comments using a custom JSONC parser that handles `//`, `/* */`, and respects string escaping
3. Parses the resulting JSON into `ToolConfig` structs
4. Normalizes tool keys to kebab-case for case-insensitive lookup
5. Skips the `_unsupported` section (tools that don't support skills)

Path templates use variables expanded at runtime: `$HOME`/`~` (user home), `$REPO_ROOT` (git repo root), `$CWD` (working directory).

## Rationale

- **Self-contained binary**: The validator works without any external files. No "which paths.jsonc do I use?" confusion, no version skew between binary and config.
- **JSONC over JSON**: Comments document why each tool's paths are what they are. A custom comment stripper is needed since standard JSON parsers reject comments, but it's ~50 lines and well-tested.
- **Kebab-case normalization**: Users type tool names inconsistently (`ClaudeCode`, `claude_code`, `CLAUDE-CODE`). Normalizing to kebab-case at load time means all lookups are case-insensitive without per-call normalization.
- **_unsupported section**: Documents tools that were evaluated and explicitly excluded (Aider, Amazon Q, etc.), preventing repeated "should we add X?" discussions.

## Trade-offs

- **Adding a new tool requires recompilation.** This is acceptable: new tools appear infrequently, and the validator is distributed as a compiled binary anyway. Contributors update `paths.jsonc` and rebuild.
- **No user customization of tool paths.** Users can't add custom tools without forking. If this becomes a need, a runtime override file (loaded after the embedded defaults) would be the extension point.

## Alternatives Rejected

**Runtime config file**: Adds a file-not-found failure mode, requires documenting where to put it, and creates version skew risk when the binary is updated but the config isn't. The XDG config system (`config.rs`) already handles user-facing configuration — path data is reference data, not user preferences.

**Hardcoded constants**: Loses the documentation value of JSONC comments and makes the tool list harder to review and update. Rust constants would scatter tool definitions across code rather than keeping them in one reviewable file.
