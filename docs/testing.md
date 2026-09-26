# Testing

Tests should establish what a caller can rely on. A case that only repeats a helper's implementation
is weaker than one that shows how competing terminal hints are resolved.

## Deterministic Coverage

Library tests construct `EnvMap` values rather than mutating process-wide environment variables. Use
`rstest` for compact marker tables and `pretty_assertions` for readable failures. Each new marker
needs a recognized case and relevant conflicts, empty values, or near-miss names. Keep evidence
assertions alongside kind assertions so a correct label cannot hide a fabricated source.

Inject `CommandRunner` for probe tests and count invocations. Cover successful output, nonzero
exits, signal-style missing status, missing executables, malformed banners, and disabled commands.
The production runner has a Unix test proving that supplied values are passed through and unrelated
parent variables are absent. Windows-specific lookup behavior is tested on Windows CI.

CLI integration tests execute the binary with `env_clear()` and explicit markers. Check JSON as
structured data, human output for meaningful content, argument failures, and exit status. Avoid
asserting every whitespace detail unless formatting itself is the contract.

## Select Checks by the Change

| Change                        | Relevant evidence                                                       |
| ----------------------------- | ----------------------------------------------------------------------- |
| Detection or command behavior | Regression tests, workspace tests, Clippy, affected docs                |
| Public API or examples        | Workspace tests, public and private Rustdoc                             |
| CLI                           | Binary integration tests, output error tests, help and README alignment |
| Markdown                      | Prose review and `just fmt-md-check`                                    |
| Dependency or MSRV            | Stable checks, `just msrv`, `just audit`, package verification          |
| CI                            | `just ci-check` locally and a successful hosted run                     |
| Release                       | Every gate in [Releasing](releasing.md)                                 |

Use `just check` before handing off an implementation change. Broaden validation when a failure or
remaining risk warrants it; repeatedly running a passing suite is not additional evidence.

## Platform Limits

CI is configured for native Linux, macOS, and Windows tests, with a separate Rust 1.88 job on Linux.
A successful local macOS run does not establish the other platforms. Record which jobs actually ran.
Environment fixtures establish the detector's rules; they do not establish how every terminal
version configures its environment. Live tmux, screen, and zellij sessions remain useful manual
checks when changing those integrations, especially client selection and inherited variables.
