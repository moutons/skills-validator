---
name: build-skill
description: Create or update skills following the Agent Skills specification. Use when creating new skills or improving existing skills.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.4.0"
---

# Build Skill

See [agentskills.io/specification](https://agentskills.io/specification) and [opencode.ai/docs/skills](https://opencode.ai/docs/skills/) for the full spec. See
[Claude Skills Best Practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices) for detailed guidance.

## When to Create or Update a Skill

Create or update a skill when:

- Domain needs dedicated behavior guidance
- Context would bloat existing skills
- Would be used independently
- Existing skill needs improvement based on usage

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER add context Claude already has (general programming knowledge, common tools)
- NEVER write in first person ("I can help you...")
- NEVER create skills for trivial/one-off tasks
- NEVER skip frontmatter on markdown files
- NEVER nest references more than one level deep

**ALWAYS** do the following:

- ALWAYS write descriptions in third person
- ALWAYS include "when" in descriptions to specify trigger conditions
- ALWAYS run validation after creating/updating skills
- ALWAYS add frontmatter to all markdown files in skills

## Best Practices

### Core Principles

- **Keep each skill focused on one job**
- **Prefer instructions over scripts** unless you need deterministic behavior or external tooling
- **Write imperative steps** with explicit inputs and outputs
- **Test prompts** against the skill description to confirm the right trigger behavior
- **Default assumption**: Claude is already very smart - only add context it doesn't have

### Setting Degrees of Freedom

Match specificity to task fragility:

- **High freedom** (text instructions): Multiple approaches valid, decisions depend on context
- **Medium freedom** (pseudocode/scripts): Preferred pattern exists, some variation acceptable
- **Low freedom** (exact scripts): Fragile operations, consistency critical, specific sequence required

### Writing Effective Descriptions

**Always write in third person**. Include both what the skill does AND when to use it.

**Good**: "Extracts text and tables from PDF files. Use when working with PDF files or when the user mentions PDFs."

**Avoid**: "I can help you process PDFs" or "You can use this to process PDFs"

### Progressive Disclosure

Keep SKILL.md body under 500 lines. Move detailed content to `references/` for lazy loading.

### Avoid Deeply Nested References

**Keep references one level deep from SKILL.md**. Claude may partially read nested files.

Bad:

```text
SKILL.md → advanced.md → details.md
```

Good:

```text
SKILL.md → references/advanced.md
SKILL.md → references/details.md
```

### Feedback Loops

For quality-critical tasks, implement validation loops:

1. Run validator → fix errors → repeat
2. Include checklists for complex workflows

## Skill Structure

```text
skill-name/
├── SKILL.md              # Required: main instructions
├── scripts/              # Optional: executable code
├── references/           # Optional: additional documentation
│   ├── REFERENCE.md
│   └── CHECKLIST.md
└── assets/              # Optional: templates, images, data
```

## Frontmatter (Required)

### SKILL.md (required fields)

```yaml
---
name: <skill-name>
description: What this skill does and when to use it.
---
```

### All other .md files (required)

```yaml
---
description: |
  Brief description of what this file contains.
  Use pipeline format for multi-line descriptions.
---
```

### Optional Fields

| Field         | Description                           |
| ------------- | ------------------------------------- |
| license       | License name or reference             |
| metadata      | Key-value map (author, version, etc.) |
| compatibility | Environment requirements              |
| allowed-tools | Space-delimited tool list             |

## Process: Creating a New Skill

1. Check existing skills and `~/.agents/shared/` for common content
2. Create directory: `.agents/skills/<name>/`
3. Create SKILL.md with proper frontmatter
4. Add optional `scripts/`, `references/`, `assets/` as needed
5. Add frontmatter to all markdown files
6. Run validation: `skills-validator validate ./my-skill`
7. Update `~/.agents/skills.md` with new skill entry

## Process: Updating Existing Skills

1. Identify gaps through usage observation
2. Review against checklist: `references/CHECKLIST.md`
3. Apply fixes:
   - Description unclear → make more specific
   - Too verbose → move to references/
   - Missing feedback loops → add validation steps
   - Deep references → flatten to one level
4. Re-validate after changes

## Validation

Use [skills-validator](https://crates.io/crates/skills-validator):

```bash
skills-validator validate .agents/skills/my-skill/
```

**ALWAYS** run validation after any changes.

## Templates & Checklists

See these reference files:

- `references/CHECKLIST.md` - Quality checklist before sharing a skill
- `references/TEMPLATE.md` - SKILL.md template

## Example: Creating a New Skill

```bash
# 1. Create directory
mkdir -p .agents/skills/my-new-skill

# 2. Create SKILL.md with frontmatter
cat > .agents/skills/my-new-skill/SKILL.md << 'EOF'
---
name: my-new-skill
description: Does something useful. Use when working with X.
---
# Content here
EOF

# 3. Validate
skills-validator validate .agents/skills/my-new-skill/

# 4. Add to skills.md
# Edit skills.md to add the new skill entry
```

## More

See `~/.agents/skills.md` for progressive disclosure guidelines and available skills.
