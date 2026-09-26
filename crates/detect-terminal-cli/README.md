# detect-terminal CLI

Inspect terminal and multiplexer hints using the `detect-terminal` command. The default output is
human-readable; JSON is available for scripts. Rust 1.88 or newer is required to build it.

## Installation

After the first release is published:

```sh
cargo install detect-terminal-cli --locked
```

From a checkout, use `cargo install --path crates/detect-terminal-cli --locked`.

## Usage

```sh
detect-terminal
detect-terminal --json
detect-terminal --format json --pretty --no-env
detect-terminal --commands
```

`--commands` opts into blocking multiplexer probes without a timeout. By default the command reads
only the environment. `--no-env` omits the diagnostic environment map; matched identifiers and raw
names remain visible. Inspect output before sharing session identifiers, paths, or command errors.

JSON reports have `schema_version: 1`. Terminal and multiplexer `kind` values are display names,
unknown kinds use `"Unknown"`, missing scalar metadata is `null`, and diagnostic collections are
empty arrays or objects when absent. Consumers should tolerate additional fields and new display
names. A probe's `error` is non-null if process creation or output collection failed; nonzero
`status` records a completed command that failed. Neither supplies detection metadata.

An unknown terminal is a successful detection result, not an error exit. Invalid arguments exit with
status 2; output failures exit with status 1. A closed downstream pipe exits successfully.

The [library documentation](https://docs.rs/detect-terminal) explains marker precedence and the
limits of environment-based detection. Licensed under MIT OR Apache-2.0.
