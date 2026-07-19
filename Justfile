set dotenv-load

# Run
run:
  cargo run --features development

# Setup development environment
setup:
    cog install-hook --all

# Hot Reload
watch:
  cargo watch -i "target/*" -i "*.log" -x check -x "run --features development"

# Hot Reload with Testing in the Loop
watch-test:
  cargo watch -i "target/*" -i "*.log" -x check -x "test --lib" -x "run --features development"

# Run tests
test:
  cargo test --color always --features testing

# Run unit tests
unit-test:
  cargo test --lib --color always --features testing

# Run tests (nextest)
nextest:
  cargo nextest run --features testing

# Run unit tests
unit-nextest:
  cargo nextest run --lib --features testing

# Watch tests
test-watch:
  cargo watch -i "target/*" -i "*.log" -x check -x "test --features testing"

# Watch unit tests
unit-test-watch:
  cargo watch -i "target/*" -i "*.log" -x check -x "test --lib --features testing"

# Watch tests
nextest-watch:
  cargo watch -i "target/*" -i "*.log" -x check -x "nextest run --features testing"

# Watch unit tests
unit-nextest-watch:
  cargo watch -i "target/*" -i "*.log" -x check -x "nextest run --lib --features testing"

# Run coverage
coverage:
  cargo +nightly llvm-cov

# Build release
build:
  cargo build --release

# Format + lint
check:
  cargo fmt --check
  cargo clippy

# Format + lint strict
check-strict:
  cargo fmt --check
  cargo clippy -- -D clippy::pedantic -D clippy::nursery

# Checking deps for vulnerabilities
audit:
  cargo audit

# Add migration
migrate name:
  sqlx migrate add {{name}}

# Reset database and reapply migrations
reset-db:
  sqlx database reset -y

# Drop database
drop-db:
  sqlx database drop -y
