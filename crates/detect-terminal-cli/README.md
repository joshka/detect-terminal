# detect-terminal CLI

Inspect terminal and multiplexer hints using the `detect-terminal` command. It reads the current
process environment and explains which hints selected the result. Rust 1.88 or newer is required to
build it.

## Installation

From crates.io:

```sh
cargo install detect-terminal-cli --locked
```

From the repository root:

```sh
cargo install --path crates/detect-terminal-cli --locked
```

The package is named `detect-terminal-cli`; the installed executable is `detect-terminal`.
Applications that need a Rust API should depend on the `detect-terminal` library.

## Inspect the Current Environment

```sh
detect-terminal
```

The human-readable report includes the terminal kind, available version and multiplexer metadata,
and detection evidence. An `Unknown` result means no supported hint matched. A known kind can name
an emulation family: `xterm` from `TERM=xterm-256color` does not prove the application is xterm.

Values are escaped for display so control characters cannot insert report lines or terminal escape
sequences. JSON preserves the decoded values. Non-Unicode input is decoded lossily in both formats.

| Option                      | Effect                                                    |
| --------------------------- | --------------------------------------------------------- |
| `--json` or `--format json` | Produce JSON instead of the default human-readable report |
| `--pretty`                  | Indent JSON; requires JSON output                         |
| `--no-env`                  | Omit only the diagnostic environment map                  |
| `--commands`                | Run optional multiplexer probes; disabled by default      |

Use either `--json` or `--format`, not both. `--no-env` leaves matched identifiers, raw names, and
command diagnostics visible. Review reports for session identifiers, paths, and command errors
before sharing them.

## Use JSON in Scripts

```sh
detect-terminal --json
detect-terminal --format json --pretty --no-env
```

Each invocation writes one JSON object followed by a newline. Check `schema_version` before
interpreting it, and tolerate additional fields and new kind names. Kind values are display names,
not Rust enum identifiers. For example, an iTerm2 result uses `"iTerm2"`, not `"ITerm2"`.

| Fields                                                     | Meaning                                                                                                                   |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `schema_version`                                           | Currently `1`                                                                                                             |
| `kind`, `version`                                          | Selected terminal or emulation name and optional version; unknown kind is `"Unknown"`                                     |
| `term_program`, `term_program_version`, `term`, `raw_name` | Raw hints, which may differ from the selected identity                                                                    |
| `multiplexer`                                              | `null` or an object with `kind`, `version`, `client_term`, and `client_type`; kind is `"tmux"`, `"screen"`, or `"zellij"` |
| `detected_via`, `identifiers`                              | Source descriptions and matching environment key/value pairs                                                              |
| `raw_env_subset`                                           | A map of existing variables consulted before detection finished                                                           |
| `command_probes`                                           | Attempted commands, their output, exit status, and execution errors                                                       |

Absent optional values are `null`; empty environment values remain empty strings. Diagnostic
collections use empty arrays or objects, not `null`. Disabling environment capture yields `{}` for
`raw_env_subset`; it does not remove the field. Disabled probes yield `[]` for `command_probes`.

## Request Multiplexer Metadata

```sh
detect-terminal --commands --json --pretty
```

A matching tmux, Screen, or Zellij environment marker must be present before its executable is
queried. Probes inherit the current environment, require the executable on `PATH`, and block without
a timeout. A failed probe does not make the CLI fail; inspect `command_probes` for its outcome:

- A non-null `error` indicates process creation or output collection failed.
- A nonzero `status` indicates a completed command failed. `status: null` can also represent signal
  termination; inspect `error` to distinguish an execution error.
- `status: 0` permits output to supply metadata, but empty or unrecognized output can still leave
  metadata absent. `stdout` and `stderr` are trimmed diagnostic strings.

## Investigate an Unexpected Result

1. Run `detect-terminal --json --pretty` in the session where the problem occurs. Compare `kind`
   with `detected_via` and `identifiers` to find the selected hint.
1. Compare the raw program and `TERM` values. Multiplexers, SSH, and nested applications can inherit
   or replace hints; a raw name can differ from the selected kind.
1. For tmux, optionally enable `--commands` to inspect client metadata and failed probes. Multiple
   attached clients can make the selected client ambiguous.
1. When reporting a mismatch, include the expected application, relevant versions, and a redacted
   report. Retain marker names and precedence relationships so the case can be reproduced.

The
[library's detection rules](https://docs.rs/detect-terminal/latest/detect_terminal/#detection-logic)
explain precedence. The diagnostic environment map contains only consulted keys; absence from it
does not prove a variable was absent from the process.

## Exit Status

| Status | Meaning                                                                                                |
| ------ | ------------------------------------------------------------------------------------------------------ |
| `0`    | Report written, including an unknown terminal or failed optional probes; also a closed downstream pipe |
| `1`    | Output failure other than a closed pipe                                                                |
| `2`    | Invalid command-line arguments                                                                         |

Licensed under MIT OR Apache-2.0.
