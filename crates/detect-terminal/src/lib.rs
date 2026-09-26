//! Detect terminal emulators and multiplexers from environment variables with
//! optional command probes.
//!
//! The detector returns a [`TerminalInfo`] with a primary terminal kind, optional
//! multiplexer metadata, and debug-friendly details
//! ([`TerminalInfo::identifiers`], [`TerminalInfo::detected_via`], and
//! [`TerminalInfo::command_probes`]). Supported terminals are listed in
//! [`TerminalKind`]; multiplexers are listed in [`MultiplexerKind`]. The list is
//! not exhaustive; rely on [`TerminalInfo::term_program`] and
//! [`TerminalInfo::term`] when a terminal is [`TerminalKind::Unknown`]. The
//! `detect-terminal-cli` crate provides a CLI for quick inspection.
//!
//! # Usage
//!
//! ```rust
//! use detect_terminal::{
//!     DetectOptions, TerminalKind, detect, detect_from_env, detect_with_options,
//! };
//!
//! let info = detect();
//! if info.kind == TerminalKind::ITerm2 {
//!     println!("iTerm2 detected");
//! }
//!
//! let env = std::env::vars_os().collect();
//! let _info = detect_from_env(&env);
//!
//! let options = DetectOptions {
//!     allow_commands: true,
//!     capture_env_subset: true,
//! };
//! let info = detect_with_options(&env, options);
//! println!("terminal: {kind}", kind = info.kind);
//! ```
//!
//! # Concepts
//!
//! ## Program vs Emulation
//!
//! Use this when you need to distinguish the application from the emulation
//! contract it presents to programs. For example, Ghostty can report
//! [`TerminalInfo::term_program`] as `ghostty` while advertising
//! [`TerminalInfo::term`] as `xterm-ghostty`.
//!
//! ```rust
//! use detect_terminal::detect;
//!
//! let info = detect();
//! println!("program: {program:?}", program = info.term_program);
//! println!("emulation: {emulation:?}", emulation = info.term);
//! ```
//!
//! ## Multiplexer Layer
//!
//! Multiplexers are detected separately from the terminal emulator. A tmux
//! session reports a mux kind of `tmux`, while the underlying terminal is still
//! identified by [`TerminalInfo::kind`].
//!
//! ```rust
//! use detect_terminal::detect;
//!
//! let info = detect();
//! if let Some(mux) = info.multiplexer {
//!     println!("mux: {kind}", kind = mux.kind);
//!     println!("mux version: {version:?}", version = mux.version);
//! }
//! ```
//!
//! # Detection Logic
//!
//! The detector runs in two phases:
//!
//! - Multiplexer detection first: looks for `TMUX`, `ZELLIJ`, or `STY`. When command probing is
//!   enabled, it shells out to gather version metadata and tmux client term details.
//! - Terminal detection second: prefers explicit program markers (for example,
//!   `TERM_PROGRAM=WezTerm`, `WT_SESSION`) before falling back to `TERM` heuristics (for example,
//!   `TERM=xterm-ghostty`).
//!
//! The multiplexer result is stored separately in [`TerminalInfo::multiplexer`]
//! and does not override the terminal emulator. When command probing is
//! enabled, tmux client term data is captured in
//! [`MultiplexerInfo::client_term`] and [`MultiplexerInfo::client_type`].
//!
//! `TERM_PROGRAM`, `TERM_PROGRAM_VERSION`, and `TERM` are recorded whenever
//! present, even when the terminal kind is [`TerminalKind::Unknown`]. These
//! values let you distinguish program identity (for example, `ghostty`) from
//! emulation identity (for example, `xterm-ghostty`).
//!
//! ## Examples
//!
//! Example: tmux inside Ghostty. `TERM_PROGRAM` still identifies Ghostty while
//! `TERM` reflects tmux's emulation choice, and mux metadata is reported
//! separately.
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use std::ffi::OsString;
//!
//! use detect_terminal::detect_from_env;
//!
//! let mut env = BTreeMap::new();
//! let tmux_path = "/tmp/tmux-1000/default,1234,0";
//! env.insert(OsString::from("TMUX"), OsString::from(tmux_path));
//! env.insert(OsString::from("TERM_PROGRAM"), OsString::from("ghostty"));
//! env.insert(OsString::from("TERM"), OsString::from("screen-256color"));
//!
//! let info = detect_from_env(&env);
//! assert_eq!(info.multiplexer.unwrap().kind.to_string(), "tmux");
//! assert_eq!(info.term_program.as_deref(), Some("ghostty"));
//! assert_eq!(info.term.as_deref(), Some("screen-256color"));
//! ```
//!
//! Example: no program markers. The detector falls back to `TERM` heuristics.
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use std::ffi::OsString;
//!
//! use detect_terminal::detect_from_env;
//!
//! let mut env = BTreeMap::new();
//! env.insert(OsString::from("TERM"), OsString::from("xterm-kitty"));
//!
//! let info = detect_from_env(&env);
//! assert_eq!(info.kind.to_string(), "Kitty");
//! ```
//!
//! # Options
//!
//! Use [`DetectOptions`] to control command probes and environment capture.
//! `detect()` and `detect_from_env()` use default options with probes enabled.
//!
//! ```rust
//! use detect_terminal::{DetectOptions, detect_with_options};
//!
//! let env = std::env::vars_os().collect();
//! let options = DetectOptions {
//!     allow_commands: false,
//!     capture_env_subset: false,
//! };
//! let info = detect_with_options(&env, options);
//! println!("terminal: {kind}", kind = info.kind);
//! ```
//!
//! # Diagnostics
//!
//! Use these fields when you need to explain or debug the result:
//!
//! - [`TerminalInfo::identifiers`] includes the env markers that matched.
//! - [`TerminalInfo::detected_via`] indicates whether env keys or commands triggered detection.
//! - [`TerminalInfo::raw_env_subset`] records only the env variables read.
//! - [`TerminalInfo::command_probes`] stores command outputs when probes run.
//!
//! ```rust
//! use detect_terminal::detect;
//!
//! let info = detect();
//! for identifier in &info.identifiers {
//!     println!(
//!         "{key}={value}",
//!         key = identifier.key,
//!         value = identifier.value
//!     );
//! }
//! ```
//!
//! # Edge Cases
//!
//! - `TERM=screen*` is treated as GNU Screen only when a tmux marker is not present, avoiding tmux
//!   misclassification.
//! - `TERM_PROGRAM` and `TERM` can both be set. Prefer [`TerminalInfo::term_program`] for the
//!   application name and [`TerminalInfo::term`] for emulation details.
//!
//! # Privacy and Performance
//!
//! - Avoid logging [`TerminalInfo::command_probes`], [`TerminalInfo::identifiers`], or
//!   [`TerminalInfo::raw_env_subset`] in telemetry unless you scrub sensitive values.
//! - Command probes add latency proportional to `tmux`, `screen`, or `zellij` invocation; disable
//!   them in hot paths if needed.
//!
//! # CLI JSON Output
//!
//! The `detect-terminal-cli` crate can emit JSON that mirrors [`TerminalInfo`].
//! This is useful for diagnostics or piping into other tools.
//!
//! # Extending Detection
//!
//! - Add new terminal markers in `detect.rs` and document them in [`TerminalKind`].
//! - Add a test case in `detect.rs` with `rstest`.
//! - Prefer explicit program markers before `TERM` heuristics.
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
