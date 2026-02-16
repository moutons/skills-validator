---
name: github-actions
description: Create, review, and update GitHub Actions workflow files. Use when working with .github/workflows/ or .github/actions/ directories.
license: Apache-2.0
metadata:
  author: moutons <sdmouton@gmail.com>
  version: "0.2.0"
---

# GitHub Actions

See [agentskills.io/specification](https://agentskills.io/specification) for skill structure.

## NEVER vs ALWAYS

**NEVER** do the following:

- NEVER use mutable action tags like @main or @master
- NEVER skip running zizmor and actionlint on workflows

**ALWAYS** do the following:

- ALWAYS pin actions to SHA or use version tags
- ALWAYS run security analysis (zizmor) on new workflows
- ALWAYS lint workflows before committing

## Workflow File Location

```text
.github/
├── workflows/
│   ├── ci.yml
│   ├── pr.yml
│   └── release.yml
└── actions/
    └── my-action/
        └── action.yml
```

## Checking Action Versions

### gh release view

Check for the most recent version of a GitHub action:

```bash
gh release view --repo owner/repo --json tagName --jq '.tagName'
```

Example:

```shell
❯ gh release view --repo katyo/publish-crates --json tagName --jq '.tagName'
v2
```

### actions-up

Check for outdated actions with [actions-up](https://github.com/azat-io/actions-up):

```bash
npx actions-up
# or
pnpx actions-up
```

For dry-run (see versions without updating):

```bash
npx actions-up --dry-run
```

This scans `.github/workflows/*.yml` and `.github/actions/*/action.yml` to find all actions and check for updates.

## Linting Workflows

### actionlint

Lint workflow files with [actionlint](https://github.com/rhysd/actionlint):

```bash
# Install: https://github.com/rhysd/actionlint#installation
actionlint --verbose .github/workflows/*.yml
```

**Installation:**

```bash
# macOS
brew install actionlint

# Binary releases
curl -sSfL https://raw.githubusercontent.com/rhysd/actionlint/main/install.sh | sh
```

## Security Analysis

### zizmor

Static analysis for workflow security with [zizmor](https://docs.zizmor.sh):

```bash
# Install: https://docs.zizmor.sh/install
zizmor .github/workflows/*.yml
```

zizmor detects:

- Pinned action vulnerabilities
- Secret exposure risks
- Permissions issues
- Workflow security anti-patterns

### dtolnay/rust-toolchain

The `dtolnay/rust-toolchain` action is a meta-action that reads from a file, so it cannot be pinned to a SHA. Suppress zizmor warnings:

```yaml
# zizmor: ignore[unpinned-uses]
uses: dtolnay/rust-toolchain@stable
```

This tells zizmor to skip the unpinned-uses check for this line.

## Best Practices

### Action Versioning

- **Pin to SHA** for security: `uses: actions/checkout@8a5a27`
- **Use tags** for convenience: `uses: actions/checkout@v4`
- **Avoid** mutable tags like `@main` or `@master`

### Workflow Structure

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup
        run: |
          # minimal setup steps

      - name: Test
        run: npm test
```

### Permissions

Use minimal permissions:

```yaml
permissions:
  contents: read
  pull-requests: write
```

## Example: Complete Node.js CI

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node: [18, 20, 22]
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
          cache: "npm"

      - run: npm ci
      - run: npm test

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: test-results-${{ matrix.node }}
          path: test-results/
```

## Common Patterns

### Node.js CI

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: "20"
    cache: "npm"
```

### Python CI

```yaml
- uses: actions/setup-python@v5
  with:
    python-version: "3.11"
    cache: "pip"
```

### Docker Build

```yaml
- uses: docker/build-push-action@v6
  with:
    context: .
    push: ${{ github.event_name != 'pull_request' }}
```

## References

See [references/WORKFLOW_EXAMPLES.md](references/WORKFLOW_EXAMPLES.md) for common workflow patterns.
