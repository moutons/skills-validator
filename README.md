# skills-validator

Reference library for Agent Skills - Rust implementation.

> **Note:** This library validates skills according to the Agent Skills specification. Unknown fields will cause validation to fail.

## Specification

This implementation follows the Agent Skills specification:

- **Official Spec**: https://agentskills.io/specification
- **OpenCode Implementation**: https://opencode.ai/docs/skills/
- **Claude Implementation**: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview

## Installation

```bash
cargo install --path .
```

## Usage

### CLI

```bash
# Validate a skill
skills-validator validate path/to/skill

# Read skill properties (outputs YAML)
skills-validator read-properties path/to/skill

# Generate <available_skills> XML for agent prompts
skills-validator to-prompt path/to/skill-a path/to/skill-b
```

### Rust API

```rust
use skills_validator::{validate, read_properties, to_prompt};

fn main() {
    // Validate a skill directory
    let problems = validate("my-skill");
    if !problems.is_empty() {
        println!("Validation errors: {:?}", problems);
    }

    // Read skill properties
    let props = read_properties("my-skill").unwrap();
    println!("Skill: {} - {}", props.name, props.description);

    // Generate prompt for available skills
    let prompt = to_prompt(&["skill-a", "skill-b"]);
    println!("{}", prompt);
}
```

## Agent Prompt Integration

Use `to-prompt` to generate the suggested `<available_skills>` XML block for your agent's system prompt. This format is recommended for Anthropic's models, but Skill Clients may choose to format it differently based on the model being used.

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

The `<location>` element tells the agent where to find the full skill instructions.

## Pre-commit Validation

Before committing changes to skills in this repository or updating skills in `~/.agents/skills/`, run:

```bash
# Validate all skills in a directory
./scripts/validate-skills.sh ~/.agents/skills
```

Or add as a git hook:

```bash
cp scripts/validate-skills.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## License

Apache 2.0
