# detect-terminal

A Rust library for identifying terminal applications and multiplexers from environment hints. It
keeps application names (`TERM_PROGRAM`) separate from emulation names (`TERM`) and records why a
result was chosen. The library has no runtime dependencies.

Detection is a best effort: inherited variables can be stale, and generic `TERM` values identify an
emulation family rather than the actual application. It does not query escape sequences or measure
terminal capabilities.

## Status and Requirements

This checkout is the 0.1.0 release candidate. It requires Rust 1.97 or newer. Linux, macOS, and
Windows are the intended CI platforms; consult the
[release checklist](https://github.com/joshka/detect-terminal/blob/main/docs/releasing.md) for the
gates required before publishing.

## Library

```rust
use detect_terminal::{EnvMap, TerminalKind, detect_from_env};

let env = EnvMap::from([
    ("TERM_PROGRAM".into(), "ghostty".into()),
    ("TERM".into(), "xterm-ghostty".into()),
]);
let info = detect_from_env(&env);
assert_eq!(info.kind, TerminalKind::Ghostty);
```

Use `detect()` for the current environment. Both entry points are environment-only by default.
`detect_with_options()` can opt into blocking tmux, screen, and zellij commands; those commands have
no timeout. Unknown values remain available in the result.

See the [crate documentation](https://docs.rs/detect-terminal) for detection precedence, command
behavior, privacy considerations, and supported markers. Build it locally with `just docs`.

After the first release, add the library with `cargo add detect-terminal`. While working from this
checkout, use a path dependency on `crates/detect-terminal`.

## CLI

From this checkout:

```sh
cargo run -p detect-terminal-cli -- --format json --pretty
```

The installed binary is `detect-terminal`; install it from the checkout with
`cargo install --path crates/detect-terminal-cli --locked`.

The CLI prints human-readable output by default. Run with `--help` for command and diagnostic
options. Environment and probe details can contain session identifiers or paths; review output
before sharing it.

## Development

See [CONTRIBUTING.md](https://github.com/joshka/detect-terminal/blob/main/CONTRIBUTING.md) for setup
and checks. The project uses nightly rustfmt for formatting and rumdl for Markdown. Normal builds
and tests use stable Rust.

## License

Licensed under either [MIT](https://github.com/joshka/detect-terminal/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/joshka/detect-terminal/blob/main/LICENSE-APACHE), at your option.
