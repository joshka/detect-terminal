# List available recipes.
default:
    @just --list

# Run the checks expected before sharing a change.
check: fmt-check test clippy docs

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
    cargo test --workspace --all-features

# Reject Clippy warnings across the workspace.
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build API documentation and reject Rustdoc warnings.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
