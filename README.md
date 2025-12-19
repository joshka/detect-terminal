# detect-terminal

Detect the current terminal and any active multiplexer by inspecting environment
variables and optionally running mux commands for extra detail.

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

## Notes

- `detect()` and `detect_from_env()` enable command probing by default.
- Multiplexer detection currently supports tmux, screen, and zellij.
- Multiplexer detection checks env markers first, then optionally runs
  `tmux -V`, `tmux display-message -p`, `screen --version`, and
  `zellij --version`.
- `TerminalInfo` includes `term_program`, `term_program_version`, and `term` so
  you can distinguish the terminal program from the emulation identity (for
  example, Ghostty can report `TERM=xterm-ghostty`).
- `TERM=screen*` is treated as GNU Screen only when tmux is not detected, to
  avoid mislabeling tmux sessions.
- `TerminalInfo` includes a `raw_env_subset` so you can see exactly which
  variables were used.

## CLI

```bash
cargo run -p detect-terminal-cli
cargo run -p detect-terminal-cli -- --format json --pretty
cargo run -p detect-terminal-cli -- --no-commands --no-env
```
