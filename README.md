# skills-validator

[![CI](https://github.com/moutons/skills-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/moutons/skills-validator/actions/workflows/ci.yml)
[![CodeQL](https://github.com/moutons/skills-validator/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/moutons/skills-validator/actions/workflows/github-code-scanning/codeql)

A Rust reimplementation of the [agentskills/skills-ref](https://github.com/agentskills/agentskills/tree/main/skills-ref) Python library with a layered validation pipeline, configurable severity tiers, and sizeyness-aware escalation.

> **Note:** This library validates skills according to the Agent Skills specification, informed by the OpenCode and Claude Code implementations.

## Specification

This implementation follows the Agent Skills specification and key implementations:

- **Official Spec**: <https://agentskills.io/specification>
- **OpenCode Implementation**: <https://opencode.ai/docs/skills/>
- **Claude Code Implementation**: <https://code.claude.com/docs/en/skills>

## Installation

```bash
cargo install --locked
```

Or from a specific version:

```bash
cargo install --locked --git https://github.com/moutons/skills-validator.git --tag v0.2.0
```

## Usage

### CLI

```bash
# Validate a skill directory
skills-validator validate path/to/skill

# Validate with strict mode (exit 1 on warnings/suggestions)
skills-validator validate path/to/skill --strict

# Filter output by severity
skills-validator validate path/to/skill --severity warning

# JSON output for CI pipelines
skills-validator validate path/to/skill --output-format json

# Read skill properties (outputs YAML)
skills-validator read-properties path/to/skill

# Generate <available_skills> XML for agent prompts
skills-validator to-prompt path/to/skill-a path/to/skill-b

# Scan for skills across multiple tool directories
skills-validator scan --all                    # Scan repo + user home
skills-validator scan --user                   # Scan user home only
skills-validator scan --repo                   # Scan repo root only
skills-validator scan --tool claude-code       # Scan specific tool(s)

# Generate a default config file
skills-validator setup

# Generate shell completions
skills-validator completions bash
skills-validator completions zsh
skills-validator completions fish
```

### CLI Flags

| Flag | Description |
| ---- | ----------- |
| `--strict` | Exit 1 on warnings or suggestions (not just errors) |
| `--output-format human\|json` | Output format (default: `human`) |
| `--severity info\|suggestion\|warning\|error` | Only show diagnostics at or above this severity |
| `--json` | **Deprecated.** Alias for `--output-format json`; emits a deprecation warning |
| `--verbose` | Show detailed output |
| `--dry-run` | Discover skills without validating (scan subcommand) |

### Rust API

```rust
use skills_validator::{run_pipeline, read_properties, to_prompt};

fn main() {
    // Validate a skill directory using the five-pass pipeline
    let result = run_pipeline("my-skill");
    for diagnostic in &result.diagnostics {
        println!("[{}] {}", diagnostic.severity, diagnostic.message);
    }

    // Read skill properties
    let props = read_properties("my-skill").unwrap();
    println!("Skill: {} - {}", props.name, props.description);

    // Generate prompt for available skills
    let prompt = to_prompt(&["skill-a", "skill-b"]);
    println!("{}", prompt);
}
```

> **Migration note:** `validate()` and `ValidationResult` are deprecated. Use `run_pipeline()` and `PipelineResult`/`Vec<Diagnostic>` instead. See CHANGELOG.md for the full migration guide.

## Validation Pipeline

Validation runs as a five-pass pipeline. Each pass can emit diagnostics at any severity level.

| Pass | Name | What it checks |
| ---- | ---- | -------------- |
| 1 | **Parse** | YAML frontmatter parsing via pulldown-cmark AST |
| 2 | **Structure** | File inventory, sizeyness classification, binary file detection |
| 3 | **Content** | Frontmatter field validation, body quality, positive reinforcement |
| 4 | **References** | Markdown link chain walking (up to 5 hops), orphan file detection |
| 5 | **Security** | Remote execution patterns, optional semgrep integration |

### Diagnostic Severity Tiers

Diagnostics use a four-tier severity model:

| Tier | Purpose | Exit code |
| ---- | ------- | --------- |
| **Info** | Positive reinforcement for good practices | 0 |
| **Suggestion** | Gentle nudge to consider adding something | 0 (1 with `--strict`) |
| **Warning** | Real quality concern affecting agent behavior | 0 (1 with `--strict`) |
| **Error** | Broken, spec-violating, or dangerous | 1 always |

### Sizeyness Escalation

Skills are classified by sizeyness (Simple, Moderate, Hefty) based on file count, total size, and body length. A check that produces a **suggestion** for a simple skill may escalate to a **warning** or **error** for a moderate or hefty one. This means larger, more complex skills are held to a higher standard.

### Frontmatter Validation

Validates against the [Agent Skills specification](https://agentskills.io/specification):

| Field | Required | Constraints |
| ----- | -------- | ----------- |
| `name` | Yes | Max 64 characters. Lowercase letters, numbers, and hyphens only. Must not start or end with a hyphen. Must not contain consecutive hyphens (`--`). Must match directory name. |
| `description` | Yes | Max 250 characters. Non-empty. |
| `license` | No | License name or reference to a bundled license file. |
| `compatibility` | No | Max 500 characters. Indicates environment requirements. |
| `metadata` | No | Arbitrary key-value mapping for additional metadata. |
| `allowed-tools` | No | Space-delimited list of pre-approved tools. (Experimental) |

**Unknown fields** produce warnings (demoted from errors in 0.2.0).

### Content Validation

Warns when skill content is missing key directive words:

| Keyword | Guidance |
| ------- | -------- |
| `never` | A well-written skill includes clear directives to NEVER do something and preferably ALWAYS do an alternative. See <https://agentskills.io/what-are-skills> |
| `always` | A well-written skill includes clear directives to ALWAYS do something in certain circumstances. See <https://agentskills.io/what-are-skills> |
| `when` | A well-written skill contains 'when' statements to inform the agent of what conditions trigger certain behaviors. See <https://code.claude.com/docs/en/skills> |
| `example` | A well-written skill contains examples to inform the agent of what to do in commonly encountered circumstances. See <https://opencode.ai/docs/skills> |

### Claude Code Extensions

Claude Code supports additional fields beyond the official spec. These generate **warnings** but not errors:

- `argument-hint` - Hint shown during autocomplete
- `disable-model-invocation` - Prevent automatic loading
- `user-invocable` - Hide from / menu
- `model` - Model to use when skill is active
- `context` - Run in forked subagent context
- `agent` - Which subagent type to use
- `hooks` - Hooks scoped to skill lifecycle

See <https://code.claude.com/docs/en/skills> for details.

## Configuration

The validator supports configurable thresholds via a TOML config file.

### Config File Location

`$XDG_CONFIG_HOME/skills-validator/config.toml` (typically `~/.config/skills-validator/config.toml`)

Generate a default config with:

```bash
skills-validator setup
```

### Config Sections

- `[sizeyness]` - Thresholds for Simple/Moderate/Hefty classification
- `[content]` - Body length limits, directive keyword requirements
- `[references]` - Chain walk depth, orphan detection settings
- `[security]` - Semgrep integration, remote execution pattern detection

### Override Order

Configuration is resolved in this order (later wins):

1. Compiled defaults
2. Config file (`config.toml`)
3. Environment variables (`SKILLS_VALIDATOR_<SECTION>_<KEY>`)
4. CLI flags

## Agent Prompt Integration

Use `to-prompt` to generate the suggested `<available_skills>` XML block for your agent's system prompt:

```xml
<available_skills>
<skill>
<name>
my-skill
</name>
<description>
What this skill does and when to use it
</description>
<location>
/path/to/my-skill/SKILL.md
</location>
</skill>
</available_skills>
```

## Skill Scanning

The `scan` command discovers and validates skills across multiple agent tool directories.

### Scan Modes

- `--all`: Scan both the current git repository and user home directory
- `--user`: Scan only the user home directory for all tool directories
- `--repo`: Scan only the current git repository
- `--tool <tools>`: Scan specific tool(s) (comma-separated list)

### Options

- `--dry-run`: Discover skills without validating
- `--verbose`: Show detailed output for each skill
- `--output-format json`: Output results as JSON

### Configuration

Tool paths are configured in `paths.jsonc` which is embedded at compile time. The tool directory templates support:

- `$HOME` or `~`: User home directory
- `$REPO_ROOT`: Git repository root (detected via git2)

### Exit Codes

- `0`: All skills valid (warnings may be present)
- `1`: Some skills invalid
- `2`: Scan or configuration error

### Examples

```bash
# Scan all known locations
skills-validator scan --all

# Scan only your home directory
skills-validator scan --user

# Scan the current repository
skills-validator scan --repo

# Scan for specific tools
skills-validator scan --tool claude-code,opencode

# Dry run to see what would be scanned
skills-validator scan --all --dry-run
```

## Development

```bash
cargo build --release
cargo test
cargo clippy
```

## Security Considerations When Building Skills

Script execution introduces security risks. Consider:

- **Sandboxing**: Run scripts in isolated environments
- **Allowlisting**: Only execute scripts from trusted skills
- **Confirmation**: Ask users before running potentially dangerous operations
- **Logging**: Record all script executions for auditing

The validator's security pass detects remote execution patterns (curl-pipe-bash, etc.) and can optionally run semgrep for deeper script analysis. Configure semgrep integration in the config file.

See <https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview#security-considerations> for more details.

## License

Apache 2.0
