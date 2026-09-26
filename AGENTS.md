# Repository Guidelines

## Purpose and Scope

`detect-terminal` identifies terminal applications and multiplexers from environment hints, with
optional command probes. The library owns detection behavior; the CLI presents its result. Keep
application identity, emulation hints, and capability claims distinct.

## Guidance to Load

- For Rust changes and reviews, apply [Rust conventions](docs/rust-conventions.md).
- For any substantive prose, including comments, apply
  [Writing documentation](docs/documentation.md).
- For public APIs, defaults, examples, and links, also apply [Rustdoc contracts](docs/rustdoc.md).
- For behavior changes, choose evidence using [Testing](docs/testing.md).
- For release preparation, use the criteria in [Releasing](docs/releasing.md).

These guides apply during implementation; documentation is part of completing a behavior change.
When feedback establishes a reusable rule, update its owning guide and keep this file as the entry
point. Record a rule's reason, not the conversation that introduced it.

## Repository Map

- `crates/detect-terminal/src/lib.rs`: public exports and detection narrative.
- `detect.rs`: precedence and orchestration, plus deterministic detection tests.
- `env.rs`: environment snapshots and diagnostic capture.
- `command.rs`: optional execution, probe diagnostics, and version parsing.
- `terminal.rs`, `multiplexer.rs`, `options.rs`: public result types and options.
- `crates/detect-terminal-cli`: human/JSON output and binary integration tests.
- `README.md`: user entry point. `CONTRIBUTING.md`: contributor setup and commands.

## Checks and Change Shape

- Run `just fmt` to apply nightly rustfmt and rumdl formatting.
- Run `just check` for formatting, workspace tests, strict Clippy, and public/private Rustdoc.
- Run `cargo check -p detect-terminal` after a structural refactor to keep changes buildable.
- Use the focused recipes in `just --list` while iterating; run release gates as documented.
- Keep warnings actionable and fix their causes. Do not add broad lint suppressions.
- Use small, coherent changes with imperative summaries; preserve unrelated local work.
- Keep behavior changes, structural refactors, and dependency maintenance independently reviewable
  when they can be validated independently.
- Explain the user-visible result, relevant evidence, and remaining limitations when handing off.

## Formatting

Follow `rustfmt.toml` and `.config/rumdl.toml`. Prose wraps at 100 columns; Markdown tables may be
wider. Use `1.` for every ordered-list marker. Prefer surgical edits to these instructions unless
the requested task changes their structure.
