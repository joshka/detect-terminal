# detect-terminal

Detect the current terminal and any active multiplexer by inspecting environment variables and
optionally running mux commands for extra detail. This library separates the terminal program
identity (`TERM_PROGRAM`) from the emulation identity (`TERM`) so you can reason about what the
terminal is and how it wants to be treated, even under tmux/screen/zellij or SSH.

Supported terminals are documented in `TerminalKind`; multiplexers are listed in `MultiplexerKind`.
The list is not exhaustive; use `TerminalKind::Unknown` plus `term_program`/`term` for fallbacks.
The CLI (`detect-terminal-cli`) is an optional helper for debugging and does not affect the library
API.

## Supported Terminals

This is a quick reference. The full detection markers live in `TerminalKind`. Identity reflects
whether we typically see program markers (such as `TERM_PROGRAM` or vendor-specific env vars) or
only emulation markers (`TERM`) in the environment.

| Terminal         | Primary marker              | Identity         |
| ---------------- | --------------------------- | ---------------- |
| iTerm2           | TERM_PROGRAM=iTerm.app      | program and term |
| Terminal.app     | TERM_PROGRAM=Apple_Terminal | program and term |
| Ghostty          | TERM_PROGRAM=ghostty        | program and term |
| WezTerm          | TERM_PROGRAM=WezTerm        | program and term |
| Kitty            | KITTY_WINDOW_ID             | program or term  |
| Alacritty        | ALACRITTY_SOCKET            | program or term  |
| GNOME Terminal   | GNOME_TERMINAL_SERVICE      | program or term  |
| Konsole          | KONSOLE_VERSION             | program or term  |
| Windows Terminal | WT_SESSION                  | program and term |
| ConHost          | SESSIONNAME=Console         | program or term  |

## Supported Multiplexers

| Multiplexer | Marker | Notes                                          |
| ----------- | ------ | ---------------------------------------------- |
| tmux        | TMUX   | optional `tmux -V` + `tmux display-message -p` |
| screen      | STY    | optional `screen --version`                    |
| zellij      | ZELLIJ | optional `zellij --version`                    |

## Usage

```rust
use detect_terminal::{detect, detect_from_env, DetectOptions, TerminalKind};

let info = detect();

if info.kind == TerminalKind::ITerm2 {
    println!("iTerm2 detected");
}

let env = std::env::vars_os().collect();
let info = detect_from_env(&env);

let options = DetectOptions {
    allow_commands: true,
    capture_env_subset: true,
};
let info = detect_terminal::detect_with_options(&env, options);
```

If you need to avoid external command execution, set `allow_commands` to `false`.

### Example Output (Human)

```text
terminal: iTerm2
term program: iTerm.app
term program version: 3.4.22
term: xterm-256color
```

### Example Output (JSON)

This is the CLI JSON output for the same iTerm2-style environment.

```json
{
  "kind": "iTerm2",
  "version": "3.4.22",
  "term_program": "iTerm.app",
  "term_program_version": "3.4.22",
  "term": "xterm-256color",
  "raw_name": "iTerm.app",
  "multiplexer": null,
  "detected_via": [
    "env: TERM_PROGRAM"
  ],
  "identifiers": [
    {
      "key": "TERM_PROGRAM",
      "value": "iTerm.app"
    }
  ],
  "raw_env_subset": {
    "TERM_PROGRAM": "iTerm.app",
    "TERM_PROGRAM_VERSION": "3.4.22",
    "TERM": "xterm-256color"
  },
  "command_probes": []
}
```

### Program vs Emulation

Use this when you need to distinguish the application from the emulation contract it presents to
programs. For example, Ghostty can report `term_program=ghostty` while advertising
`term=xterm-ghostty` in `term`.

```rust
use detect_terminal::detect;

let info = detect();
println!("program: {:?}", info.term_program);
println!("emulation: {:?}", info.term);
```

### Custom Environments (Testing)

This is helpful for unit tests or reproducing a specific terminal signature. The example mirrors a
typical iTerm2 setup with `TERM_PROGRAM=iTerm.app` and a common `TERM` value.

```rust
use detect_terminal::detect_from_env;
use std::collections::BTreeMap;
use std::ffi::OsString;

let mut env = BTreeMap::new();
env.insert(OsString::from("TERM_PROGRAM"), OsString::from("iTerm.app"));
env.insert(OsString::from("TERM_PROGRAM_VERSION"), OsString::from("3.4.22"));
env.insert(OsString::from("TERM"), OsString::from("xterm-256color"));

let info = detect_from_env(&env);
println!("terminal: {}", info.kind);
```

### Tuning Detection Options

Disable command probes or env capture to control side effects and output. The example shows a purely
env-driven run without the extra metadata from tmux or zellij commands.

```rust
use detect_terminal::{detect_with_options, DetectOptions};

let env = std::env::vars_os().collect();
let options = DetectOptions {
    allow_commands: false,
    capture_env_subset: false,
};
let info = detect_with_options(&env, options);
println!("terminal: {}", info.kind);
```

### Interpreting TerminalInfo

Use these fields together to understand both the program identity and the behavior it claims to
emulate. `term_program_version` is usually a semantic version string such as `3.4.22`.

- `kind` is the terminal app when known.
- `term_program`/`term_program_version` identify the program.
- `term` identifies the emulation contract (e.g., `xterm-ghostty`).
- `multiplexer` describes tmux/screen/zellij if present.
- `identifiers` and `detected_via` show exactly which markers matched.
- `raw_name` is a best-effort label when a specific terminal kind is unknown.

### Debugging a Misclassification

```rust
use detect_terminal::detect;

let info = detect();
for source in info.detected_via {
    println!("{}", source);
}
```

### Multiplexer Details

```rust
use detect_terminal::detect;

let info = detect();
if let Some(mux) = info.multiplexer {
    println!("mux: {}", mux.kind);
    println!("mux version: {:?}", mux.version);
}
```

## Notes

- `detect()` and `detect_from_env()` enable command probing by default.
- Multiplexer detection currently supports tmux, screen, and zellij.
- Multiplexer detection checks env markers first, then optionally runs `tmux -V`,
  `tmux display-message -p`, `screen --version`, and `zellij --version`. This shells out to external
  commands unless `allow_commands` is disabled.
- `detect_from_env()` is the deterministic path for tests and fixtures.
- `TerminalInfo` includes `term_program`, `term_program_version`, and `term` so you can distinguish
  the terminal program from the emulation identity (for example, Ghostty can report
  `TERM=xterm-ghostty`).
- `TERM=screen*` is treated as GNU Screen only when tmux is not detected, to avoid mislabeling tmux
  sessions.
- `TerminalInfo` includes a `raw_env_subset` so you can see exactly which variables were used. It
  may include sensitive values; handle with care.
- Avoid logging `command_probes`, `identifiers`, or `raw_env_subset` in telemetry unless you scrub
  sensitive values.
- Detection is environment-driven and works over SSH/WSL as long as env vars are forwarded.
- Command probes add latency proportional to `tmux`, `screen`, or `zellij` invocation; disable them
  in hot paths if needed.

## Troubleshooting

- If you see `TerminalKind::Unknown`, check `term_program`, `term`, and `identifiers` to see what
  was available in the environment.
- If mux metadata is missing, ensure `allow_commands` is true and the mux binaries are on `PATH`.
- If a terminal is misidentified, capture the env with `raw_env_subset` and use `detect_from_env()`
  to reproduce the result.

## Extending Detection

- Add new terminal markers in `crates/detect-terminal/src/detect.rs` and list them in `TerminalKind`
  docs.
- Add a test case in `crates/detect-terminal/src/detect.rs` using `rstest`.
- Prefer explicit program markers (`TERM_PROGRAM`, vendor env vars) before `TERM` heuristics.

## CLI

```bash
cargo run -p detect-terminal-cli
cargo run -p detect-terminal-cli -- --format json --pretty
cargo run -p detect-terminal-cli -- --no-commands --no-env
```

### CLI Flags

- `--format human|json` selects output mode.
- `--json` forces JSON output (same as `--format json`).
- `--pretty` pretty-prints JSON output.
- `--no-commands` disables external command probes.
- `--no-env` disables `raw_env_subset` capture.
- CLI JSON output mirrors `TerminalInfo`; update it if fields change.
