//! Detection logic for terminals and multiplexers.
//!
//! See [Detection Logic](crate#detection-logic) for the crate-level narrative.
use crate::command::{
    CommandRunner, CommandSpec, DefaultCommandRunner, run_version_command, tmux_display_message,
};
use crate::env::{EnvView, env_map_from_os};
use crate::{
    CommandProbe, DetectOptions, DetectionSource, EnvMap, Identifier, MultiplexerInfo,
    MultiplexerKind, TerminalInfo, TerminalKind,
};

/// Detect terminal and multiplexer info from the current process environment.
///
/// This uses default [`DetectOptions`] with command probing enabled and captures
/// the subset of environment variables read.
/// The returned [`TerminalInfo`] always includes any `TERM_PROGRAM` or `TERM`
/// values found in the environment.
/// Returns [`TerminalKind::Unknown`] when no markers match.
/// See the [Detection Logic](crate#detection-logic) overview for precedence and
/// mux interaction details.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect;
///
/// let info = detect();
/// println!("terminal: {kind:?}", kind = info.kind);
/// ```
pub fn detect() -> TerminalInfo {
    let env = env_map_from_os();
    detect_from_env(&env)
}

/// Detect terminal and multiplexer info from the provided environment map.
///
/// This uses default [`DetectOptions`] and is useful for testing with a custom
/// environment. The input map drives both detection and the
/// [`TerminalInfo::raw_env_subset`]
/// capture.
/// Returns [`TerminalKind::Unknown`] when no markers match.
/// See the [Detection Logic](crate#detection-logic) overview for precedence and
/// mux interaction details.
///
/// # Example
///
/// ```rust
/// use detect_terminal::detect_from_env;
/// use std::ffi::OsString;
/// use std::collections::BTreeMap;
///
/// let mut env = BTreeMap::new();
/// env.insert(OsString::from("TERM_PROGRAM"), OsString::from("iTerm.app"));
///
/// let info = detect_from_env(&env);
/// println!("terminal: {kind:?}", kind = info.kind);
/// ```
pub fn detect_from_env(env: &EnvMap) -> TerminalInfo {
    detect_with_options(env, DetectOptions::default())
}

/// Detect terminal and multiplexer info with explicit options.
///
/// This is the most flexible entry point and can disable command probes or
/// environment capture. [`TerminalInfo::term_program`],
/// [`TerminalInfo::term_program_version`], and [`TerminalInfo::term`] are set
/// directly from the provided environment map.
/// Returns [`TerminalKind::Unknown`] when no markers match.
/// See the [Detection Logic](crate#detection-logic) overview for precedence and
/// mux interaction details.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{detect_with_options, DetectOptions};
///
/// let env = std::env::vars_os().collect();
/// let options = DetectOptions {
///     allow_commands: false,
///     capture_env_subset: false,
/// };
/// let info = detect_with_options(&env, options);
/// println!("terminal: {kind:?}", kind = info.kind);
/// ```
pub fn detect_with_options(env: &EnvMap, options: DetectOptions) -> TerminalInfo {
    let runner = DefaultCommandRunner;
    detect_with_runner(env, &options, &runner)
}

/// Run detection with an injected command runner for testability.
fn detect_with_runner(
    env: &EnvMap,
    options: &DetectOptions,
    runner: &dyn CommandRunner,
) -> TerminalInfo {
    let mut env_view = EnvView::new(env, options.capture_env_subset);
    let mut info = TerminalInfo::unknown();

    let mux = detect_multiplexer(&mut env_view, options, runner, &mut info.command_probes);
    info.multiplexer = mux;

    let mux_kind = info.multiplexer.as_ref().map(|mux| mux.kind.clone());
    let detection = detect_terminal(
        &mut env_view,
        options,
        mux_kind.as_ref(),
        runner,
        &mut info.command_probes,
    );
    info.kind = detection.kind;
    info.version = detection.version;
    info.term_program = detection.term_program;
    info.term_program_version = detection.term_program_version;
    info.term = detection.term;
    info.raw_name = detection.raw_name;
    info.detected_via = detection.detected_via;
    info.identifiers = detection.identifiers;
    info.raw_env_subset = env_view.into_used();

    info
}

/// Internal detection accumulator used while scanning env markers.
#[derive(Debug, Clone)]
struct DetectionResult {
    kind: TerminalKind,
    version: Option<String>,
    term_program: Option<String>,
    term_program_version: Option<String>,
    term: Option<String>,
    raw_name: Option<String>,
    detected_via: Vec<DetectionSource>,
    identifiers: Vec<Identifier>,
}

impl DetectionResult {
    fn new() -> Self {
        Self {
            kind: TerminalKind::Unknown,
            version: None,
            term_program: None,
            term_program_version: None,
            term: None,
            raw_name: None,
            detected_via: Vec::new(),
            identifiers: Vec::new(),
        }
    }
}

/// Detect multiplexers from env markers and optional command probes.
fn detect_multiplexer(
    env: &mut EnvView<'_>,
    options: &DetectOptions,
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
) -> Option<MultiplexerInfo> {
    if env.contains_key("TMUX") {
        let (version, client_term, client_type) = if options.allow_commands {
            let version = tmux_version(runner, probes);
            let client_term = tmux_display_message(runner, probes, "#{client_termname}");
            let client_type = tmux_display_message(runner, probes, "#{client_termtype}");
            (version, client_term, client_type)
        } else {
            (None, None, None)
        };
        return Some(MultiplexerInfo {
            kind: MultiplexerKind::Tmux,
            version,
            client_term,
            client_type,
        });
    }

    if env.contains_key("ZELLIJ") {
        let version = if options.allow_commands {
            zellij_version(runner, probes)
        } else {
            None
        };
        return Some(MultiplexerInfo {
            kind: MultiplexerKind::Zellij,
            version,
            client_term: None,
            client_type: None,
        });
    }

    if env.contains_key("STY") {
        let version = if options.allow_commands {
            screen_version(runner, probes)
        } else {
            None
        };
        return Some(MultiplexerInfo {
            kind: MultiplexerKind::Screen,
            version,
            client_term: None,
            client_type: None,
        });
    }

    None
}

/// Detect the terminal emulator using env markers and optional mux probes.
///
/// Add new terminal markers here and update the TerminalKind docs and tests.
fn detect_terminal(
    env: &mut EnvView<'_>,
    options: &DetectOptions,
    mux_kind: Option<&MultiplexerKind>,
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
) -> DetectionResult {
    let mut result = DetectionResult::new();

    let term_program = env.get("TERM_PROGRAM");
    let term_program_version = env.get("TERM_PROGRAM_VERSION");
    let term = env.get("TERM");

    result.term_program = term_program.clone();
    result.term_program_version = term_program_version.clone();
    result.term = term.clone();
    result.raw_name = term_program.clone().or_else(|| term.clone());
    result.version = term_program_version.clone();

    if let Some(value) = term_program.clone() {
        let normalized = value.to_lowercase();
        if normalized == "apple_terminal" {
            set_terminal(
                &mut result,
                TerminalKind::AppleTerminal,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "iterm.app" {
            set_terminal(
                &mut result,
                TerminalKind::ITerm2,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "wezterm" {
            set_terminal(
                &mut result,
                TerminalKind::WezTerm,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "vscode" {
            set_terminal(
                &mut result,
                TerminalKind::VSCode,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "hyper" {
            set_terminal(
                &mut result,
                TerminalKind::Hyper,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "warpterminal" {
            set_terminal(
                &mut result,
                TerminalKind::Warp,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "ghostty" {
            set_terminal(
                &mut result,
                TerminalKind::Ghostty,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "mintty" {
            set_terminal(
                &mut result,
                TerminalKind::Mintty,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        } else if normalized == "windows_terminal" {
            set_terminal(
                &mut result,
                TerminalKind::WindowsTerminal,
                term_program_version.clone(),
                "TERM_PROGRAM",
                value,
            );
        }
    }

    if result.kind == TerminalKind::Unknown && env.contains_key("WT_SESSION") {
        set_terminal_simple(&mut result, TerminalKind::WindowsTerminal, "WT_SESSION");
    }

    if result.kind == TerminalKind::Unknown
        && let Some(session) = env.get("SESSIONNAME")
        && session.eq_ignore_ascii_case("console")
    {
        set_terminal(
            &mut result,
            TerminalKind::ConHost,
            None,
            "SESSIONNAME",
            session,
        );
    }

    if result.kind == TerminalKind::Unknown {
        if env.contains_key("WEZTERM_EXECUTABLE") {
            set_terminal_simple(&mut result, TerminalKind::WezTerm, "WEZTERM_EXECUTABLE");
        } else if env.contains_key("WEZTERM_PANE") {
            set_terminal_simple(&mut result, TerminalKind::WezTerm, "WEZTERM_PANE");
        }
    }

    if result.kind == TerminalKind::Unknown {
        if env.contains_key("KITTY_WINDOW_ID") {
            set_terminal_simple(&mut result, TerminalKind::Kitty, "KITTY_WINDOW_ID");
        } else if env.contains_key("KITTY_PID") {
            set_terminal_simple(&mut result, TerminalKind::Kitty, "KITTY_PID");
        }
    }

    if result.kind == TerminalKind::Unknown && env.contains_key("ALACRITTY_SOCKET") {
        set_terminal_simple(&mut result, TerminalKind::Alacritty, "ALACRITTY_SOCKET");
    }

    if result.kind == TerminalKind::Unknown && env.contains_key("GHOSTTY") {
        set_terminal_simple(&mut result, TerminalKind::Ghostty, "GHOSTTY");
    }

    if result.kind == TerminalKind::Unknown
        && let Some(value) = env.get("TERMINAL_EMULATOR")
        && value.to_lowercase().contains("jetbrains")
    {
        set_terminal(
            &mut result,
            TerminalKind::JetBrains,
            None,
            "TERMINAL_EMULATOR",
            value,
        );
    }

    if result.kind == TerminalKind::Unknown {
        if env.contains_key("KONSOLE_VERSION") {
            set_terminal_simple(&mut result, TerminalKind::Konsole, "KONSOLE_VERSION");
        } else if env.contains_key("GNOME_TERMINAL_SERVICE")
            || env.contains_key("GNOME_TERMINAL_SCREEN")
        {
            set_terminal_simple(
                &mut result,
                TerminalKind::GnomeTerminal,
                "GNOME_TERMINAL_SERVICE",
            );
        } else if env.contains_key("TILIX_ID") {
            set_terminal_simple(&mut result, TerminalKind::Tilix, "TILIX_ID");
        } else if env.contains_key("TERMINATOR_UUID") {
            set_terminal_simple(&mut result, TerminalKind::Terminator, "TERMINATOR_UUID");
        } else if env.contains_key("XFCE4_TERMINAL") {
            set_terminal_simple(&mut result, TerminalKind::Xfce4Terminal, "XFCE4_TERMINAL");
        } else if env.contains_key("FOOT_CLIENT") {
            set_terminal_simple(&mut result, TerminalKind::Foot, "FOOT_CLIENT");
        } else if env.contains_key("FOOT_MAIN_PID") {
            set_terminal_simple(&mut result, TerminalKind::Foot, "FOOT_MAIN_PID");
        } else if env.contains_key("XTERM_VERSION") {
            set_terminal_simple(&mut result, TerminalKind::Xterm, "XTERM_VERSION");
        } else if env.contains_key("RXVT_SOCKET") {
            set_terminal_simple(&mut result, TerminalKind::Rxvt, "RXVT_SOCKET");
        } else if env.contains_key("RXVT_TERM") {
            set_terminal_simple(&mut result, TerminalKind::Rxvt, "RXVT_TERM");
        } else if env.contains_key("CMDER_ROOT") {
            set_terminal_simple(&mut result, TerminalKind::Cmder, "CMDER_ROOT");
        } else if env.contains_key("ConEmuPID") {
            set_terminal_simple(&mut result, TerminalKind::ConEmu, "ConEmuPID");
        } else if env.contains_key("ConEmuHWND") {
            set_terminal_simple(&mut result, TerminalKind::ConEmu, "ConEmuHWND");
        } else if env.contains_key("VTE_VERSION") {
            set_terminal_simple(&mut result, TerminalKind::Vte, "VTE_VERSION");
        }
    }

    // Skip TERM=screen* fallback when tmux is detected to avoid mislabeling
    // the terminal as GNU Screen inside tmux sessions.
    if result.kind == TerminalKind::Unknown
        && let Some(value) = term.clone()
    {
        let normalized = value.to_lowercase();
        if normalized.contains("xterm-kitty") {
            set_terminal(&mut result, TerminalKind::Kitty, None, "TERM", value);
        } else if normalized.contains("xterm-ghostty") || normalized.contains("ghostty") {
            set_terminal(&mut result, TerminalKind::Ghostty, None, "TERM", value);
        } else if normalized == "alacritty" {
            set_terminal(&mut result, TerminalKind::Alacritty, None, "TERM", value);
        } else if normalized.starts_with("foot") {
            set_terminal(&mut result, TerminalKind::Foot, None, "TERM", value);
        } else if normalized.starts_with("st") {
            set_terminal(&mut result, TerminalKind::St, None, "TERM", value);
        } else if normalized.contains("rxvt") {
            set_terminal(&mut result, TerminalKind::Rxvt, None, "TERM", value);
        } else if normalized.contains("xterm") {
            set_terminal(&mut result, TerminalKind::Xterm, None, "TERM", value);
        } else if normalized.contains("screen") && mux_kind != Some(&MultiplexerKind::Tmux) {
            set_terminal(&mut result, TerminalKind::Screen, None, "TERM", value);
        }
    }

    if options.allow_commands
        && result.kind == TerminalKind::Unknown
        && mux_kind == Some(&MultiplexerKind::Tmux)
        && let Some(term_name) = tmux_display_message(runner, probes, "#{client_termname}")
        && term_name.to_lowercase().contains("kitty")
    {
        set_terminal(
            &mut result,
            TerminalKind::Kitty,
            None,
            "tmux-client",
            term_name,
        );
        result
            .detected_via
            .push(DetectionSource::Command("tmux display-message".to_string()));
    }

    result
}

/// Assign terminal info when a new match is found.
fn set_terminal(
    result: &mut DetectionResult,
    kind: TerminalKind,
    version: Option<String>,
    key: &str,
    value: String,
) {
    if result.kind != TerminalKind::Unknown {
        return;
    }
    result.kind = kind;
    result.version = version;
    result
        .detected_via
        .push(DetectionSource::EnvVar(key.to_string()));
    result.identifiers.push(Identifier {
        key: key.to_string(),
        value,
    });
}

/// Assign terminal info without storing a specific identifier value.
fn set_terminal_simple(result: &mut DetectionResult, kind: TerminalKind, key: &str) {
    if result.kind != TerminalKind::Unknown {
        return;
    }
    result.kind = kind;
    result
        .detected_via
        .push(DetectionSource::EnvVar(key.to_string()));
}

/// Run `tmux -V` and parse its version output.
fn tmux_version(runner: &dyn CommandRunner, probes: &mut Vec<CommandProbe>) -> Option<String> {
    let spec = CommandSpec {
        program: "tmux",
        args: vec!["-V"],
    };
    run_version_command(runner, probes, spec, parse_tmux_version)
}

/// Run `zellij --version` and parse its version output.
fn zellij_version(runner: &dyn CommandRunner, probes: &mut Vec<CommandProbe>) -> Option<String> {
    let spec = CommandSpec {
        program: "zellij",
        args: vec!["--version"],
    };
    run_version_command(runner, probes, spec, parse_zellij_version)
}

/// Run `screen --version` and parse its version output.
fn screen_version(runner: &dyn CommandRunner, probes: &mut Vec<CommandProbe>) -> Option<String> {
    let spec = CommandSpec {
        program: "screen",
        args: vec!["--version"],
    };
    run_version_command(runner, probes, spec, parse_screen_version)
}

/// Parse `tmux -V` output into a version string.
fn parse_tmux_version(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if let Some(version) = trimmed.strip_prefix("tmux ") {
        Some(version.to_string())
    } else {
        trimmed
            .split_whitespace()
            .nth(1)
            .map(|value| value.to_string())
    }
}

/// Parse `zellij --version` output into a version string.
fn parse_zellij_version(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if let Some(version) = trimmed.strip_prefix("zellij ") {
        Some(version.to_string())
    } else {
        trimmed
            .split_whitespace()
            .nth(1)
            .map(|value| value.to_string())
    }
}

/// Parse `screen --version` output into a version string.
fn parse_screen_version(output: &str) -> Option<String> {
    let trimmed = output.trim();
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for window in tokens.windows(2) {
        if window[0].eq_ignore_ascii_case("version") {
            return Some(window[1].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandOutput;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::collections::BTreeMap;
    use std::ffi::OsString;

    struct TestCommandRunner {
        outputs: BTreeMap<String, CommandOutput>,
    }

    impl TestCommandRunner {
        fn new(outputs: BTreeMap<String, CommandOutput>) -> Self {
            Self { outputs }
        }
    }

    impl CommandRunner for TestCommandRunner {
        fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
            self.outputs
                .get(&spec.as_command_string())
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing"))
        }
    }

    fn env_from_pairs(pairs: &[(&str, &str)]) -> EnvMap {
        pairs
            .iter()
            .map(|(key, value)| (OsString::from(*key), OsString::from(*value)))
            .collect()
    }

    #[rstest]
    #[case(
        env_from_pairs(&[("TERM_PROGRAM", "iTerm.app"), ("TERM_PROGRAM_VERSION", "3.4.22")]),
        TerminalKind::ITerm2,
        Some("3.4.22".to_string()),
        Some("iTerm.app".to_string())
    )]
    #[case(
        env_from_pairs(&[("TERM_PROGRAM", "Apple_Terminal")]),
        TerminalKind::AppleTerminal,
        None,
        Some("Apple_Terminal".to_string())
    )]
    #[case(
        env_from_pairs(&[("TERM", "xterm-kitty")]),
        TerminalKind::Kitty,
        None,
        Some("xterm-kitty".to_string())
    )]
    #[case(
        env_from_pairs(&[("TERM_PROGRAM", "iTerm.app"), ("TERM", "xterm-kitty")]),
        TerminalKind::ITerm2,
        None,
        Some("iTerm.app".to_string())
    )]
    #[case(
        env_from_pairs(&[("WEZTERM_EXECUTABLE", "/opt/wezterm")]),
        TerminalKind::WezTerm,
        None,
        None
    )]
    #[case(
        env_from_pairs(&[("GNOME_TERMINAL_SERVICE", "1")]),
        TerminalKind::GnomeTerminal,
        None,
        None
    )]
    #[case(
        env_from_pairs(&[("WT_SESSION", "abc123")]),
        TerminalKind::WindowsTerminal,
        None,
        None
    )]
    #[case(
        env_from_pairs(&[("SESSIONNAME", "Console")]),
        TerminalKind::ConHost,
        None,
        None
    )]
    fn detects_terminals(
        #[case] env: EnvMap,
        #[case] expected_kind: TerminalKind,
        #[case] expected_version: Option<String>,
        #[case] expected_raw: Option<String>,
    ) {
        let runner = TestCommandRunner::new(BTreeMap::new());
        let info = detect_with_runner(&env, &DetectOptions::default(), &runner);
        assert_eq!(info.kind, expected_kind);
        assert_eq!(info.version, expected_version);
        if expected_raw.as_deref() == Some("iTerm.app") {
            assert_eq!(info.term_program.as_deref(), Some("iTerm.app"));
        }
        if expected_raw.is_some() {
            assert_eq!(info.raw_name, expected_raw);
        }
    }

    #[test]
    fn preserves_term_program_version_for_unknown_terminal() {
        let env = env_from_pairs(&[
            ("TERM_PROGRAM", "unknown-term"),
            ("TERM_PROGRAM_VERSION", "9.9.9"),
        ]);
        let runner = TestCommandRunner::new(BTreeMap::new());
        let info = detect_with_runner(&env, &DetectOptions::default(), &runner);
        assert_eq!(info.kind, TerminalKind::Unknown);
        assert_eq!(info.term_program_version.as_deref(), Some("9.9.9"));
        assert_eq!(info.version.as_deref(), Some("9.9.9"));
    }

    #[test]
    fn tmux_does_not_set_screen_terminal_kind() {
        let env = env_from_pairs(&[("TMUX", "1"), ("TERM", "screen-256color")]);
        let runner = TestCommandRunner::new(BTreeMap::new());
        let info = detect_with_runner(&env, &DetectOptions::default(), &runner);
        assert_eq!(info.kind, TerminalKind::Unknown);
        assert_eq!(info.multiplexer.unwrap().kind, MultiplexerKind::Tmux);
    }

    #[test]
    fn detects_tmux_with_command_info() {
        let env = env_from_pairs(&[("TMUX", "1")]);
        let mut outputs = BTreeMap::new();
        outputs.insert(
            "tmux -V".to_string(),
            CommandOutput {
                stdout: "tmux 3.4".to_string(),
                stderr: String::new(),
                status: Some(0),
            },
        );
        outputs.insert(
            "tmux display-message -p #{client_termname}".to_string(),
            CommandOutput {
                stdout: "xterm-256color".to_string(),
                stderr: String::new(),
                status: Some(0),
            },
        );
        outputs.insert(
            "tmux display-message -p #{client_termtype}".to_string(),
            CommandOutput {
                stdout: "screen".to_string(),
                stderr: String::new(),
                status: Some(0),
            },
        );
        let runner = TestCommandRunner::new(outputs);

        let info = detect_with_runner(&env, &DetectOptions::default(), &runner);
        let mux = info.multiplexer.expect("multiplexer");
        assert_eq!(mux.kind, MultiplexerKind::Tmux);
        assert_eq!(mux.version, Some("3.4".to_string()));
        assert_eq!(mux.client_term, Some("xterm-256color".to_string()));
        assert_eq!(mux.client_type, Some("screen".to_string()));
    }

    #[test]
    fn captures_raw_env_subset() {
        let env = env_from_pairs(&[("TERM_PROGRAM", "iTerm.app"), ("EXTRA", "nope")]);
        let runner = TestCommandRunner::new(BTreeMap::new());
        let info = detect_with_runner(&env, &DetectOptions::default(), &runner);
        assert_eq!(info.raw_env_subset.contains_key("TERM_PROGRAM"), true);
        assert_eq!(info.raw_env_subset.contains_key("EXTRA"), false);
    }

    #[test]
    fn disables_command_probes() {
        let env = env_from_pairs(&[("TMUX", "1")]);
        let runner = TestCommandRunner::new(BTreeMap::new());
        let options = DetectOptions {
            allow_commands: false,
            capture_env_subset: true,
        };
        let info = detect_with_runner(&env, &options, &runner);
        assert_eq!(info.command_probes.len(), 0);
        assert_eq!(info.multiplexer.unwrap().version, None);
    }

    #[rstest]
    #[case("tmux 3.3a", Some("3.3a".to_string()))]
    #[case("tmux 3.4", Some("3.4".to_string()))]
    fn parses_tmux_version(#[case] output: &str, #[case] expected: Option<String>) {
        assert_eq!(parse_tmux_version(output), expected);
    }

    #[rstest]
    #[case("zellij 0.39.2", Some("0.39.2".to_string()))]
    #[case("zellij 0.40.0-dev", Some("0.40.0-dev".to_string()))]
    fn parses_zellij_version(#[case] output: &str, #[case] expected: Option<String>) {
        assert_eq!(parse_zellij_version(output), expected);
    }

    #[rstest]
    #[case(
        "Screen version 4.08.00 (GNU) 05-Feb-20",
        Some("4.08.00".to_string())
    )]
    fn parses_screen_version(#[case] output: &str, #[case] expected: Option<String>) {
        assert_eq!(parse_screen_version(output), expected);
    }
}
