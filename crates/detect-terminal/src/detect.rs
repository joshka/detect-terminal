//! Detection orchestration. See [Detection logic](crate#detection-logic) for precedence.

use crate::command::{CommandRunner, DefaultCommandRunner, probe_multiplexer};
use crate::env::{EnvView, env_map_from_os};
use crate::{
    DetectOptions, DetectionSource, EnvMap, Identifier, MultiplexerKind, TerminalInfo, TerminalKind,
};

/// Detect the terminal and multiplexer from the current process environment.
///
/// This never executes commands. Use [`detect_with_options`] to opt into command probes.
/// Returns [`TerminalKind::Unknown`] when no supported marker matches.
///
/// # Example
///
/// ```
/// let info = detect_terminal::detect();
/// println!("terminal: {}", info.kind);
/// ```
pub fn detect() -> TerminalInfo {
    detect_from_env(&env_map_from_os())
}

/// Detect the terminal and multiplexer from an environment snapshot without executing commands.
///
/// Non-Unicode values are converted lossily for matching and diagnostics. The input is borrowed
/// without mutation. See [Detection logic](crate#detection-logic) for precedence and limitations.
///
/// # Example
///
/// ```
/// use detect_terminal::{EnvMap, TerminalKind, detect_from_env};
///
/// let env = EnvMap::from([("TERM_PROGRAM".into(), "iTerm.app".into())]);
/// assert_eq!(detect_from_env(&env).kind, TerminalKind::ITerm2);
/// ```
pub fn detect_from_env(env: &EnvMap) -> TerminalInfo {
    detect_with_options(env, DetectOptions::default())
}

/// Detect from a snapshot with explicit command and diagnostic options.
///
/// Opted-in commands inherit the supplied environment, not the calling process's environment.
/// Executables are resolved by the platform using `PATH`; include it in custom snapshots when
/// probing. Commands are synchronous and have no timeout. Failed probes leave optional metadata
/// absent and are recorded in [`TerminalInfo::command_probes`].
///
/// # Disable diagnostic capture
///
/// Keep environment-only detection while omitting the diagnostic map. Other result fields still
/// retain matching identifiers and raw hints. For enabling external commands, see
/// [Optional command probes](crate#optional-command-probes).
///
/// ```
/// use detect_terminal::{DetectOptions, detect_with_options};
///
/// let env = std::env::vars_os().collect();
/// let options = DetectOptions {
///     allow_commands: false,
///     capture_env_subset: false,
/// };
/// let info = detect_with_options(&env, options);
/// assert!(info.command_probes.is_empty());
/// assert!(info.raw_env_subset.is_empty());
/// ```
pub fn detect_with_options(env: &EnvMap, options: DetectOptions) -> TerminalInfo {
    let runner = DefaultCommandRunner { env };
    detect_with_runner(env, options, &runner)
}

/// Keep command execution injectable while sharing the public detection path.
fn detect_with_runner(
    env: &EnvMap,
    options: DetectOptions,
    runner: &dyn CommandRunner,
) -> TerminalInfo {
    let mut env = EnvView::new(env, options.capture_env_subset);
    let mut info = TerminalInfo::unknown();
    info.term_program = env.get("TERM_PROGRAM");
    info.term_program_version = env.get("TERM_PROGRAM_VERSION");
    info.term = env.get("TERM");
    info.raw_name = info.term_program.clone().or_else(|| info.term.clone());

    info.multiplexer = detect_multiplexer(&mut env);
    if options.allow_commands
        && let Some(mux) = &mut info.multiplexer
    {
        probe_multiplexer(mux, runner, &mut info.command_probes);
    }
    detect_terminal(&mut env, &mut info);
    info.raw_env_subset = env.into_used();
    info
}

/// Select one multiplexer using the documented marker order.
fn detect_multiplexer(env: &mut EnvView<'_>) -> Option<crate::MultiplexerInfo> {
    for (key, kind) in [
        ("TMUX", MultiplexerKind::Tmux),
        ("ZELLIJ", MultiplexerKind::Zellij),
        ("STY", MultiplexerKind::Screen),
    ] {
        if env.get(key).is_some_and(|value| !value.is_empty()) {
            return Some(crate::MultiplexerInfo {
                kind,
                version: None,
                client_term: None,
                client_type: None,
            });
        }
    }
    None
}

/// Assign a single terminal match, preferring program identity over emulation hints.
fn detect_terminal(env: &mut EnvView<'_>, info: &mut TerminalInfo) {
    if let Some(program) = info.term_program.clone()
        && let Some(kind) = program_kind(&program)
    {
        record_env_match(info, kind, "TERM_PROGRAM", program);
        info.version = info.term_program_version.clone();
        return;
    }

    // Ordered marker pairs keep precedence visible and record the exact successful lookup.
    for (key, kind) in [
        ("WT_SESSION", TerminalKind::WindowsTerminal),
        ("WEZTERM_EXECUTABLE", TerminalKind::WezTerm),
        ("WEZTERM_PANE", TerminalKind::WezTerm),
        ("KITTY_WINDOW_ID", TerminalKind::Kitty),
        ("KITTY_PID", TerminalKind::Kitty),
        ("ALACRITTY_SOCKET", TerminalKind::Alacritty),
        ("GHOSTTY_RESOURCES_DIR", TerminalKind::Ghostty),
        ("TERMINAL_EMULATOR", TerminalKind::JetBrains),
        ("KONSOLE_VERSION", TerminalKind::Konsole),
        ("GNOME_TERMINAL_SERVICE", TerminalKind::GnomeTerminal),
        ("GNOME_TERMINAL_SCREEN", TerminalKind::GnomeTerminal),
        ("TILIX_ID", TerminalKind::Tilix),
        ("TERMINATOR_UUID", TerminalKind::Terminator),
        ("XTERM_VERSION", TerminalKind::Xterm),
        ("RXVT_SOCKET", TerminalKind::Rxvt),
        ("RXVT_TERM", TerminalKind::Rxvt),
        ("CMDER_ROOT", TerminalKind::Cmder),
        ("ConEmuPID", TerminalKind::ConEmu),
        ("ConEmuHWND", TerminalKind::ConEmu),
        ("VTE_VERSION", TerminalKind::Vte),
    ] {
        let Some(value) = env.get(key).filter(|value| !value.is_empty()) else {
            continue;
        };
        if key == "TERMINAL_EMULATOR" && !value.to_ascii_lowercase().contains("jetbrains") {
            continue;
        }
        record_env_match(info, kind, key, value);
        return;
    }

    if let Some(mux) = &info.multiplexer
        && let Some(client_term) = &mux.client_term
        && let Some(kind) = term_kind(client_term)
    {
        info.kind = kind;
        info.detected_via.push(DetectionSource::Command(
            "tmux display-message -p #{client_termname}".into(),
        ));
        return;
    }

    if let Some(term) = info.term.clone()
        && let Some(kind) = term_kind(&term)
    {
        // screen's terminfo is also used by tmux and zellij; it does not identify their host.
        if kind == TerminalKind::Screen
            && info
                .multiplexer
                .as_ref()
                .is_some_and(|mux| mux.kind != MultiplexerKind::Screen)
        {
            return;
        }
        record_env_match(info, kind, "TERM", term);
    }
}

/// Map explicit program names case-insensitively.
fn program_kind(program: &str) -> Option<TerminalKind> {
    Some(match program.to_ascii_lowercase().as_str() {
        "apple_terminal" => TerminalKind::AppleTerminal,
        "iterm.app" => TerminalKind::ITerm2,
        "wezterm" => TerminalKind::WezTerm,
        "vscode" => TerminalKind::VSCode,
        "hyper" => TerminalKind::Hyper,
        "warpterminal" => TerminalKind::Warp,
        "ghostty" => TerminalKind::Ghostty,
        "rio" => TerminalKind::Rio,
        "mintty" => TerminalKind::Mintty,
        "windows_terminal" => TerminalKind::WindowsTerminal,
        _ => return None,
    })
}

/// Recognize terminfo names at family boundaries rather than arbitrary substrings.
fn term_kind(term: &str) -> Option<TerminalKind> {
    let term = term.to_ascii_lowercase();
    for (name, kind) in [
        ("xterm-kitty", TerminalKind::Kitty),
        ("xterm-ghostty", TerminalKind::Ghostty),
        ("ghostty", TerminalKind::Ghostty),
        ("alacritty", TerminalKind::Alacritty),
        ("foot", TerminalKind::Foot),
        ("st", TerminalKind::St),
        ("rio", TerminalKind::Rio),
        ("rxvt", TerminalKind::Rxvt),
        ("screen", TerminalKind::Screen),
        ("xterm", TerminalKind::Xterm),
    ] {
        if term == name
            || term
                .strip_prefix(name)
                .is_some_and(|suffix| suffix.starts_with('-') || suffix.starts_with('.'))
        {
            return Some(kind);
        }
    }
    None
}

/// Store environment evidence together so the kind, source, and identifier cannot drift.
fn record_env_match(info: &mut TerminalInfo, kind: TerminalKind, key: &str, value: String) {
    info.kind = kind;
    info.detected_via.push(DetectionSource::EnvVar(key.into()));
    info.identifiers.push(Identifier {
        key: key.into(),
        value,
    });
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::command::{CommandOutput, CommandSpec, parse_version};

    /// Build snapshots without changing the process environment.
    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect()
    }

    #[rstest]
    #[case("Apple_Terminal", TerminalKind::AppleTerminal)]
    #[case("iTerm.app", TerminalKind::ITerm2)]
    #[case("WezTerm", TerminalKind::WezTerm)]
    #[case("vscode", TerminalKind::VSCode)]
    #[case("Hyper", TerminalKind::Hyper)]
    #[case("WarpTerminal", TerminalKind::Warp)]
    #[case("ghostty", TerminalKind::Ghostty)]
    #[case("rio", TerminalKind::Rio)]
    #[case("RIO", TerminalKind::Rio)]
    #[case("mintty", TerminalKind::Mintty)]
    #[case("Windows_Terminal", TerminalKind::WindowsTerminal)]
    #[case("GHOSTTY", TerminalKind::Ghostty)]
    fn program_marker_wins_over_other_hints(#[case] program: &str, #[case] kind: TerminalKind) {
        let snapshot = env(&[
            ("TERM_PROGRAM", program),
            ("TERM_PROGRAM_VERSION", "1.2.3"),
            ("WT_SESSION", "abc"),
            ("TERM", "xterm-kitty"),
        ]);
        let info = detect_from_env(&snapshot);
        assert_eq!(info.kind, kind);
        assert_eq!(info.version.as_deref(), Some("1.2.3"));
        assert_eq!(info.term.as_deref(), Some("xterm-kitty"));
        assert_eq!(
            info.detected_via,
            vec![DetectionSource::EnvVar("TERM_PROGRAM".into())]
        );
    }

    #[rstest]
    #[case("WT_SESSION", TerminalKind::WindowsTerminal)]
    #[case("WEZTERM_EXECUTABLE", TerminalKind::WezTerm)]
    #[case("WEZTERM_PANE", TerminalKind::WezTerm)]
    #[case("KITTY_WINDOW_ID", TerminalKind::Kitty)]
    #[case("KITTY_PID", TerminalKind::Kitty)]
    #[case("ALACRITTY_SOCKET", TerminalKind::Alacritty)]
    #[case("GHOSTTY_RESOURCES_DIR", TerminalKind::Ghostty)]
    #[case("KONSOLE_VERSION", TerminalKind::Konsole)]
    #[case("GNOME_TERMINAL_SERVICE", TerminalKind::GnomeTerminal)]
    #[case("GNOME_TERMINAL_SCREEN", TerminalKind::GnomeTerminal)]
    #[case("TILIX_ID", TerminalKind::Tilix)]
    #[case("TERMINATOR_UUID", TerminalKind::Terminator)]
    #[case("XTERM_VERSION", TerminalKind::Xterm)]
    #[case("RXVT_SOCKET", TerminalKind::Rxvt)]
    #[case("RXVT_TERM", TerminalKind::Rxvt)]
    #[case("CMDER_ROOT", TerminalKind::Cmder)]
    #[case("ConEmuPID", TerminalKind::ConEmu)]
    #[case("ConEmuHWND", TerminalKind::ConEmu)]
    #[case("VTE_VERSION", TerminalKind::Vte)]
    fn reports_the_matching_marker(#[case] key: &str, #[case] kind: TerminalKind) {
        let info = detect_from_env(&env(&[(key, "marker value"), ("TERM", "screen-256color")]));
        assert_eq!(info.kind, kind);
        assert_eq!(info.detected_via, vec![DetectionSource::EnvVar(key.into())]);
        assert_eq!(
            info.identifiers,
            vec![Identifier {
                key: key.into(),
                value: "marker value".into()
            }]
        );
        assert_eq!(
            info.raw_env_subset.get(key).map(String::as_str),
            Some("marker value")
        );
    }

    #[rstest]
    #[case("JetBrains-JediTerm", TerminalKind::JetBrains)]
    #[case("unrelated", TerminalKind::Unknown)]
    fn jetbrains_requires_a_recognized_value(#[case] value: &str, #[case] kind: TerminalKind) {
        assert_eq!(
            detect_from_env(&env(&[("TERMINAL_EMULATOR", value)])).kind,
            kind
        );
    }

    #[rstest]
    #[case("xterm-kitty", TerminalKind::Kitty)]
    #[case("xterm-ghostty", TerminalKind::Ghostty)]
    #[case("alacritty", TerminalKind::Alacritty)]
    #[case("alacritty-direct", TerminalKind::Alacritty)]
    #[case("foot-extra", TerminalKind::Foot)]
    #[case("foot", TerminalKind::Foot)]
    #[case("st-256color", TerminalKind::St)]
    #[case("rxvt-unicode-256color", TerminalKind::Rxvt)]
    #[case("screen.xterm-256color", TerminalKind::Screen)]
    #[case("screen-256color", TerminalKind::Screen)]
    #[case("xterm-256color", TerminalKind::Xterm)]
    #[case("rio", TerminalKind::Rio)]
    #[case("rio-256color", TerminalKind::Rio)]
    #[case("riot", TerminalKind::Unknown)]
    #[case("not-rio", TerminalKind::Unknown)]
    #[case("stupid", TerminalKind::Unknown)]
    #[case("not-xterm", TerminalKind::Unknown)]
    #[case("not-ghostty", TerminalKind::Unknown)]
    #[case("screenfake", TerminalKind::Unknown)]
    #[case("dumb", TerminalKind::Unknown)]
    #[case("", TerminalKind::Unknown)]
    fn term_matching_respects_name_boundaries(#[case] term: &str, #[case] kind: TerminalKind) {
        let info = detect_from_env(&env(&[("TERM", term)]));
        assert_eq!(info.kind, kind);
        assert_eq!(info.term.as_deref(), Some(term));
    }

    #[test]
    fn session_name_does_not_identify_conhost_or_mask_specific_markers() {
        assert_eq!(
            detect_from_env(&env(&[("SESSIONNAME", "Console")])).kind,
            TerminalKind::Unknown
        );
        let info = detect_from_env(&env(&[("SESSIONNAME", "Console"), ("ConEmuPID", "123")]));
        assert_eq!(info.kind, TerminalKind::ConEmu);
    }

    #[test]
    fn empty_markers_are_not_matches() {
        let info = detect_from_env(&env(&[
            ("WT_SESSION", ""),
            ("TMUX", ""),
            ("KITTY_WINDOW_ID", "42"),
        ]));
        assert_eq!(info.kind, TerminalKind::Kitty);
        assert!(info.multiplexer.is_none());
    }

    #[test]
    fn preserves_raw_versions_without_assigning_them_to_other_terminals() {
        for term in ["dumb", "xterm-kitty"] {
            let info = detect_from_env(&env(&[
                ("TERM_PROGRAM", "tmux"),
                ("TERM_PROGRAM_VERSION", "3.5"),
                ("TERM", term),
            ]));
            assert_eq!(info.term_program_version.as_deref(), Some("3.5"));
            assert_eq!(info.version, None);
        }
    }

    #[test]
    fn capture_can_be_disabled_without_hiding_match_evidence() {
        let snapshot = env(&[
            ("TERM_PROGRAM", "ghostty"),
            ("UNRELATED_SECRET", "never captured"),
        ]);
        let options = DetectOptions {
            capture_env_subset: false,
            ..DetectOptions::default()
        };
        let info = detect_with_options(&snapshot, options);
        assert!(info.raw_env_subset.is_empty());
        assert_eq!(info.identifiers.len(), 1);
        assert!(
            !detect_from_env(&snapshot)
                .raw_env_subset
                .contains_key("UNRELATED_SECRET")
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_values_do_not_panic() {
        use std::os::unix::ffi::OsStringExt;
        let snapshot = EnvMap::from([(
            "TERM_PROGRAM".into(),
            std::ffi::OsString::from_vec(vec![0xff]),
        )]);
        let info = detect_from_env(&snapshot);
        assert_eq!(info.kind, TerminalKind::Unknown);
        assert_eq!(info.term_program.as_deref(), Some("\u{fffd}"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_environment_keys_ignore_ascii_case() {
        let info = detect_from_env(&env(&[("term_program", "ghostty")]));
        assert_eq!(info.kind, TerminalKind::Ghostty);
    }

    /// Count actual invocations so disabled and duplicate probes cannot pass unnoticed.
    #[derive(Default)]
    struct Runner {
        outputs: BTreeMap<String, CommandOutput>,
        calls: RefCell<Vec<String>>,
    }

    impl CommandRunner for Runner {
        fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
            let command = spec.as_command_string();
            self.calls.borrow_mut().push(command.clone());
            self.outputs
                .get(&command)
                .cloned()
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    #[test]
    fn default_detection_never_invokes_commands() {
        let runner = Runner::default();
        let snapshot = env(&[
            ("TMUX", "socket"),
            ("ZELLIJ", "1"),
            ("STY", "session"),
            ("TERM", "screen-256color"),
        ]);
        let info = detect_with_runner(&snapshot, DetectOptions::default(), &runner);
        assert!(runner.calls.borrow().is_empty());
        assert_eq!(info.multiplexer.unwrap().kind, MultiplexerKind::Tmux);
        assert_eq!(info.kind, TerminalKind::Unknown);
    }

    #[rstest]
    #[case("STY", MultiplexerKind::Screen, TerminalKind::Screen)]
    #[case("ZELLIJ", MultiplexerKind::Zellij, TerminalKind::Unknown)]
    #[case("TMUX", MultiplexerKind::Tmux, TerminalKind::Unknown)]
    fn separates_multiplexer_from_screen_emulation(
        #[case] marker: &str,
        #[case] mux: MultiplexerKind,
        #[case] terminal: TerminalKind,
    ) {
        let info = detect_from_env(&env(&[
            (marker, "session"),
            ("TERM", "screen.xterm-256color"),
        ]));
        assert_eq!(info.multiplexer.unwrap().kind, mux);
        assert_eq!(info.kind, terminal);
    }

    #[test]
    fn zellij_takes_precedence_over_screen() {
        let info = detect_from_env(&env(&[("ZELLIJ", "1"), ("STY", "session")]));
        assert_eq!(info.multiplexer.unwrap().kind, MultiplexerKind::Zellij);
    }

    #[test]
    fn reuses_tmux_client_probe_and_reports_command_evidence() {
        let mut runner = Runner::default();
        for (command, value) in [
            ("tmux -V", "tmux 3.5"),
            ("tmux display-message -p #{client_termname}", "xterm-kitty"),
            ("tmux display-message -p #{client_termtype}", "kitty"),
        ] {
            runner.outputs.insert(
                command.into(),
                CommandOutput {
                    stdout: value.into(),
                    stderr: String::new(),
                    status: Some(0),
                },
            );
        }
        let options = DetectOptions {
            allow_commands: true,
            ..DetectOptions::default()
        };
        let info = detect_with_runner(
            &env(&[("TMUX", "socket"), ("TERM", "screen-256color")]),
            options,
            &runner,
        );
        assert_eq!(info.kind, TerminalKind::Kitty);
        assert_eq!(info.multiplexer.unwrap().version.as_deref(), Some("3.5"));
        assert_eq!(runner.calls.borrow().len(), 3);
        assert_eq!(info.command_probes.len(), 3);
        assert!(info.identifiers.is_empty());
        assert_eq!(
            info.detected_via,
            vec![DetectionSource::Command(
                "tmux display-message -p #{client_termname}".into()
            )]
        );
    }

    #[rstest]
    #[case(Some(1))]
    #[case(None)]
    fn failed_probes_cannot_supply_metadata(#[case] status: Option<i32>) {
        let mut runner = Runner::default();
        runner.outputs.insert(
            "zellij --version".into(),
            CommandOutput {
                stdout: "zellij 1.2.3".into(),
                stderr: "error".into(),
                status,
            },
        );
        let options = DetectOptions {
            allow_commands: true,
            ..DetectOptions::default()
        };
        let info = detect_with_runner(&env(&[("ZELLIJ", "1")]), options, &runner);
        assert_eq!(info.multiplexer.unwrap().version, None);
        assert_eq!(info.command_probes[0].status, status);
        assert_eq!(info.command_probes[0].stderr, "error");
    }

    #[test]
    fn missing_executable_is_recorded_as_a_failed_attempt() {
        let options = DetectOptions {
            allow_commands: true,
            ..DetectOptions::default()
        };
        let info = detect_with_runner(&env(&[("STY", "session")]), options, &Runner::default());
        assert_eq!(info.multiplexer.unwrap().version, None);
        assert_eq!(info.command_probes.len(), 1);
        assert!(info.command_probes[0].error.is_some());
    }

    #[rstest]
    #[case(MultiplexerKind::Tmux, "tmux 3.3a", Some("3.3a"))]
    #[case(MultiplexerKind::Tmux, "tmux next-3.6", Some("next-3.6"))]
    #[case(MultiplexerKind::Tmux, "tmux master", Some("master"))]
    #[case(MultiplexerKind::Tmux, "error connecting to server", None)]
    #[case(MultiplexerKind::Tmux, "tmux ", None)]
    #[case(MultiplexerKind::Tmux, "tmux failed", None)]
    #[case(MultiplexerKind::Zellij, "zellij 0.40.0-dev", Some("0.40.0-dev"))]
    #[case(MultiplexerKind::Zellij, "error: command failed", None)]
    #[case(
        MultiplexerKind::Screen,
        "Screen version 4.08.00 (GNU) 05-Feb-20",
        Some("4.08.00")
    )]
    #[case(MultiplexerKind::Screen, "wrong version 3.0", None)]
    fn parses_only_recognized_version_banners(
        #[case] kind: MultiplexerKind,
        #[case] banner: &str,
        #[case] expected: Option<&str>,
    ) {
        assert_eq!(parse_version(kind, banner).as_deref(), expected);
    }
}
