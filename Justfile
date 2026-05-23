set dotenv-load
# Run
run:
    cargo run

# Hot Reload
watch:
    cargo watch -x check -x run

# Hot Reload with Testing in the Loop
watch-test:
    cargo watch -x check -x test -x run

# Run tests
test:
    cargo test

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

# Checking deps for vulnerabilities
audit:
    cargo audit

migrate name:
  sqlx migrate add {{name}}

reset-db:
  sqlx database reset -y

drop-db:
  sqlx database drop -y
