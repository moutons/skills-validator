# Justfile for skills-validator

# Default recipe lists all recipes
default:
  just --list

# Run all CI checks locally
ensure-ci:
  @echo "Running CI checks..."
  just fmt
  just clippy
  just test
  just security
  just markdown
  just workflows
  just build
  @echo "All CI checks passed!"

# Check Rust formatting
fmt:
  cargo fmt --check

# Run clippy lints
clippy:
  cargo clippy -- -D warnings

# Run tests
test:
  cargo test
  just workflows

# Run security audit
security:
  cargo audit

# Lint markdown files
markdown:
  pnpx markdownlint-cli2 '**/*.md'

# Lint, validate, and pin GitHub workflows
workflows:
  ./scripts/fix-rust-toolchain.sh
  actionlint --verbose
  pnpx pin-github-action --allow katyo/publish-crates ./.github/workflows

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
