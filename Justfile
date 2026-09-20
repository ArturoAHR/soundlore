set dotenv-load

# Run
run *args:
  cargo run --features development {{args}}

# Run with Hot Reload
hot-reload *args:
  cargo hot --features hot-reload {{args}}

# Setup development environment
setup:
    cog install-hook --all

# Run tests
test *args:
  cargo test --color always --features testing {{args}}

# Run unit tests
unit-test *args:
  cargo test --lib --color always --features testing {{args}}

# Run tests (nextest)
nextest *args:
  cargo nextest run --features testing {{args}}

# Run unit tests
unit-nextest *args:
  cargo nextest run --lib --features testing {{args}}

watch_flags := '-i "target/*" -i "*.log"'

# Hot Reload
watch *args:
  cargo watch {{watch_flags}} -x check -x "run --features development {{args}}"

# Hot Reload with Testing in the Loop
watch-test *args:
  cargo watch {{watch_flags}} -x check -x "test --lib" -x "run --features development {{args}}"

# Watch tests
test-watch *args:
  cargo watch {{watch_flags}} -x check -x "test --features testing {{args}}"

# Watch unit tests
unit-test-watch *args:
  cargo watch {{watch_flags}} -x check -x "test --lib --features testing {{args}}"

# Watch tests
nextest-watch *args:
  cargo watch {{watch_flags}} -x check -x "nextest run --features testing {{args}}"

# Watch unit tests
unit-nextest-watch *args:
  cargo watch {{watch_flags}} -x check -x "nextest run --lib --features testing {{args}}"

# Run coverage
coverage:
  cargo +nightly llvm-cov

# Build
build *args:
  cargo build {{args}}

build-release *args:
  cargo build --release --locked {{args}}

# Format + lint
check *args:
  cargo fmt --check
  cargo clippy {{args}}

# Checking deps for vulnerabilities
audit *args:
  cargo audit {{args}}

# Add migration
migrate name *args:
  sqlx migrate add {{name}} {{args}}

# Reset database and reapply migrations
reset-db:
  sqlx database reset -y

# Drop database
drop-db:
  sqlx database drop -y
