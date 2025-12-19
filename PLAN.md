# Plan

## Scope

Create a Rust library for detecting terminals and multiplexers from environment
variables with optional command probing for tmux, screen, and zellij. Provide a
small, ergonomic public API with sensible defaults and debug-friendly output.

## Public API

- `detect()` uses current process environment with default options.
- `detect_from_env(&EnvMap)` uses a provided environment map.
- `detect_with_options(&EnvMap, DetectOptions)` uses explicit options.

## Detection strategy

- Read a known set of environment variables and map them to terminal kinds.
- Detect multiplexers (tmux/screen/zellij) and, when allowed, run commands to
  extract versions and tmux client terminal data.
- Prefer specific terminal markers first, fall back to TERM/TERM_PROGRAM heuristics.
- Store matched identifiers, detection sources, and a raw subset of the
  environment for debugging.

## Test strategy

- Use `rstest` with an injected command runner to simulate command output.
- Cover terminal detection by marker, multiplexer detection with and without
  commands, and raw environment capture behavior.
- Add targeted parsing tests for tmux, screen, and zellij version extraction.

## Documentation

- Provide a concise README with purpose, usage examples, and notes on command probing.
- Keep documentation structured and direct, without template-heavy sections.
