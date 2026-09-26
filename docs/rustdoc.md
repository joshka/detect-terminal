# Rustdoc Contracts

Apply [Writing documentation](documentation.md) to API prose. Rustdoc is a caller contract, so
examples, field descriptions, and defaults must agree with the code that ships.

## Teach the Model at the Crate Root

The crate introduction should explain `detect`, `detect_from_env`, `detect_with_options`, and the
relationship between terminal identity, emulation, and multiplexer metadata. Keep the precedence and
edge-case narrative there. Private modules should explain their local responsibility and link back
when readers need the broader model.

Keep one small example that produces an observable result. Show a synthetic `EnvMap` when the
example promises a particular terminal. Calling `detect()` on the reader's real environment is
appropriate for inspection, but cannot support an assertion that the result is Ghostty.

## Make Each Item Usable in Isolation

Start with what the item returns or represents. Document relevant input ownership, output
invariants, empty and unknown values, encoding, and side effects. Explain public fields and enum
variants, as well as private APIs whose purpose or ordering affects maintenance.

For this crate, specifically verify:

- Whether commands can run, which environment they receive, and whether they can block.
- Whether a version belongs to the selected terminal or is an uninterpreted raw value.
- What diagnostic capture omits and what identifiers remain elsewhere in the result.
- Whether failed probes, empty values, and absent variables remain distinguishable.
- Whether a terminal kind denotes an application marker or only a terminfo family.

For `TerminalKind`, put the product's proper name and website URL on the first line of each known
variant, then a blank line before its accepted markers. Display names should match product naming,
including Terminal.app, iTerm2, and Visual Studio Code Terminal.

Use `# Errors`, `# Panics`, or `# Safety` only when the item has those contracts. Describe the
actual trigger and effect. An infallible detection result can contain missing metadata; explain that
partial result without inventing an error return.

## Examples, Links, and Layout

Use compiling examples with useful assertions or output handling. Mark examples `no_run` when they
need external commands or resources, and explain the requirement. Do not hide an options override
inside a doctest that changes the behavior the reader sees. Group related fields into one example
when individual examples would repeat the same setup.

Keep links brief: `DetectOptions::allow_commands` is a better visible label than a fully qualified
implementation path. Use local reference links when a name is out of scope, and disambiguate
`detect()` from the private `detect` module. Add links when they provide a useful next step, not on
every repeated mention.

Leave a blank doc-comment line after headings and between paragraphs. Keep reference definitions
with the item they document. Prefer descriptive prose over comments that narrate ordinary Rust
assignments or control flow.

## Validation

For changed contracts, examples, or API shape, run `just test`, `just docs`, and
`just docs-private`. Private documentation catches links and ownership problems hidden by
public-only builds. Inspect rendered pages after changing navigation or layout. Keep the README, CLI
help, examples, and Rustdoc aligned when a behavior changes. The compiler checks syntax and links; a
prose review must still check the truth of the explanation.
