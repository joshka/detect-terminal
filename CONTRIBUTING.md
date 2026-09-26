# Contributing

The library owns detection rules and diagnostics. The CLI consumes that API and formats results.
Keep environment hints, optional command metadata, and claims about actual terminal capabilities
separate when proposing a change.

## Setup

Rust 1.97 is the minimum supported version. It was the previous stable release when this baseline
was chosen. Raise the MSRV only when a language feature or dependency needs it, targeting the
previous stable release at that time; do not advance it automatically for every crate release. Use
stable Rust for development and nightly rustfmt for the unstable options in `rustfmt.toml`.

```sh
rustup toolchain install stable --component clippy
rustup toolchain install nightly --component rustfmt
rustup toolchain install 1.97.0
cargo install just --version 1.58.0 --locked
cargo install rumdl --version 0.2.77 --locked
```

The versioned utility installs match CI. `cargo-audit` 0.22.2 and `actionlint` are needed for
advisory and workflow checks. Update utility versions in CI and this guide together.

## Development Checks

```sh
just fmt
just check
```

`just check` validates Rust and Markdown formatting, runs workspace tests and doctests, rejects
Clippy warnings, and builds public and private Rustdoc with warnings treated as errors. Use
`just --list` for focused recipes. `cargo check -p detect-terminal` provides a quick library check.

Run the CLI with `cargo run -p detect-terminal-cli -- --json --pretty`. It uses environment-only
detection unless `--commands` is supplied.

## Changing Behavior

1. Establish the marker or command behavior from a product reference or recorded observation.
1. State the expected result and how it interacts with existing precedence.
1. Add deterministic regression cases, including relevant failure or ambiguity cases.
1. Implement the smallest coherent change and update the owning API docs and CLI contract.
1. Run the checks appropriate to the change and describe remaining platform or integration gaps.

Follow [Rust conventions](docs/rust-conventions.md), [Writing documentation](docs/documentation.md),
and [Rustdoc contracts](docs/rustdoc.md). Use [Testing](docs/testing.md) to choose evidence and
[Releasing](docs/releasing.md) for release gates. These guides apply to ordinary code changes as
well as dedicated documentation work.

## Dependencies and Releases

Dependabot groups weekly Cargo lockfile updates and monthly GitHub Actions updates. Changes that
require wider or higher manifest requirements are reviewed separately. A lockfile refresh must not
silently raise the supported Rust version or change the public API's dependency integration.

Keep fixes and maintenance in reviewable `jj` changes. Describe the problem and resulting behavior
with an imperative summary. Include validation and limitations when requesting review. The project
is licensed under MIT OR Apache-2.0; contributions must be compatible with that choice.
