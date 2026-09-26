use std::fmt;

/// A multiplexer inferred from a nonempty session environment marker.
///
/// This identifies the session manager independently of [`TerminalInfo::kind`]. `TERM=screen*`
/// alone does not establish a Screen session; it may describe emulation used by another
/// multiplexer. If markers conflict, only one kind is selected; see [Detection
/// logic](crate#detection-logic). [`fmt::Display`] returns `tmux`, `screen`, or `zellij`.
///
/// # Example
///
/// ```
/// let info = detect_terminal::detect();
/// if let Some(mux) = &info.multiplexer {
///     println!("Multiplexer: {}", mux.kind);
/// }
/// ```
///
/// [`TerminalInfo::kind`]: crate::TerminalInfo::kind
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultiplexerKind {
    /// GNU Screen, selected by a nonempty `STY`.
    Screen,
    /// tmux, selected by a nonempty `TMUX`.
    Tmux,
    /// Zellij, selected by a nonempty `ZELLIJ`.
    Zellij,
}

impl fmt::Display for MultiplexerKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            MultiplexerKind::Screen => "screen",
            MultiplexerKind::Tmux => "tmux",
            MultiplexerKind::Zellij => "zellij",
        })
    }
}

/// Multiplexer identity with optional command-derived version and client information.
///
/// Returned in [`TerminalInfo::multiplexer`] when a session marker is present. Environment-only
/// detection populates `kind`; every other field is `None`. Enable
/// [`DetectOptions::allow_commands`] through [`detect_with_options`] to request the extra metadata.
/// Missing executables, failed probes, and unavailable values still leave those fields absent.
/// Inspect [`TerminalInfo::command_probes`] to distinguish failed attempts from disabled probing.
///
/// The client fields apply only to tmux. They describe the client selected by tmux, which can be
/// ambiguous when several clients are attached. Screen and Zellij always leave these fields `None`.
/// See [Optional command probes](crate#optional-command-probes) for the blocking execution
/// contract.
///
/// # Example
///
/// Run inside a multiplexer session with its executable on `PATH`. Probes have no timeout.
///
/// ```no_run
/// use detect_terminal::{DetectOptions, detect_with_options};
///
/// let env = std::env::vars_os().collect();
/// let options = DetectOptions {
///     allow_commands: true,
///     ..DetectOptions::default()
/// };
/// let info = detect_with_options(&env, options);
/// if let Some(mux) = &info.multiplexer {
///     println!("Multiplexer: {}", mux.kind);
///     match &mux.version {
///         Some(version) => println!("Version: {version}"),
///         None => println!("Version unavailable; inspect command_probes for details"),
///     }
/// }
/// ```
///
/// [`TerminalInfo::multiplexer`]: crate::TerminalInfo::multiplexer
/// [`TerminalInfo::command_probes`]: crate::TerminalInfo::command_probes
/// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
/// [`detect_with_options`]: crate::detect_with_options
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplexerInfo {
    /// Multiplexer type detected from environment markers.
    pub kind: MultiplexerKind,
    /// Version extracted from optional command probes.
    ///
    /// This uses `tmux -V`, `screen --version`, or `zellij --version` when
    /// command probing is enabled.
    pub version: Option<String>,
    /// The tmux client's terminal name, usually a terminfo name such as `xterm-256color`.
    ///
    /// Read from `#{client_termname}` with command probing enabled. It describes the client side,
    /// which can differ from the pane's `TERM`. Recognized names can supply a terminal fallback.
    pub client_term: Option<String>,
    /// Additional terminal identification reported by tmux through `#{client_termtype}`.
    ///
    /// Retained as an opaque diagnostic string, not a terminfo name or capability list. It does
    /// not participate in terminal matching. Absent when probing is disabled or tmux cannot supply
    /// it.
    pub client_type: Option<String>,
}
