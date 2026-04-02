# Invalid Skills

Place invalid skill fixtures here. These should fail validation in predictable ways.

## Subdirectories

### `missing-frontmatter/`
SKILL.md with no TOML frontmatter block.

### `malformed-toml/`
SKILL.md with invalid TOML syntax (unclosed brackets, bad quotes, etc.).

### `missing-name/`
SKILL.md with frontmatter but no required `name` field.

### `invalid-name/`
SKILL.md with name that violates naming rules (special chars, too long, etc.).

### `unknown-fields/`
SKILL.md with valid structure but unrecognized frontmatter fields.

## Expected Behavior
Each should produce specific error messages matching the validation rules.