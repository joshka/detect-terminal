use std::process::Command;

use crate::CommandProbe;

/// Runner abstraction for executing command probes.
///
/// This allows tests to inject deterministic command results while keeping the
/// production code path simple. Errors bubble up and are treated as missing
/// metadata by the detector.
pub(crate) trait CommandRunner {
    fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput>;
}

/// [`CommandProbe::command`]: crate::CommandProbe::command
/// [`CommandSpec::as_command_string`]: crate::command::CommandSpec::as_command_string
/// [`DetectOptions::allow_commands`]: crate::DetectOptions::allow_commands
/// [`TerminalInfo::command_probes`]: crate::TerminalInfo::command_probes
/// [`detect_with_options`]: crate::detect_with_options
/// Specification for a command probe invocation.
///
/// Commands are recorded in [`CommandProbe::command`] using
/// [`CommandSpec::as_command_string`].
#[derive(Debug, Clone)]
pub(crate) struct CommandSpec {
    pub(crate) program: &'static str,
    pub(crate) args: Vec<&'static str>,
}

impl CommandSpec {
    /// Render the command as a single string for audit logs.
    ///
    /// Arguments are joined with spaces and are not shell-escaped.
    pub(crate) fn as_command_string(&self) -> String {
        let mut command = String::from(self.program);
        for arg in &self.args {
            command.push(' ');
            command.push_str(arg);
        }
        command
    }
}

/// Captured output of a command probe.
///
/// Output is stored as trimmed UTF-8 lossy strings to avoid allocation
/// surprises when command output is not valid UTF-8.
#[derive(Debug, Clone)]
pub(crate) struct CommandOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) status: Option<i32>,
}

/// Default command runner that shells out for mux probes.
///
/// This is used by [`detect_with_options`] when
/// [`DetectOptions::allow_commands`] is enabled.
pub(crate) struct DefaultCommandRunner;

impl CommandRunner for DefaultCommandRunner {
    fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
        let output = Command::new(spec.program).args(&spec.args).output()?;
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            status: output.status.code(),
        })
    }
}

/// Run a version command and parse its output using the supplied parser.
///
/// Returns `None` when the command fails or the parser does not recognize the
/// output. The parser handles terminal/multiplexer-specific output formats.
pub(crate) fn run_version_command<F>(
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
    spec: CommandSpec,
    parser: F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let output = run_command(runner, probes, &spec)?;
    parser(&output)
}

/// Execute a command and record the probe output.
///
/// The captured output is appended to [`TerminalInfo::command_probes`]. The
/// returned string prefers stdout, then stderr, and returns `None` if both are
/// empty or the command fails.
pub(crate) fn run_command(
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
    spec: &CommandSpec,
) -> Option<String> {
    let output = runner.run(spec).ok()?;
    probes.push(CommandProbe {
        command: spec.as_command_string(),
        stdout: output.stdout.clone(),
        stderr: output.stderr.clone(),
        status: output.status,
    });
    if !output.stdout.is_empty() {
        Some(output.stdout)
    } else if !output.stderr.is_empty() {
        Some(output.stderr)
    } else {
        None
    }
}

/// Run `tmux display-message -p` for a specific format string.
///
/// This is used to fetch tmux client term metadata. Returns `None` if the
/// command fails or produces no output.
///
/// Add new probe helpers in this module so probe output is consistently
/// recorded in [`TerminalInfo::command_probes`].
pub(crate) fn tmux_display_message(
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
    format: &'static str,
) -> Option<String> {
    let spec = CommandSpec {
        program: "tmux",
        args: vec!["display-message", "-p", format],
    };
    run_command(runner, probes, &spec)
}
