use std::fmt;

/// Multiplexer kinds detected from environment markers.
///
/// This is independent of the terminal emulator; it indicates a session is
/// running under tmux/screen/zellij rather than naming the underlying terminal
/// app. The detector checks env markers first and optionally runs `tmux -V`,
/// `tmux display-message -p`, `screen --version`, and `zellij --version` for
/// richer metadata.
/// The [`fmt::Display`] implementation returns lowercase names (`tmux`,
/// `screen`, `zellij`).
///
/// Markers:
/// - `TMUX` for tmux
/// - `STY` for screen
/// - `ZELLIJ` for zellij
///
/// Multiplexers are identified separately from the terminal so clients can
/// treat tmux/screen/zellij as distinct from the underlying terminal.
///
/// When multiple markers are present, tmux takes precedence over zellij and
/// screen. Note that `TERM=screen*` does not set the multiplexer kind; GNU
/// Screen requires the `STY` marker. When adding a multiplexer, update markers
/// here and add a test case in `detect.rs`.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// if let Some(mux) = info.multiplexer.as_ref() {
///     println!("multiplexer: {kind}", kind = mux.kind);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiplexerKind {
    Screen,
    Tmux,
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

/// Multiplexer metadata gathered from the environment and optional commands.
///
/// This is returned in [`TerminalInfo::multiplexer`] when a multiplexer marker
/// is present. Fields may be `None` when command probing is disabled, when a
/// multiplexer binary is not available on `PATH`, or when the multiplexer does
/// not expose equivalent metadata. For non-tmux multiplexers,
/// [`MultiplexerInfo::client_term`] and [`MultiplexerInfo::client_type`] are
/// always `None`.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{MultiplexerKind, detect};
///
/// let info = detect();
/// if let Some(mux) = info.multiplexer.as_ref() {
///     let kind = mux.kind.to_string();
///     let version = &mux.version;
///     println!("{kind} version: {version:?}");
/// }
/// ```
///
/// [`TerminalInfo::multiplexer`]: crate::TerminalInfo::multiplexer
/// [`MultiplexerInfo::client_term`]: crate::MultiplexerInfo::client_term
/// [`MultiplexerInfo::client_type`]: crate::MultiplexerInfo::client_type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiplexerInfo {
    /// Multiplexer type detected from environment markers.
    pub kind: MultiplexerKind,
    /// Version extracted from optional command probes.
    ///
    /// This uses `tmux -V`, `screen --version`, or `zellij --version` when
    /// command probing is enabled.
    pub version: Option<String>,
    /// tmux client term name from `tmux display-message`, when available.
    ///
    /// This is only set for tmux sessions with command probing enabled.
    pub client_term: Option<String>,
    /// tmux client term type from `tmux display-message`, when available.
    ///
    /// This is only set for tmux sessions with command probing enabled.
    pub client_type: Option<String>,
}
