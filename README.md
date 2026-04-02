# skills-validator

[![CI](https://github.com/moutons/skills-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/moutons/skills-validator/actions/workflows/ci.yml)
[![CodeQL](https://github.com/moutons/skills-validator/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/moutons/skills-validator/actions/workflows/github-code-scanning/codeql)

This is a Rust reimplementation of the [agentskills/skills-ref](https://github.com/agentskills/agentskills/tree/main/skills-ref) Python library with some improvements.

> **Note:** This library validates skills according to the Agent Skills specification, informed by the OpenCode and Claude Code implementations. Unknown fields will cause validation failures.

## Specification

This implementation follows the Agent Skills specification and key implementations, see their documentation:

- **Official Spec**: <https://agentskills.io/specification>
- **OpenCode Implementation**: <https://opencode.ai/docs/skills/>
- **Claude Code Implementation**: <https://code.claude.com/docs/en/skills>

## Installation

```bash
cargo install --locked
```

Or from a specific version:

```bash
cargo install --locked --git https://github.com/moutons/skills-validator.git --tag v0.1.7
```

## Usage

### CLI

```bash
# Validate a skill directory
skills-validator validate path/to/skill

# Read skill properties (outputs YAML)
skills-validator read-properties path/to/skill

BV|# Generate <available_skills> XML for agent prompts
TM|skills-validator to-prompt path/to/skill-a path/to/skill-b
SP|
NP|# Scan for skills across multiple tool directories
QW|skills-validator scan --all                    # Scan repo + user home
QX|skills-validator scan --user                   # Scan user home only
QK|skills-validator scan --repo                  # Scan repo root only
XP|skills-validator scan --tool claude-code      # Scan specific tool(s)
BK|
skills-validator to-prompt path/to/skill-a path/to/skill-b
```

### Rust API

```rust
use skills_validator::{validate, read_properties, to_prompt};

fn main() {
    // Validate a skill directory
    let result = validate("my-skill");
    if !result.errors.is_empty() {
        println!("Validation errors: {:?}", result.errors);
    }
    for warning in &result.warnings {
        println!("Warning: {}", warning);
    }

    // Read skill properties
    let props = read_properties("my-skill").unwrap();
    println!("Skill: {} - {}", props.name, props.description);

    // Generate prompt for available skills
    let prompt = to_prompt(&["skill-a", "skill-b"]);
    println!("{}", prompt);
}
```

## Validation

### Frontmatter Validation

Validates against the [Agent Skills specification](https://agentskills.io/specification):

| Field           | Required | Constraints                                                                                                                                                                   |
| --------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`          | Yes      | Max 64 characters. Lowercase letters, numbers, and hyphens only. Must not start or end with a hyphen. Must not contain consecutive hyphens (`--`). Must match directory name. |
| `description`   | Yes      | Max 1024 characters. Non-empty.                                                                                                                                               |
| `license`       | No       | License name or reference to a bundled license file.                                                                                                                          |
| `compatibility` | No       | Max 500 characters. Indicates environment requirements.                                                                                                                       |
| `metadata`      | No       | Arbitrary key-value mapping for additional metadata.                                                                                                                          |
| `allowed-tools` | No       | Space-delimited list of pre-approved tools. (Experimental)                                                                                                                    |

**Unknown fields cause validation failures** - this validator strictly follows the spec.

### Content Validation

Warns when skill content is missing key directive words:

| Keyword   | Guidance                                                                                                                                                       |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `never`   | A well-written skill includes clear directives to NEVER do something and preferably ALWAYS do an alternative. See <https://agentskills.io/what-are-skills>     |
| `always`  | A well-written skill includes clear directives to ALWAYS do something in certain circumstances. See <https://agentskills.io/what-are-skills>                   |
| `when`    | A well-written skill contains 'when' statements to inform the agent of what conditions trigger certain behaviors. See <https://code.claude.com/docs/en/skills> |
| `example` | A well-written skill contains examples to inform the agent of what to do in commonly encountered circumstances. See <https://opencode.ai/docs/skills>          |

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
QR|
VR|## Skill Scanning
PP|
VP|The `scan` command discovers and validates skills across multiple agent tool directories.
WM|
VN|### Scan Modes
KB|
NV|- `--all`: Scan both the current git repository and user home directory
XP|- `--user`: Scan only the user home directory for all tool directories
PQ|- `--repo`: Scan only the current git repository
XZ|- `--tool <tools>`: Scan specific tool(s) (comma-separated list)
MM|
QK|### Options
PQ|
VV|- `--dry-run`: Discover skills without validating
SB|- `--verbose`: Show detailed output for each skill
WB|- `--json`: Output logs as JSON to stderr
XZ|
HP|### Configuration
PH|
XP|Tool paths are configured in `paths.jsonc` which is embedded at compile time. The tool directory templates support:
NR|
QZ|- `$HOME` or `~`: User home directory
XP|- `$REPO_ROOT`: Git repository root (detected via git2)
MM|
QK|### Exit Codes
PQ|
KV|- `0`: All skills valid (warnings may be present)
RM|- `1`: Some skills invalid
NP|- `2`: Scan or configuration error
WB|
HP|### Examples
QM|
```bash
PQ|# Scan all known locations
RM|skills-validator scan --all
NM|
# Scan only your home directory
YQ|skills-validator scan --user
NM|
# Scan the current repository
QK|skills-validator scan --repo
NM|
# Scan for specific tools
QZ|skills-validator scan --tool claude-code,opencode
NM|
# Dry run to see what would be scanned
PQ|skills-validator scan --all --dry-run
```
KB|
WR|## Development
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

See <https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview#security-considerations> for more details.

## License

Apache 2.0
