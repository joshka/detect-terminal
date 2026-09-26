//! Multiplexer command execution, diagnostics, and version parsing.

use std::process::Command;

use crate::{CommandProbe, EnvMap, MultiplexerInfo, MultiplexerKind};

/// Execute a probe; tests substitute recorded output without spawning processes.
pub(crate) trait CommandRunner {
    fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput>;
}

/// A fixed executable and arguments, passed directly without a shell.
#[derive(Debug)]
pub(crate) struct CommandSpec {
    pub(crate) program: &'static str,
    pub(crate) args: &'static [&'static str],
}

impl CommandSpec {
    /// Human-readable command identity; this is not shell-escaped executable text.
    pub(crate) fn as_command_string(&self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Trimmed, lossily decoded process output and its exit code.
#[derive(Debug, Clone)]
pub(crate) struct CommandOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) status: Option<i32>,
}

/// Run probes using the same snapshot that selected the multiplexer.
pub(crate) struct DefaultCommandRunner<'a> {
    pub(crate) env: &'a EnvMap,
}

impl CommandRunner for DefaultCommandRunner<'_> {
    fn run(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
        let output = Command::new(spec.program)
            .args(spec.args)
            .env_clear()
            .envs(self.env)
            .output()?;
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).trim().into(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().into(),
            status: output.status.code(),
        })
    }
}

/// Query each optional field once, retaining failed attempts as diagnostics.
pub(crate) fn probe_multiplexer(
    mux: &mut MultiplexerInfo,
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
) {
    let spec = match mux.kind {
        MultiplexerKind::Tmux => CommandSpec {
            program: "tmux",
            args: &["-V"],
        },
        MultiplexerKind::Screen => CommandSpec {
            program: "screen",
            args: &["--version"],
        },
        MultiplexerKind::Zellij => CommandSpec {
            program: "zellij",
            args: &["--version"],
        },
    };
    mux.version =
        run_command(runner, probes, &spec).and_then(|output| parse_version(mux.kind, &output));
    if mux.kind == MultiplexerKind::Tmux {
        let term = CommandSpec {
            program: "tmux",
            args: &["display-message", "-p", "#{client_termname}"],
        };
        mux.client_term = run_command(runner, probes, &term);
        let features = CommandSpec {
            program: "tmux",
            args: &["display-message", "-p", "#{client_termtype}"],
        };
        mux.client_type = run_command(runner, probes, &features);
    }
}

/// Only successful commands can supply metadata; stderr remains available for diagnostics.
fn run_command(
    runner: &dyn CommandRunner,
    probes: &mut Vec<CommandProbe>,
    spec: &CommandSpec,
) -> Option<String> {
    match runner.run(spec) {
        Ok(output) => {
            probes.push(CommandProbe {
                command: spec.as_command_string(),
                stdout: output.stdout.clone(),
                stderr: output.stderr,
                status: output.status,
                error: None,
            });
            (output.status == Some(0) && !output.stdout.is_empty()).then_some(output.stdout)
        }
        Err(error) => {
            probes.push(CommandProbe {
                command: spec.as_command_string(),
                stdout: String::new(),
                stderr: String::new(),
                status: None,
                error: Some(error.to_string()),
            });
            None
        }
    }
}

/// Accept known version banners and reject arbitrary error text or missing version tokens.
pub(crate) fn parse_version(kind: MultiplexerKind, output: &str) -> Option<String> {
    let prefix = match kind {
        MultiplexerKind::Tmux => "tmux ",
        MultiplexerKind::Screen => "Screen version ",
        MultiplexerKind::Zellij => "zellij ",
    };
    let version = output
        .trim()
        .strip_prefix(prefix)?
        .split_whitespace()
        .next()?;
    // tmux development builds report "next-<version>" or "master".
    let recognized = version.starts_with(|c: char| c.is_ascii_digit())
        || (kind == MultiplexerKind::Tmux && (version == "master" || version.starts_with("next-")));
    recognized.then(|| version.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn runner_uses_snapshot_environment() {
        let env = EnvMap::from([("DETECT_PROBE_TEST".into(), "from snapshot".into())]);
        let runner = DefaultCommandRunner { env: &env };
        let spec = CommandSpec {
            program: "/bin/sh",
            args: &["-c", "printf '%s' \"$DETECT_PROBE_TEST:${HOME-unset}\""],
        };
        let output = runner.run(&spec).unwrap();
        assert_eq!(output.stdout, "from snapshot:unset");
        assert_eq!(output.status, Some(0));
    }
}
