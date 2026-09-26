# Changelog

## Unreleased

Initial release:

- Identify terminal applications from program markers and retain raw program and terminfo values.
- Ignore inherited `GHOSTTY_RESOURCES_DIR` when identifying the terminal.
- Recognize Rio through `TERM_PROGRAM=rio` or the `rio` terminfo family.
- Report tmux, GNU Screen, and Zellij separately from terminal identity.
- Use environment-only detection by default, with optional blocking multiplexer probes.
- Explain matches with environment identifiers and command diagnostics, including failed attempts.
- Provide the `detect-terminal` CLI with human-readable output and versioned JSON reports.
- Support Rust 1.88 and newer, under MIT OR Apache-2.0.
