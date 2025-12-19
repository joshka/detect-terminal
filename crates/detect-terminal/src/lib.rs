//! Detect terminal emulators and multiplexers from environment variables and
//! optional command probes.
//!
//! # Usage
//!
//! ```rust
//! use detect_terminal::{detect, detect_from_env, DetectOptions, TerminalKind};
//!
//! let info = detect();
//! if info.kind == TerminalKind::ITerm2 {
//!     println!("iTerm2 detected");
//! }
//!
//! let env = std::env::vars_os().collect();
//! let info = detect_from_env(&env);
//!
//! let options = DetectOptions {
//!     allow_commands: true,
//!     capture_env_subset: true,
//! };
//! let info = detect_terminal::detect_with_options(&env, options);
//! ```
//!
//! # Notes
//!
//! - [`detect`] and [`detect_from_env`] enable command probing by default.
//! - Multiplexer detection currently supports tmux, screen, and zellij.
//! - Multiplexer detection first checks environment markers, then optionally
//!   runs `tmux -V`, `tmux display-message -p`, `screen --version`, and
//!   `zellij --version` for richer metadata.
//! - [`TerminalInfo`] includes `term_program`, `term_program_version`, and
//!   `term` so you can distinguish the program identity from the emulation
//!   identity (for example, Ghostty can report `TERM=xterm-ghostty`).
//! - `TERM=screen*` is treated as GNU Screen only when a tmux session is not
//!   detected, to avoid mislabeling tmux sessions as Screen.
//! - [`TerminalInfo`] includes a `raw_env_subset` for auditing which variables
//!   were read.
mod command;
mod detect;
mod env;
mod multiplexer;
mod options;
mod terminal;

pub use crate::detect::{detect, detect_from_env, detect_with_options};
pub use crate::env::EnvMap;
pub use crate::multiplexer::{MultiplexerInfo, MultiplexerKind};
pub use crate::options::DetectOptions;
pub use crate::terminal::{CommandProbe, DetectionSource, Identifier, TerminalInfo, TerminalKind};
