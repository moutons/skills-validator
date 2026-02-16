---
description: |
  Quality checklist for verifying skills before sharing.
  Based on Claude best practices - verify all items before publishing.
---

# Checklist for Effective Skills

Before sharing a skill, verify all items below.

## Core Quality

- [ ] Description is specific and includes key terms
- [ ] Description includes both what the skill does AND when to use it
- [ ] SKILL.md body is under 500 lines
- [ ] Additional details are in separate files (if needed)
- [ ] No time-sensitive information (or in "old patterns" section)
- [ ] Consistent terminology throughout
- [ ] Examples are concrete, not abstract
- [ ] File references are one level deep
- [ ] Progressive disclosure used appropriately
- [ ] Workflows have clear steps

## Frontmatter & Structure

- [ ] SKILL.md has valid frontmatter with name and description
- [ ] All .md files in skill have frontmatter with description
- [ ] name field follows constraints (lowercase, hyphens, 1-64 chars)
- [ ] description is 1-1024 chars, describes what AND when to use
- [ ] Directory structure follows spec (scripts/, references/, assets/)

## Code and Scripts

- [ ] Scripts solve problems rather than punt to Claude
- [ ] Error handling is explicit and helpful
- [ ] No "voodoo constants" (all values justified)
- [ ] Required packages listed in instructions and verified as available
- [ ] Scripts have clear documentation
- [ ] No Windows-style paths (all forward slashes)
- [ ] Validation/verification steps for critical operations
- [ ] Feedback loops included for quality-critical tasks

## Naming

- [ ] Skill name is in gerund form (verb-ing) or noun phrase
- [ ] Name is specific and descriptive
- [ ] Consistent with other skills in collection

## Testing

- [ ] Tested with actual usage scenarios
- [ ] Description triggers skill when expected
- [ ] Instructions provide enough context without being verbose

## Best Practices Applied

- [ ] Degrees of freedom appropriately matched to task
- [ ] Concise - only adds context Claude doesn't already have
- [ ] Third-person description style
- [ ] References nested no more than one level deep
- [ ] Checklists included for multi-step workflows
