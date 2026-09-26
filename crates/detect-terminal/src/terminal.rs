use std::collections::BTreeMap;
use std::fmt;

use crate::multiplexer::MultiplexerInfo;

/// A terminal application or emulation family inferred from available hints.
///
/// A variant does not always identify the application hosting the session. For example, `Xterm`
/// can mean only that `TERM` names an xterm-compatible emulation. Inspect
/// [`TerminalInfo::detected_via`] to distinguish a program marker from a terminfo fallback.
/// See [Detection logic](crate#detection-logic) for precedence and each variant for its markers.
///
/// [`Unknown`](Self::Unknown) means no supported hint matched; raw values remain in
/// [`TerminalInfo`]. [`fmt::Display`] returns a user-facing name, not a capability guarantee.
/// The enum is non-exhaustive, so matches need a fallback for other and future variants.
///
/// # Example
///
/// ```
/// use detect_terminal::{TerminalKind, detect};
///
/// match detect().kind {
///     TerminalKind::Ghostty => println!("Ghostty hint found"),
///     TerminalKind::Unknown => println!("No recognized terminal hint"),
///     other => println!("Other application or emulation: {other}"),
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalKind {
    /// Alacritty terminal emulator. <https://alacritty.org>
    ///
    /// Markers: `ALACRITTY_SOCKET`, `TERM=alacritty`.
    Alacritty,
    /// Terminal.app on macOS. <https://support.apple.com/guide/terminal/welcome/mac>
    ///
    /// Markers: `TERM_PROGRAM=Apple_Terminal`.
    AppleTerminal,
    /// Cmder terminal emulator. <https://cmder.app>
    ///
    /// Markers: `CMDER_ROOT`.
    Cmder,
    /// ConEmu terminal emulator. <https://conemu.github.io>
    ///
    /// Markers: `ConEmuPID`, `ConEmuHWND`.
    ConEmu,
    /// foot terminal emulator. <https://codeberg.org/dnkl/foot>
    ///
    /// Markers: `TERM=foot` and `foot-` variants.
    Foot,
    /// Ghostty terminal emulator. <https://ghostty.org>
    ///
    /// Markers: `TERM_PROGRAM=ghostty`, `GHOSTTY_RESOURCES_DIR`, `TERM=xterm-ghostty`.
    Ghostty,
    /// GNOME Terminal. <https://wiki.gnome.org/Apps/Terminal>
    ///
    /// Markers: `GNOME_TERMINAL_SERVICE`, `GNOME_TERMINAL_SCREEN`.
    GnomeTerminal,
    /// Hyper terminal emulator. <https://hyper.is>
    ///
    /// Markers: `TERM_PROGRAM=Hyper`.
    Hyper,
    /// iTerm2 terminal emulator. <https://iterm2.com>
    ///
    /// Markers: `TERM_PROGRAM=iTerm.app`, `TERM_PROGRAM_VERSION`.
    ITerm2,
    /// JetBrains IDE terminal. <https://www.jetbrains.com/help/idea/terminal-emulator.html>
    ///
    /// Markers: `TERMINAL_EMULATOR` containing `jetbrains`.
    JetBrains,
    /// Kitty terminal emulator. <https://sw.kovidgoyal.net/kitty/>
    ///
    /// Markers: `KITTY_WINDOW_ID`, `KITTY_PID`, `TERM=xterm-kitty`.
    Kitty,
    /// Konsole terminal emulator. <https://konsole.kde.org>
    ///
    /// Markers: `KONSOLE_VERSION`.
    Konsole,
    /// mintty terminal emulator. <https://mintty.github.io>
    ///
    /// Markers: `TERM_PROGRAM=mintty`.
    Mintty,
    /// rxvt-unicode terminal emulator. <https://software.schmorp.de/pkg/rxvt-unicode.html>
    ///
    /// Markers: `RXVT_SOCKET`, `RXVT_TERM`, `TERM=rxvt*`.
    Rxvt,
    /// GNU Screen terminal emulation. <https://www.gnu.org/software/screen/>
    ///
    /// Markers: `TERM=screen*` (mux marker `STY` only affects multiplexer detection).
    Screen,
    /// Simple terminal (st). <https://st.suckless.org>
    ///
    /// Markers: `TERM=st` and `st-` variants.
    St,
    /// Terminator terminal emulator. <https://gnome-terminator.org>
    ///
    /// Markers: `TERMINATOR_UUID`.
    Terminator,
    /// Tilix terminal emulator. <https://gnunn1.github.io/tilix-web/>
    ///
    /// Markers: `TILIX_ID`.
    Tilix,
    /// VTE-based terminal emulator. <https://wiki.gnome.org/Apps/Terminal/VTE>
    ///
    /// Markers: `VTE_VERSION`.
    Vte,
    /// Visual Studio Code terminal. <https://code.visualstudio.com/docs/terminal/basics>
    ///
    /// Markers: `TERM_PROGRAM=vscode`.
    VSCode,
    /// Warp terminal emulator. <https://www.warp.dev>
    ///
    /// Markers: `TERM_PROGRAM=WarpTerminal`.
    Warp,
    /// WezTerm terminal emulator. <https://wezfurlong.org/wezterm/>
    ///
    /// Markers: `TERM_PROGRAM=WezTerm`, `WEZTERM_EXECUTABLE`, `WEZTERM_PANE`.
    WezTerm,
    /// Windows Terminal. <https://aka.ms/terminal>
    ///
    /// Markers: `WT_SESSION`, `TERM_PROGRAM=Windows_Terminal`.
    WindowsTerminal,
    /// xterm terminal emulator. <https://invisible-island.net/xterm/>
    ///
    /// Markers: `XTERM_VERSION`, `TERM=xterm*`.
    Xterm,
    /// Unknown or unsupported terminal.
    Unknown,
}

impl fmt::Display for TerminalKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TerminalKind::Alacritty => "Alacritty",
            TerminalKind::AppleTerminal => "Terminal.app",
            TerminalKind::Cmder => "Cmder",
            TerminalKind::ConEmu => "ConEmu",
            TerminalKind::Foot => "foot",
            TerminalKind::Ghostty => "Ghostty",
            TerminalKind::GnomeTerminal => "GNOME Terminal",
            TerminalKind::Hyper => "Hyper",
            TerminalKind::ITerm2 => "iTerm2",
            TerminalKind::JetBrains => "JetBrains IDE Terminal",
            TerminalKind::Kitty => "Kitty",
            TerminalKind::Konsole => "Konsole",
            TerminalKind::Mintty => "mintty",
            TerminalKind::Rxvt => "rxvt-unicode",
            TerminalKind::Screen => "GNU Screen",
            TerminalKind::St => "st",
            TerminalKind::Terminator => "Terminator",
            TerminalKind::Tilix => "Tilix",
            TerminalKind::Vte => "VTE-based terminal",
            TerminalKind::VSCode => "Visual Studio Code Terminal",
            TerminalKind::Warp => "Warp",
            TerminalKind::WezTerm => "WezTerm",
            TerminalKind::WindowsTerminal => "Windows Terminal",
            TerminalKind::Xterm => "xterm",
            TerminalKind::Unknown => "Unknown",
        };
        formatter.write_str(label)
    }
}

/// An environment key and value that identified the terminal.
///
/// Command-derived matches have a [`DetectionSource::Command`] and no environment identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// Name of the matching variable.
    pub key: String,

    /// Lossily decoded value of the matching variable.
    pub value: String,
}

/// Evidence for the selected terminal kind, separate from multiplexer metadata.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSource {
    /// The full diagnostic command string of a successful probe.
    Command(String),
    /// The matching environment key, using the detector's canonical spelling.
    EnvVar(String),
}

impl fmt::Display for DetectionSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(command) => write!(formatter, "command: {command}"),
            Self::EnvVar(key) => write!(formatter, "env: {key}"),
        }
    }
}

/// A completed or failed command attempt, recorded in execution order.
///
/// `stdout` and `stderr` are trimmed and decoded lossily. Output from unsuccessful commands is
/// retained for diagnostics but cannot supply detected metadata. See the crate's
/// [probe contract](crate#optional-command-probes) for execution behavior.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProbe {
    /// Diagnostic command string. Arguments are joined by spaces, without shell escaping.
    pub command: String,

    /// Captured standard output, empty when execution failed before collection.
    pub stdout: String,

    /// Captured standard error, empty when execution failed before collection.
    pub stderr: String,

    /// Exit code, or `None` for a signal termination or execution failure.
    pub status: Option<i32>,

    /// Process creation or output collection error, if any.
    pub error: Option<String>,
}

/// Terminal identity, multiplexer metadata, and the evidence used to select them.
///
/// Obtain this result with [`detect`](crate::detect()),
/// [`detect_from_env`](crate::detect_from_env),
/// or [`detect_with_options`](crate::detect_with_options). An unknown terminal is a valid result,
/// and a known multiplexer does not imply a known terminal application.
///
/// # Reading the result
///
/// Start with [`kind`](Self::kind), [`version`](Self::version), and
/// [`multiplexer`](Self::multiplexer). A kind can identify an emulation family rather than an
/// application, and version metadata may be absent even for a known kind.
///
/// For unexpected or unknown results, inspect [`detected_via`](Self::detected_via) and
/// [`identifiers`](Self::identifiers). The raw `term_program`, `term_program_version`, and `term`
/// fields preserve the input hints even when they do not identify the selected kind. `raw_name`
/// is a convenience fallback label, not the detected application's name.
///
/// [`raw_env_subset`](Self::raw_env_subset) contains consulted variables when capture is enabled;
/// [`command_probes`](Self::command_probes) contains attempted commands when probing is enabled.
/// These diagnostics can contain private values. See [Diagnostics and
/// privacy](crate#diagnostics-and-privacy) and [Detection logic](crate#detection-logic).
///
/// # Example
///
/// ```
/// let info = detect_terminal::detect();
/// println!("Terminal: {}", info.kind);
/// if let Some(version) = &info.version {
///     println!("Version: {version}");
/// }
/// for source in &info.detected_via {
///     println!("Selected using: {source}");
/// }
/// // Debug formatting escapes control characters in raw environment values.
/// println!(
///     "Program hint: {:?}; emulation hint: {:?}",
///     info.term_program, info.term
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    /// Application or emulation family selected from the available hints.
    pub kind: TerminalKind,

    /// `TERM_PROGRAM_VERSION` only when `TERM_PROGRAM` identified the selected kind.
    ///
    /// A fallback match never borrows a version belonging to another application or multiplexer.
    pub version: Option<String>,

    /// Raw `TERM_PROGRAM`, including unknown or empty values.
    pub term_program: Option<String>,

    /// Raw `TERM_PROGRAM_VERSION`, including when its owner is unknown.
    pub term_program_version: Option<String>,

    /// Raw `TERM` describing the emulation environment.
    pub term: Option<String>,

    /// Raw `TERM_PROGRAM` if present, otherwise `TERM`.
    ///
    /// This convenience label can name something different from the selected kind.
    pub raw_name: Option<String>,

    /// One selected multiplexer, independently of the terminal match.
    pub multiplexer: Option<MultiplexerInfo>,

    /// Evidence for the terminal match, empty when the kind is unknown.
    pub detected_via: Vec<DetectionSource>,

    /// Matching environment key/value pairs, empty for unknown or command-derived matches.
    pub identifiers: Vec<Identifier>,

    /// Existing variables consulted during detection, when diagnostic capture is enabled.
    ///
    /// Absent keys and unrelated variables are omitted. Disabling capture does not redact the
    /// other fields in this result. Values may contain session identifiers and paths.
    pub raw_env_subset: BTreeMap<String, String>,

    /// All attempted probes, empty when command execution is disabled or no mux marker matches.
    pub command_probes: Vec<CommandProbe>,
}

impl TerminalInfo {
    /// Start an empty result before assigning environment and probe evidence.
    pub(crate) fn unknown() -> Self {
        Self {
            kind: TerminalKind::Unknown,
            version: None,
            term_program: None,
            term_program_version: None,
            term: None,
            raw_name: None,
            multiplexer: None,
            detected_via: Vec::new(),
            identifiers: Vec::new(),
            raw_env_subset: BTreeMap::new(),
            command_probes: Vec::new(),
        }
    }
}
