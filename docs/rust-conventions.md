# Rust Conventions

Keep the detection pipeline readable from top to bottom: collect raw hints, select a multiplexer,
optionally probe it, choose terminal evidence, then return the result. Explicit precedence is more
valuable here than a generic rule engine.

## Code and API Shape

- Group code by the concept it owns. Keep detection ordering in `detect.rs`, process behavior in
  `command.rs`, and environment access in `env.rs`; avoid generic `utils.rs` or `types.rs` buckets.
- Extract helpers when their names reduce what a reader must remember. Avoid wrapper structs that
  only duplicate fields before immediately copying them into a result.
- Keep side effects visible. Use named commands and explicit execution; reuse gathered metadata
  rather than repeating a probe in a fallback path.
- Public enums that may gain supported products are non-exhaustive. Document unknown and partial
  results so callers can degrade gracefully.
- Preserve the library's dependency-free runtime unless a dependency solves a demonstrated need. Use
  the widest honest compatible requirements, and refresh the lockfile without raising minimums
  merely to match the latest release. Review incompatible dependency changes separately.
- Document private APIs where their responsibilities and constraints are not obvious. Put the
  cross-module behavior and edge cases in crate Rustdoc.

## Local Style

- Keep field-access formatting as `println!("{}", info.kind)`; inline simple variable names.
- Bind multiline struct literals before passing them as function arguments.
- When a method receiver spans multiple lines, bind it before starting a method chain. Do not put
  chained calls at the first column after the receiver.
- Use pure iterators for transformations and explicit statements for command execution or mutation.
- Prefer existing standard-library types and small functions over extra indirection or
  configuration.
- Do not add speculative optimization. Measure a specific cost before trading away readability.

## Changing Detection

Find a documented product marker or a recorded observation before adding a heuristic. An environment
variable can be inherited by nested applications; describe the uncertainty and add precedence tests
when it can conflict with existing markers. Keep match evidence consistent with the actual source,
including command-derived matches. Update the variant reference and user-facing behavior docs with
the implementation.

Use [Testing](testing.md) for evidence and [Rustdoc contracts](rustdoc.md) for API documentation.
