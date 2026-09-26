#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, ValueEnum};
use detect_terminal::{DetectOptions, MultiplexerInfo};
use serde_json::{Value, json};

/// Available output encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

/// Inspect terminal and multiplexer hints from the process environment.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Output encoding.
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,

    /// Shorthand for --format json.
    #[arg(long, conflicts_with = "format")]
    json: bool,

    /// Run blocking multiplexer probes (no timeout; disabled by default).
    #[arg(long)]
    commands: bool,

    /// Omit the diagnostic environment map; matched identifiers remain visible.
    #[arg(long = "no-env")]
    no_env: bool,

    /// Indent JSON output; requires --json or --format json.
    #[arg(long)]
    pretty: bool,
}

/// Write one report and treat a closed downstream pipe as normal termination.
fn main() -> ExitCode {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };
    if cli.pretty && format != OutputFormat::Json {
        Cli::command()
            .error(
                clap::error::ErrorKind::InvalidValue,
                "--pretty requires --json or --format json",
            )
            .exit();
    }
    let env: BTreeMap<_, _> = std::env::vars_os().collect();
    let options = DetectOptions {
        allow_commands: cli.commands,
        capture_env_subset: !cli.no_env,
    };
    let info = detect_terminal::detect_with_options(&env, options);
    let mut stdout = io::stdout().lock();
    let result = match format {
        OutputFormat::Human => print_human(&mut stdout, &info),
        OutputFormat::Json => print_json(&mut stdout, &info, cli.pretty),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("could not write terminal report: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Render a report to any writer so output failures remain recoverable.
fn print_human(writer: &mut impl Write, info: &detect_terminal::TerminalInfo) -> io::Result<()> {
    writeln!(writer, "terminal: {}", info.kind)?;
    if let Some(version) = &info.version {
        writeln!(writer, "version: {version}")?;
    }
    if let Some(raw) = &info.raw_name {
        writeln!(writer, "raw name: {raw}")?;
    }
    if let Some(term_program) = &info.term_program {
        writeln!(writer, "term program: {term_program}")?;
    }
    if let Some(term_program_version) = &info.term_program_version {
        writeln!(writer, "term program version: {term_program_version}")?;
    }
    if let Some(term) = &info.term {
        writeln!(writer, "term: {term}")?;
    }

    if let Some(mux) = &info.multiplexer {
        writeln!(writer, "multiplexer: {}", mux.kind)?;
        if let Some(version) = &mux.version {
            writeln!(writer, "mux version: {version}")?;
        }
        if let Some(client_term) = &mux.client_term {
            writeln!(writer, "tmux client term: {client_term}")?;
        }
        if let Some(client_type) = &mux.client_type {
            writeln!(writer, "tmux client type: {client_type}")?;
        }
    }

    if !info.detected_via.is_empty() {
        writeln!(writer, "detected via:")?;
        for source in &info.detected_via {
            writeln!(writer, "- {source}")?;
        }
    }

    if !info.identifiers.is_empty() {
        writeln!(writer, "identifiers:")?;
        for identifier in &info.identifiers {
            writeln!(writer, "- {}={}", identifier.key, identifier.value)?;
        }
    }

    if !info.raw_env_subset.is_empty() {
        writeln!(writer, "env subset:")?;
        for (key, value) in &info.raw_env_subset {
            writeln!(writer, "- {key}={value}")?;
        }
    }

    if !info.command_probes.is_empty() {
        writeln!(writer, "command probes:")?;
        for probe in &info.command_probes {
            let status = probe
                .status
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            writeln!(writer, "- {} (status: {})", probe.command, status)?;
            if !probe.stdout.is_empty() {
                writeln!(writer, "  stdout: {}", probe.stdout)?;
            }
            if let Some(error) = &probe.error {
                writeln!(writer, "  error: {error}")?;
            }
            if !probe.stderr.is_empty() {
                writeln!(writer, "  stderr: {}", probe.stderr)?;
            }
        }
    }
    Ok(())
}

/// Serialize a diagnostic report; display names are used for terminal and mux kinds.
fn print_json(
    writer: &mut impl Write,
    info: &detect_terminal::TerminalInfo,
    pretty: bool,
) -> io::Result<()> {
    let mux = info
        .multiplexer
        .as_ref()
        .map(multiplexer_value)
        .unwrap_or(Value::Null);

    let detected_via: Vec<Value> = info
        .detected_via
        .iter()
        .map(|source| Value::String(source.to_string()))
        .collect();

    let identifiers: Vec<Value> = info
        .identifiers
        .iter()
        .map(|identifier| json!({"key": identifier.key, "value": identifier.value}))
        .collect();

    let raw_env_subset: Value = info
        .raw_env_subset
        .iter()
        .map(|(key, value)| (key.clone(), Value::String(value.clone())))
        .collect::<serde_json::Map<_, _>>()
        .into();

    let command_probes: Vec<Value> = info
        .command_probes
        .iter()
        .map(|probe| {
            json!({
                "command": probe.command,
                "stdout": probe.stdout,
                "stderr": probe.stderr,
                "status": probe.status,
                "error": probe.error
            })
        })
        .collect();

    let value = json!({
        "schema_version": 1,
        "kind": info.kind.to_string(),
        "version": info.version,
        "term_program": info.term_program,
        "term_program_version": info.term_program_version,
        "term": info.term,
        "raw_name": info.raw_name,
        "multiplexer": mux,
        "detected_via": detected_via,
        "identifiers": identifiers,
        "raw_env_subset": raw_env_subset,
        "command_probes": command_probes,
    });

    let result = if pretty {
        serde_json::to_writer_pretty(&mut *writer, &value)
    } else {
        serde_json::to_writer(&mut *writer, &value)
    };
    result.map_err(|error| {
        io::Error::new(error.io_error_kind().unwrap_or(io::ErrorKind::Other), error)
    })?;
    writeln!(writer)
}

/// Keep absent multiplexer metadata explicit as JSON nulls.
fn multiplexer_value(mux: &MultiplexerInfo) -> Value {
    json!({
        "kind": mux.kind.to_string(),
        "version": mux.version,
        "client_term": mux.client_term,
        "client_type": mux.client_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A downstream consumer that has closed its read end.
    struct ClosedPipe;

    impl Write for ClosedPipe {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn renderers_preserve_broken_pipe_errors() {
        let info = detect_terminal::detect_from_env(&BTreeMap::new());
        assert_eq!(
            print_human(&mut ClosedPipe, &info).unwrap_err().kind(),
            io::ErrorKind::BrokenPipe
        );
        assert_eq!(
            print_json(&mut ClosedPipe, &info, false)
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
