//! Identify terminal applications and multiplexers from environment hints.
//!
//! [`detect()`] reads the current environment; [`detect_from_env`] accepts a snapshot for
//! repeatable detection. Both return [`TerminalInfo`] without running commands.
//! [`detect_with_options`] can opt into synchronous multiplexer probes. The library has no runtime
//! dependencies.
//!
//! # Usage
//!
//! ```
//! let info = detect_terminal::detect();
//! println!("Terminal: {}", info.kind);
//!
//! if let Some(multiplexer) = info.multiplexer {
//!     println!("Multiplexer: {}", multiplexer.kind);
//! }
//! ```
//!
//! # Interpret the result
//!
//! `TERM_PROGRAM` names an application; `TERM` names its terminfo description. Many applications
//! use `TERM=xterm-256color`, so an [`Xterm`](TerminalKind::Xterm) fallback identifies an emulation
//! family, not proof that the application is xterm. Prefer explicit program markers when present.
//! [`TerminalInfo::detected_via`] explains the chosen match, and raw values remain available even
//! when [`TerminalInfo::kind`] is [`Unknown`](TerminalKind::Unknown).
//!
//! [`TerminalInfo::version`] may be absent even when a terminal is recognized: it is populated
//! only when a recognized `TERM_PROGRAM` has a `TERM_PROGRAM_VERSION`. The raw version is retained
//! separately. [`TerminalInfo::multiplexer`] describes tmux, Screen, or Zellij independently of the
//! terminal; a recognized multiplexer can coexist with an unknown terminal.
//!
//! For an unfamiliar result, start with the selected kind and its evidence:
//!
//! ```
//! let info = detect_terminal::detect();
//! println!("Terminal: {}", info.kind);
//! for source in &info.detected_via {
//!     println!("Evidence: {source}");
//! }
//! if info.kind == detect_terminal::TerminalKind::Unknown {
//!     println!(
//!         "Unrecognized hints: {:?}, {:?}",
//!         info.term_program, info.term
//!     );
//! }
//! ```
//!
//! Environment hints may be inherited, overridden, or absent after SSH, sudo, or nested sessions.
//! This crate does not verify a live terminal, query escape sequences, detect whether stdout is a
//! TTY, or infer color and keyboard protocol support. Use `std::io::IsTerminal` for a TTY check.
//!
//! # Choose an entry point
//!
//! | Input and purpose | Function |
//! | --- | --- |
//! | Inspect the current process environment | [`detect()`] |
//! | Reuse or construct an environment snapshot | [`detect_from_env`] |
//! | Enable probes or disable diagnostic capture | [`detect_with_options`] |
//!
//! All three return a result even if no terminal is recognized. [`EnvMap`] shows how to collect
//! a snapshot; [`DetectOptions`] documents the defaults and available overrides.
//!
//! # Optional command probes
//!
//! Set [`DetectOptions::allow_commands`] to query `tmux -V`, tmux client term name and type,
//! `screen --version`, or `zellij --version` after a matching mux marker is found. Commands use the
//! supplied snapshot as their environment and do not invoke a shell. They block without a timeout;
//! leave them disabled when latency must be bounded. Multiple tmux clients may make the selected
//! client ambiguous. Missing tools, failed commands, and unrecognized banners leave metadata
//! absent.
//!
//! This example requires a matching multiplexer environment and its executable on `PATH`.
//!
//! ```no_run
//! use detect_terminal::{DetectOptions, detect_with_options};
//!
//! let env = std::env::vars_os().collect();
//! let options = DetectOptions {
//!     allow_commands: true,
//!     ..DetectOptions::default()
//! };
//! let info = detect_with_options(&env, options);
//! for probe in &info.command_probes {
//!     println!("{}: {:?}", probe.command, probe.status);
//! }
//! ```
//!
//! # Detection logic
//!
//! When a result is surprising, compare its evidence with this precedence order.
//!
//! Detection selects one multiplexer and one terminal:
//!
//! 1. Nonempty `TMUX`, `ZELLIJ`, then `STY` select a multiplexer in that order. Multiple markers
//!    cannot reliably establish nesting order; the result does not model a session stack.
//! 2. A recognized `TERM_PROGRAM` wins over vendor markers. Vendor markers are checked in a fixed
//!    order, beginning with `WT_SESSION`, WezTerm, Kitty, and Alacritty. Each successful
//!    environment match records the matching key and value. See [`TerminalKind`] for supported
//!    hints.
//! 3. With commands enabled, a recognized tmux client term name takes precedence over the pane's
//!    `TERM`. Each probe runs once and its full command string identifies any resulting match.
//! 4. `TERM` is a fallback. Matching uses complete family names followed by `-` or `.`, avoiding
//!    substring matches such as `stupid` or `not-xterm`. The `screen` family identifies Screen
//!    emulation only when neither tmux nor zellij was selected.
//!
//! Empty markers do not identify an application. Unrecognized `TERM_PROGRAM` values are retained
//! and allow later rules to match. `TERM_PROGRAM_VERSION` becomes [`TerminalInfo::version`] only
//! when `TERM_PROGRAM` itself identifies the result; otherwise it remains raw metadata in
//! [`TerminalInfo::term_program_version`]. `SESSIONNAME=Console` does not identify Windows Console
//! Host, and is not used for detection. `GHOSTTY_RESOURCES_DIR` is also ignored: resource paths
//! can survive into nested applications such as VS Code and tmux. If those applications replace
//! the program marker and no other recognized hint remains, the result is `Unknown`.
//!
//! This controlled snapshot demonstrates why a program marker wins over a multiplexer
//! emulation name. It does not describe the environment of every tmux session.
//!
//! ```
//! use detect_terminal::{EnvMap, MultiplexerKind, TerminalKind, detect_from_env};
//!
//! let env = EnvMap::from([
//!     ("TMUX".into(), "/tmp/tmux-1000/default,1234,0".into()),
//!     ("TERM_PROGRAM".into(), "ghostty".into()),
//!     ("TERM".into(), "screen-256color".into()),
//! ]);
//! let info = detect_from_env(&env);
//! assert_eq!(info.kind, TerminalKind::Ghostty);
//! assert_eq!(info.multiplexer.unwrap().kind, MultiplexerKind::Tmux);
//! assert!(info.command_probes.is_empty());
//! ```
//!
//! # Edge cases and likely mismatches
//!
//! A matched hint is evidence about the process environment, not proof of the application
//! currently drawing the terminal. The following cases are especially relevant on Windows and
//! Linux. See [`TerminalInfo::detected_via`] and [`TerminalInfo::identifiers`] to check which hint
//! won when several are present.
//!
//! ## Windows Terminal
//!
//! Its normal connection path supplies `WT_SESSION`. When Windows Terminal attaches as the
//! default host after the shell starts, that shell may receive no marker. With no other recognized
//! hint, detection returns [`Unknown`](TerminalKind::Unknown) or a `TERM` emulation family.
//!
//! ## Alacritty
//!
//! Without its terminfo entry, Alacritty uses `TERM=xterm-256color`. Its `ALACRITTY_SOCKET`
//! marker is Unix-only and can be absent. With only that shared `TERM`, detection returns
//! [`Xterm`](TerminalKind::Xterm), not Alacritty.
//!
//! ## foot
//!
//! A build without foot terminfo can also use `TERM=xterm-256color`. Without another marker, the
//! result is [`Xterm`](TerminalKind::Xterm), not foot.
//!
//! ## Tilix
//!
//! Tilix sets `TILIX_ID` but copies many parent variables. If it inherits a nonempty `WT_SESSION`
//! and no recognized `TERM_PROGRAM` is present, this detector selects Windows Terminal because
//! `WT_SESSION` precedes `TILIX_ID`. This is a possible precedence outcome, not a live observation.
//!
//! ## Cmder
//!
//! Cmder can run on ConEmu, supplying `CMDER_ROOT` alongside ConEmu markers. The Cmder marker wins
//! when both are present.
//!
//! ## ConEmu
//!
//! If `CMDER_ROOT` remains set in a later ConEmu session, detection selects Cmder before ConEmu's
//! own markers. This is a possible inherited-variable outcome, not a live observation.
//!
//! ## GNU Screen
//!
//! `TERM=screen*` describes emulation rather than proof of a Screen session. If no multiplexer
//! marker is present, this terminfo name alone can yield a misleading
//! [`Screen`](TerminalKind::Screen) result.
//!
//! ## tmux
//!
//! A nonempty `TMUX` marker prevents `TERM=screen*` from claiming Screen. With commands enabled,
//! the client term name is a terminfo name and may be ambiguous when several clients are attached.
//!
//! ## Zellij
//!
//! A nonempty `ZELLIJ` marker also prevents `TERM=screen*` from claiming Screen. If the marker is
//! absent, the shared terminfo name alone can still yield Screen.
//!
//! No combination of these environment hints can establish terminal capabilities or recover an
//! identity that was never passed to the process. See the repository's
//! [source-backed environment fixtures](https://github.com/joshka/detect-terminal/blob/main/docs/terminal-environments.md)
//! for upstream code and representative inputs.
//!
//! # Diagnostics and privacy
//!
//! [`TerminalInfo::raw_env_subset`] captures consulted variables by default. Disabling
//! [`DetectOptions::capture_env_subset`] removes that map, but matched [`Identifier`] values and
//! raw terminal names still appear in the result. Probe records include failures as well as output
//! from successful commands. Non-Unicode environment values and output are decoded lossily.
//! Review these fields before logging: session identifiers, paths, and command errors may contain
//! private information. Detection stops after a match, so the capture can omit existing variables
//! that were never consulted. Missing entries do not prove those variables were unset.
//! Use debug formatting or escape control characters when displaying raw values in a terminal;
//! the library preserves decoded input rather than sanitizing it for display.

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

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
