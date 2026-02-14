# Justfile for skills-validator

# Default recipe
default: ensure-ci

# Run all CI checks locally
ensure-ci:
  @echo "Running CI checks..."
  just fmt
  just clippy
  just test
  just security
  just markdown
  just build
  @echo "All CI checks passed!"

# Check formatting
fmt:
  cargo fmt --check

# Run clippy lints
clippy:
  cargo clippy -- -D warnings

# Run tests
test:
  cargo test

# Run security audit
security:
  cargo audit

# Lint markdown files
markdown:
  pnpx markdownlint-cli2 '**/*.md'

# Build release
build:
  cargo build --release

# Build for publishing
publish:
  cargo build --release --quiet

# Clean build artifacts
clean:
  cargo clean

# Run all checks including doc tests
full: fmt clippy test security markdown build

# Install dependencies (if needed)
deps:
  cargo fetch
