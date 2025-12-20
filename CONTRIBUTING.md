# Contributing

## Overview

This repository contains a Rust library (`detect-terminal`) and a CLI
(`detect-terminal-cli`). The library is the source of truth for detection
behavior; the CLI is only a consumer.

## Scope

Create a Rust library for detecting terminals and multiplexers from environment
variables with optional command probing for tmux, screen, and zellij. Provide a
small, ergonomic public API with sensible defaults and debug-friendly output.
Document behavior clearly for first-time users and for testing/extension
workflows.

## Development Workflow

- Use your preferred version control tool and keep commit messages short and
  imperative (for example, `Add terminal markers for FooTerm`).
- Prefer small, buildable steps. Run `cargo check -p detect-terminal` after
  refactors or detection changes.
- Keep documentation aligned with behavior; update README and crate docs when
  detection rules change.
- When you change doc comments, run `cargo doc -p detect-terminal` to keep
  docs.rs output current.

## Running Commands

- `cargo check -p detect-terminal` validates the library build.
- `cargo test -p detect-terminal` runs unit tests and doc tests.
- `cargo doc -p detect-terminal` rebuilds docs.rs-style output.
- `cargo run -p detect-terminal-cli -- --format json --pretty` runs the CLI.

## Detection Changes

When adding a terminal or multiplexer:

1) Add env markers in `crates/detect-terminal/src/detect.rs`.
2) Document markers in `crates/detect-terminal/src/terminal.rs` or
   `crates/detect-terminal/src/multiplexer.rs`.
3) Add a test case in `crates/detect-terminal/src/detect.rs` using `rstest`.
4) Update `README.md` if the new item should be listed in the supported tables.
5) Keep the detection narrative and edge cases in the crate docs
   (`crates/detect-terminal/src/lib.rs`), not in private modules.

## Detection Strategy

The canonical detection strategy is documented in the crate-level docs under
`Detection Logic` in `crates/detect-terminal/src/lib.rs`.

## Testing Strategy

When extending detection, add tests that mirror real env combinations and keep
them deterministic.

- Use `rstest` cases in `crates/detect-terminal/src/detect.rs`.
- Provide inputs via `EnvMap` (`BTreeMap<OsString, OsString>`).
- Prefer `detect_with_options` when you need to control command probing or env
  capture explicitly.
- When testing command probes, inject a custom `CommandRunner` with a stable
  output map.
- Add parsing tests for tmux/screen/zellij outputs as needed.
- Run `cargo test -p detect-terminal` to exercise unit tests and doc tests.

## Documentation and Style

- Keep Markdown line length at 100 characters and lint with
  `markdownlint-cli2`.
- Use fenced code blocks with language tags.
- Prefer descriptive module names over generic buckets (avoid `types.rs`).
- Avoid multiline struct literals in function arguments; bind a local variable.
- Preserve `println!("{}", x.y)` style where field access is printed; otherwise
  prefer inline format strings.
- In `TerminalKind` docs, include the proper product name and website URL on the
  first line, then a blank line before details.
- Keep `TerminalKind` display names aligned with product naming
  (Terminal.app, iTerm2, Visual Studio Code Terminal).

## Documentation Rubric

Use this rubric to evaluate crate-level docs, README content, and public API
docs for `detect-terminal`.

### First-time User Success

- A reader can answer what the crate does, what it returns, and how to use it in
  two minutes.
- There is a minimal happy-path example and a targeted debugging example.

### Clear Hierarchy

- The structure follows purpose → usage → concepts → behavior → options →
  diagnostics → extension.
- Each section answers one question and does not mix unrelated concerns.

### Non-redundancy

- No section repeats content from another section.
- Each section adds unique, concrete guidance.

### Actionability

- Examples use realistic values and explain how to interpret outputs.
- Warnings include a clear mitigation or opt-out behavior.

### Edge-case Clarity

- At least two edge cases are documented and tied to real scenarios.

## Documentation Lenses (Ongoing Maintenance)

Use these lenses to keep docs practical and to surface gaps:

- **First-time user**: clarify purpose, supported coverage, and expected outputs.
- **Side effects**: call out command execution and how to disable it.
- **Data model**: explain program identity vs emulation identity (`TERM_PROGRAM`
  vs `TERM`).
- **Environment variability**: describe tmux/screen/zellij/SSH effects and
  fallbacks.
- **Testing/repro**: highlight deterministic paths (`detect_from_env`) and
  fixtures.
- **Extensibility**: explain where to add detection rules and tests.
- **Output consumption**: show how to use `TerminalInfo` fields and debug data.
- **Decision hierarchy**: outline precedence (mux first, program markers, TERM
  fallbacks) and notable edge cases.

## Documentation Plan

Moved from `PLAN.md` to keep guidance centralized.

- Provide a concise README with purpose, usage examples, and notes on command
  probing.
- Keep documentation structured and direct, without template-heavy sections.
- Add a docs.rs-friendly narrative with examples that explain realistic values.
- Keep supported terminals/multiplexers discoverable in docs and README.
- Include troubleshooting and extension guidance.
