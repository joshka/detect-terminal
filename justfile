# List available recipes.
default:
    @just --list

# Run the checks expected before sharing a change.
check: fmt-check test clippy docs docs-private docs-rs

# Format Rust and Markdown.
fmt: fmt-rust fmt-md

# Check Rust and Markdown formatting.
fmt-check: fmt-rust-check fmt-md-check

# Format Rust with the unstable options in rustfmt.toml.
fmt-rust:
    cargo +nightly fmt --all

# Check Rust formatting without changing files.
fmt-rust-check:
    cargo +nightly fmt --all -- --check

# Reflow prose and align Markdown tables.
fmt-md:
    rumdl fmt .

# Check Markdown formatting without changing files.
fmt-md-check:
    rumdl check .

# Run workspace unit tests and doctests.
test:
    cargo test --workspace --all-features --locked

# Reject Clippy warnings across the workspace.
clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Build API documentation and reject Rustdoc warnings.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# Validate links and contracts in private API documentation too.
docs-private:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --document-private-items

# Reproduce the docs.rs build for each published package with nightly Rustdoc.
docs-rs:
    cargo +nightly docs-rs --package detect-terminal --locked
    cargo +nightly docs-rs --package detect-terminal-cli --locked

# Run all tests on the minimum supported compiler.
msrv:
    cargo +1.88.0 test --workspace --all-features --locked

# Assemble and compile both release archives from a clean checkout.
package:
    cargo package --workspace --locked

# Validate GitHub Actions syntax and embedded shell scripts.
ci-check:
    actionlint

# Audit workflows and Dependabot configuration, including pedantic findings.
zizmor:
    zizmor --no-progress --strict-collection --persona pedantic .github

# Check advisories, licenses, duplicate versions, and dependency sources.
deny:
    cargo deny --workspace --all-features --locked check --deny warnings
