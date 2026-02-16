# Shared Safety Rules

## Never Modify Without Explicit Approval

The following must NOT be modified without **explicit user approval**:

- **macOS**: `/Applications`, `/System`, `/usr`, `/bin`, `/sbin`, `/Library`
- **User Data**: `~/.ssh/`, `~/.aws/`, `~/.gnupg/`, `~/Library/`, `~/Dropbox/`, etc.
- **Config**: `~/.config/`, `~/.zshrc`, `~/.gitconfig` (ask first)
- **Global packages**: Never use global npm/yarn/pip (use project-local)

## Allowed With Caution

- `~/src/` - project source code
- Project-local `node_modules`, virtual environments, `target/`
- Project-specific config (`.env`, `package.json`, `Cargo.toml`)

## Project Defaults

All projects use:

- **License**: Apache 2.0
- **Author**: moutons <sdmouton@gmail.com>
- **Versioning**: SemVer (start pre-1.0.0)

## Check for openspec

Always check for project-local openspec config:

- Look for `.openspec.json`, `openspec.json`
- Check `package.json`, `pyproject.toml`

Warn user if no openspec found.
