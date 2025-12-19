use crate::multiplexer::MultiplexerInfo;
use std::collections::BTreeMap;
use std::fmt;

/// Known terminal emulators detected from environment markers.
///
/// This enum is populated by [`detect`], [`detect_from_env`], and
/// [`detect_with_options`] based on environment markers and optional mux
/// commands. Marker lists include program identity (`TERM_PROGRAM`) and
/// emulation identity (`TERM`) where applicable; both may be available in
/// `TerminalInfo` even when a specific `TerminalKind` is not identified.
/// Detection prefers explicit program markers before falling back to `TERM`
/// heuristics.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalKind {
    /// Name: Alacritty.
    /// Markers: `ALACRITTY_SOCKET`, `TERM=alacritty`.
    /// <https://alacritty.org>
    Alacritty,
    /// Name: Terminal.app.
    /// Markers: `TERM_PROGRAM=Apple_Terminal`.
    /// <https://support.apple.com/guide/terminal/welcome/mac>
    AppleTerminal,
    /// Name: Cmder.
    /// Markers: `CMDER_ROOT`.
    /// <https://cmder.app>
    Cmder,
    /// Name: ConEmu.
    /// Markers: `ConEmuPID`, `ConEmuHWND`.
    /// <https://conemu.github.io>
    ConEmu,
    /// Name: Windows Console Host (conhost.exe).
    /// Markers: `SESSIONNAME=Console`.
    /// <https://learn.microsoft.com/en-us/windows/console/>
    ConHost,
    /// Name: foot.
    /// Markers: `FOOT_CLIENT`, `FOOT_MAIN_PID`, `TERM=foot*`.
    /// <https://codeberg.org/dnkl/foot>
    Foot,
    /// Name: Ghostty.
    /// Markers: `TERM_PROGRAM=ghostty`, `GHOSTTY`, `TERM=xterm-ghostty`.
    /// <https://ghostty.org>
    Ghostty,
    /// Name: GNOME Terminal.
    /// Markers: `GNOME_TERMINAL_SERVICE`, `GNOME_TERMINAL_SCREEN`.
    /// <https://wiki.gnome.org/Apps/Terminal>
    GnomeTerminal,
    /// Name: Hyper.
    /// Markers: `TERM_PROGRAM=Hyper`.
    /// <https://hyper.is>
    Hyper,
    /// Name: iTerm2.
    /// Markers: `TERM_PROGRAM=iTerm.app`, `TERM_PROGRAM_VERSION`.
    /// <https://iterm2.com>
    ITerm2,
    /// Name: JetBrains IDE Terminal.
    /// Markers: `TERMINAL_EMULATOR` containing `jetbrains`.
    /// <https://www.jetbrains.com/help/idea/terminal-emulator.html>
    JetBrains,
    /// Name: Kitty.
    /// Markers: `KITTY_WINDOW_ID`, `KITTY_PID`, `TERM=xterm-kitty`.
    /// <https://sw.kovidgoyal.net/kitty/>
    Kitty,
    /// Name: Konsole.
    /// Markers: `KONSOLE_VERSION`.
    /// <https://konsole.kde.org>
    Konsole,
    /// Name: mintty.
    /// Markers: `TERM_PROGRAM=mintty`.
    /// <https://mintty.github.io>
    Mintty,
    /// Name: rxvt-unicode.
    /// Markers: `RXVT_SOCKET`, `RXVT_TERM`, `TERM=rxvt*`.
    /// <https://software.schmorp.de/pkg/rxvt-unicode.html>
    Rxvt,
    /// Name: GNU Screen.
    /// Markers: `TERM=screen*`.
    /// <https://www.gnu.org/software/screen/>
    Screen,
    /// Name: st.
    /// Markers: `TERM=st*`.
    /// <https://st.suckless.org>
    St,
    /// Name: Terminator.
    /// Markers: `TERMINATOR_UUID`.
    /// <https://gnome-terminator.org>
    Terminator,
    /// Name: Tilix.
    /// Markers: `TILIX_ID`.
    /// <https://gnunn1.github.io/tilix-web/>
    Tilix,
    /// Name: VTE-based terminal.
    /// Markers: `VTE_VERSION`.
    /// <https://wiki.gnome.org/Apps/Terminal/VTE>
    Vte,
    /// Name: VS Code Terminal.
    /// Markers: `TERM_PROGRAM=vscode`.
    /// <https://code.visualstudio.com/docs/terminal/basics>
    VSCode,
    /// Name: Warp.
    /// Markers: `TERM_PROGRAM=WarpTerminal`.
    /// <https://www.warp.dev>
    Warp,
    /// Name: WezTerm.
    /// Markers: `TERM_PROGRAM=WezTerm`, `WEZTERM_EXECUTABLE`, `WEZTERM_PANE`.
    /// <https://wezfurlong.org/wezterm/>
    WezTerm,
    /// Name: Windows Terminal.
    /// Markers: `WT_SESSION`, `TERM_PROGRAM=Windows_Terminal`.
    /// <https://aka.ms/terminal>
    WindowsTerminal,
    /// Name: Xfce4 Terminal.
    /// Markers: `XFCE4_TERMINAL`.
    /// <https://docs.xfce.org/apps/terminal/start>
    Xfce4Terminal,
    /// Name: xterm.
    /// Markers: `XTERM_VERSION`, `TERM=xterm*`.
    /// <https://invisible-island.net/xterm/>
    Xterm,
    /// Name: Unknown.
    Unknown,
}

impl fmt::Display for TerminalKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TerminalKind::Alacritty => "Alacritty",
            TerminalKind::AppleTerminal => "Terminal.app",
            TerminalKind::Cmder => "Cmder",
            TerminalKind::ConEmu => "ConEmu",
            TerminalKind::ConHost => "ConHost",
            TerminalKind::Foot => "foot",
            TerminalKind::Ghostty => "Ghostty",
            TerminalKind::GnomeTerminal => "GNOME Terminal",
            TerminalKind::Hyper => "Hyper",
            TerminalKind::ITerm2 => "iTerm2",
            TerminalKind::JetBrains => "JetBrains",
            TerminalKind::Kitty => "Kitty",
            TerminalKind::Konsole => "Konsole",
            TerminalKind::Mintty => "mintty",
            TerminalKind::Rxvt => "rxvt",
            TerminalKind::Screen => "screen",
            TerminalKind::St => "st",
            TerminalKind::Terminator => "Terminator",
            TerminalKind::Tilix => "Tilix",
            TerminalKind::Vte => "VTE",
            TerminalKind::VSCode => "VS Code",
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
/// populate `term_program`, `term_program_version`, and `term`.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// for identifier in info.identifiers {
///     println!("{}={}", identifier.key, identifier.value);
/// }
/// ```
///
/// [`TerminalInfo::identifiers`]: crate::TerminalInfo::identifiers
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
/// entries in `TerminalInfo::command_probes`.
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
/// This is populated when [`DetectOptions::allow_commands`] is true. Each probe
/// stores the invoked command and captured output for auditing.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// for probe in info.command_probes {
///     println!("{} -> {}", probe.command, probe.stdout);
/// }
/// ```
///
/// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
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
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// println!("terminal: {:?}", info.kind);
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
    /// This currently uses `TERM_PROGRAM_VERSION` when present.
    pub version: Option<String>,
    /// `TERM_PROGRAM` when provided by the terminal (program identity).
    ///
    /// This identifies the terminal application, for example `iTerm.app`.
    pub term_program: Option<String>,
    /// `TERM_PROGRAM_VERSION` when provided by the terminal.
    ///
    /// This is a raw string value reported by the terminal app.
    pub term_program_version: Option<String>,
    /// `TERM` value describing emulation capabilities (emulation identity).
    ///
    /// This describes how the terminal wants to be treated by applications,
    /// for example `xterm-ghostty` or `screen-256color`.
    pub term: Option<String>,
    /// Raw name from `TERM_PROGRAM` or `TERM`, when available.
    ///
    /// This is a quick, best-effort label when a specific terminal kind is not
    /// identified.
    pub raw_name: Option<String>,
    /// Multiplexer details if a tmux/screen/zellij session was detected.
    ///
    /// Multiplexer detection is independent of the terminal kind.
    pub multiplexer: Option<MultiplexerInfo>,
    /// Sources that contributed to the final detection result.
    ///
    /// Values correspond to either env keys or command probes used by the
    /// detector.
    pub detected_via: Vec<DetectionSource>,
    /// Environment identifiers used to reach the result.
    ///
    /// This is a curated subset of env key/value pairs that matched a detector
    /// rule.
    pub identifiers: Vec<Identifier>,
    /// Subset of the environment variables read during detection.
    ///
    /// This is populated when `DetectOptions::capture_env_subset` is enabled.
    pub raw_env_subset: BTreeMap<String, String>,
    /// Command probe output collected during detection.
    ///
    /// This is populated when `DetectOptions::allow_commands` is enabled.
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
