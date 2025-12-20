use crate::multiplexer::MultiplexerInfo;
use std::collections::BTreeMap;
use std::fmt;

/// Known terminal emulators detected from environment markers.
///
/// This enum is populated by [`detect`], [`detect_from_env`], and
/// [`detect_with_options`] based on environment markers and optional mux
/// commands. Marker lists include program identity (`TERM_PROGRAM`) and
/// emulation identity (`TERM`) where applicable; both may be available in
/// [`TerminalInfo`] even when a specific `TerminalKind` is not identified.
/// Detection prefers explicit program markers before falling back to `TERM`
/// heuristics.
/// The [`fmt::Display`] implementation returns the user-facing product name.
/// If the kind is [`TerminalKind::Unknown`], consult
/// [`TerminalInfo::term_program`] and [`TerminalInfo::term`] for raw hints.
///
/// When adding a new terminal, document its markers here and add a test case in
/// `detect.rs`.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{TerminalKind, detect};
///
/// let info = detect();
/// if info.kind == TerminalKind::Unknown {
///     println!("terminal not identified");
/// }
/// ```
///
/// [`detect`]: crate::detect
/// [`detect_from_env`]: crate::detect_from_env
/// [`detect_with_options`]: crate::detect_with_options
/// [`DetectOptions::capture_env_subset`]: crate::DetectOptions::capture_env_subset
/// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
/// [`TerminalInfo`]: crate::TerminalInfo
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Windows Console Host (conhost.exe). <https://learn.microsoft.com/en-us/windows/console/>
    ///
    /// Markers: `SESSIONNAME=Console` (conservative heuristic).
    ConHost,
    /// foot terminal emulator. <https://codeberg.org/dnkl/foot>
    ///
    /// Markers: `FOOT_CLIENT`, `FOOT_MAIN_PID`, `TERM=foot*`.
    Foot,
    /// Ghostty terminal emulator. <https://ghostty.org>
    ///
    /// Markers: `TERM_PROGRAM=ghostty`, `GHOSTTY`, `TERM=xterm-ghostty`.
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
    /// Markers: `TERM=st*`.
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
    /// Xfce4 Terminal. <https://docs.xfce.org/apps/terminal/start>
    ///
    /// Markers: `XFCE4_TERMINAL`.
    Xfce4Terminal,
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
            TerminalKind::ConHost => "Windows Console Host",
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
            TerminalKind::Xfce4Terminal => "Xfce4 Terminal",
            TerminalKind::Xterm => "xterm",
            TerminalKind::Unknown => "Unknown",
        };
        formatter.write_str(label)
    }
}

/// Key-value pairs that contributed to detection.
///
/// This provides the explicit environment values used to reach a result in
/// [`TerminalInfo::identifiers`]. Values come from the same env lookups that
/// populate [`TerminalInfo::term_program`],
/// [`TerminalInfo::term_program_version`], and [`TerminalInfo::term`].
/// Keys and values are only included for markers that matched.
/// The list is ordered in the sequence detection rules were applied.
/// It is empty when no markers match.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// for identifier in &info.identifiers {
///     let key = &identifier.key;
///     let value = &identifier.value;
///     println!("{key}={value}");
/// }
/// ```
///
/// [`TerminalInfo::identifiers`]: crate::TerminalInfo::identifiers
/// [`TerminalInfo::term_program`]: crate::TerminalInfo::term_program
/// [`TerminalInfo::term_program_version`]: crate::TerminalInfo::term_program_version
/// [`TerminalInfo::term`]: crate::TerminalInfo::term
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// Environment variable name that contributed to detection.
    pub key: String,
    /// Environment variable value that contributed to detection.
    pub value: String,
}

/// Sources used to identify the terminal or multiplexer.
///
/// Stored in [`TerminalInfo::detected_via`] alongside
/// [`TerminalInfo::identifiers`] for debugging. Command sources correspond to
/// entries in [`TerminalInfo::command_probes`] by their command string.
/// Sources are ordered by the matching sequence in the detector.
/// The list is empty when no markers match.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{detect, DetectionSource};
///
/// let info = detect();
/// if info
///     .detected_via
///     .iter()
///     .any(|source| matches!(source, DetectionSource::EnvVar(_)))
/// {
///     println!("env var used");
/// }
/// ```
///
/// [`TerminalInfo::detected_via`]: crate::TerminalInfo::detected_via
/// [`TerminalInfo::identifiers`]: crate::TerminalInfo::identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSource {
    Command(String),
    EnvVar(String),
}

impl fmt::Display for DetectionSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionSource::Command(command) => write!(formatter, "command: {command}"),
            DetectionSource::EnvVar(key) => write!(formatter, "env: {key}"),
        }
    }
}

/// Results from a command probe when command execution is enabled.
///
/// This is populated when [`DetectOptions::allow_commands`] is true.
/// Each probe
/// stores the invoked command and captured output for auditing.
/// [`CommandProbe::stdout`] and [`CommandProbe::stderr`] are trimmed and
/// UTF-8 lossy.
/// The list is ordered by execution time and is empty when probes are disabled.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// for probe in &info.command_probes {
///     let command = &probe.command;
///     let stdout = &probe.stdout;
///     println!("{command} -> {stdout}");
/// }
/// ```
///
/// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
/// [`CommandProbe::stderr`]: crate::CommandProbe::stderr
/// [`CommandProbe::stdout`]: crate::CommandProbe::stdout
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProbe {
    /// Command line that was executed.
    pub command: String,
    /// Standard output captured from the probe.
    pub stdout: String,
    /// Standard error captured from the probe.
    pub stderr: String,
    /// Exit status code from the probe, when available.
    pub status: Option<i32>,
}

/// Full detection result including debug data for auditability.
///
/// This is returned by [`detect`], [`detect_from_env`], and
/// [`detect_with_options`]. Fields that reflect environment values are always
/// derived from the current or provided env map, regardless of whether a
/// terminal kind is identified.
/// [`TerminalInfo::multiplexer`] is independent from [`TerminalInfo::kind`] and
/// may be present when `kind` is [`TerminalKind::Unknown`].
/// [`TerminalInfo::kind`] is [`TerminalKind::Unknown`] when no terminal markers
/// match, but [`TerminalInfo::term_program`] and [`TerminalInfo::term`] can
/// still be populated.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// println!("terminal: {kind:?}", kind = info.kind);
/// ```
///
/// [`detect`]: crate::detect
/// [`detect_from_env`]: crate::detect_from_env
/// [`detect_with_options`]: crate::detect_with_options
#[derive(Debug, Clone)]
pub struct TerminalInfo {
    /// Detected terminal emulator kind.
    pub kind: TerminalKind,
    /// Terminal version when reported by environment markers.
    ///
    /// This currently uses `TERM_PROGRAM_VERSION` when present and does not
    /// attempt to parse other env-specific version formats.
    /// It may be `None` even when [`TerminalInfo::term_program`] is present.
    pub version: Option<String>,
    /// `TERM_PROGRAM` when provided by the terminal (program identity).
    ///
    /// This identifies the terminal application, for example `iTerm.app`.
    /// It may be `None` when the terminal does not set `TERM_PROGRAM`.
    pub term_program: Option<String>,
    /// `TERM_PROGRAM_VERSION` when provided by the terminal.
    ///
    /// This is a raw string value reported by the terminal app.
    /// It may be `None` if the terminal does not provide a version.
    pub term_program_version: Option<String>,
    /// `TERM` value describing emulation capabilities (emulation identity).
    ///
    /// This describes how the terminal wants to be treated by applications,
    /// for example `xterm-ghostty` or `screen-256color`.
    /// It may be `None` in minimal environments.
    pub term: Option<String>,
    /// Raw name from `TERM_PROGRAM` or `TERM`, when available.
    ///
    /// This is a quick, best-effort label when a specific terminal kind is not
    /// identified.
    /// It prefers `TERM_PROGRAM` over `TERM`.
    pub raw_name: Option<String>,
    /// Multiplexer details if a tmux/screen/zellij session was detected.
    ///
    /// Multiplexer detection is independent of the terminal kind.
    /// It may be `None` even when `TERM` looks like `screen*`.
    pub multiplexer: Option<MultiplexerInfo>,
    /// Sources that contributed to the final detection result.
    ///
    /// Values correspond to either env keys or command probes used by the
    /// detector.
    /// The list can be empty when no markers match.
    pub detected_via: Vec<DetectionSource>,
    /// Environment identifiers used to reach the result.
    ///
    /// This is a curated subset of env key/value pairs that matched a detector
    /// rule.
    /// The list can be empty when no markers match.
    pub identifiers: Vec<Identifier>,
    /// Subset of the environment variables read during detection.
    ///
    /// This is populated when [`DetectOptions::capture_env_subset`] is enabled.
    /// The map is empty when capture is disabled.
    ///
    /// [`DetectOptions::capture_env_subset`]: crate::DetectOptions::capture_env_subset
    pub raw_env_subset: BTreeMap<String, String>,
    /// Command probe output collected during detection.
    ///
    /// This is populated when [`DetectOptions::allow_commands`] is enabled.
    /// The list is empty when command probes are disabled.
    ///
    /// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
    pub command_probes: Vec<CommandProbe>,
}

impl TerminalInfo {
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
