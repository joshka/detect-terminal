# Repository Guidelines

## Project Structure & Module Organization

- `crates/detect-terminal/src/lib.rs` is the public entry point and re-exports
  the library API.
- Detection logic lives in `crates/detect-terminal/src/detect.rs`; environment
  helpers in `crates/detect-terminal/src/env.rs`.
- Domain types are grouped by purpose in `crates/detect-terminal/src/terminal.rs`,
  `crates/detect-terminal/src/multiplexer.rs`, `crates/detect-terminal/src/options.rs`,
  and command probing in `crates/detect-terminal/src/command.rs`.
- The CLI is `crates/detect-terminal-cli/src/main.rs`.
- Docs live in `README.md` and `CONTRIBUTING.md`.
- Tests are unit tests in `crates/detect-terminal/src/detect.rs`; doc tests are
  embedded in docs.

## Build, Test, and Development Commands

- `cargo check -p detect-terminal` validates the library build quickly without
  running tests.
- `cargo test -p detect-terminal` runs unit tests and doc tests.
- `cargo fmt` formats Rust code.
- `cargo clippy -p detect-terminal --all-targets --all-features` runs lint checks.
- `cargo doc -p detect-terminal` builds API documentation.
- `cargo outdated` checks dependency versions.
- `markdownlint-cli2 "**/*.md"` lints Markdown files.
- `cargo run -p detect-terminal-cli -- --format json --pretty` runs the CLI.
  In a workspace, use `-p` to target a specific crate.

## Coding Style & Naming Conventions

- Follow standard Rust formatting via `cargo fmt`.
- Prefer small, buildable changes; run `cargo check -p detect-terminal` after
  refactors.
- Keep Markdown line length at 100 characters and use fenced code blocks.
- Prefer descriptive module names over generic buckets (no `types.rs`).
- Use direct, explicit detection markers in docs (env vars and commands).

## Testing Guidelines

- Use `rstest` and `pretty_assertions` for unit tests in
  `crates/detect-terminal/src/detect.rs`.
- Add cases for new terminals or multiplexers and keep tests deterministic.
- Doc examples are expected to compile as doc tests.

## Commit & Pull Request Guidelines

- Use your preferred version control tool and keep commit messages short and
  imperative.
- Only one commit exists (`Add terminal detection library`), so there is no
  established convention yet; use short, imperative summaries.
- No PR template is defined; include a concise change description and test
  results when proposing changes.

## Agent-Specific Instructions

- Keep refactors buildable; prefer small, atomic edits and run `cargo check`.
- Document public and private APIs, including detection behavior and defaults.
- Run `cargo doc` whenever doc comments change.
- Preserve `println!("{}", x.y)` style where field access is printed; otherwise
  prefer inline format strings.
- Avoid multiline struct literals in function arguments; bind a local variable.
- Keep docs and tests aligned with behavior changes (update README and doc tests
  when detection logic changes).
- In `TerminalKind` docs, include the proper product name and website URL on the
  first line, then a blank line before details.
- Use `TerminalKind` display names that match the actual product naming
  (Terminal.app, iTerm2, Visual Studio Code Terminal).
- Keep detection strategy and edge-case narrative in the crate docs, not in
  private modules.
- Refer to `CONTRIBUTING.md` for project scope and documentation expectations.
