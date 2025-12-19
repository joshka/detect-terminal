use std::fmt;

/// Multiplexer kinds detected from environment markers.
///
/// The detector checks env markers first and optionally runs `tmux -V`,
/// `tmux display-message -p`, `screen --version`, and `zellij --version` for
/// richer metadata.
///
/// Markers:
/// - `TMUX` for tmux
/// - `STY` for screen
/// - `ZELLIJ` for zellij
///
/// Multiplexers are identified separately from the terminal so clients can
/// treat tmux/screen/zellij as distinct from the underlying terminal.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// if let Some(mux) = info.multiplexer.as_ref() {
///     println!("multiplexer: {}", mux.kind);
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
/// This is returned in [`TerminalInfo::multiplexer`] when a multiplexer marker is
/// present. Fields may be `None` when command probing is disabled or when the
/// multiplexer does not expose equivalent metadata.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{detect, MultiplexerKind};
///
/// let info = detect();
/// if let Some(mux) = info.multiplexer {
///     if mux.kind == MultiplexerKind::Zellij {
///         println!("zellij version: {:?}", mux.version);
///     }
/// }
/// ```
///
/// [`TerminalInfo::multiplexer`]: crate::TerminalInfo::multiplexer
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
