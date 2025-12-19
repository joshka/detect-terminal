use clap::{Parser, ValueEnum};
use detect_terminal::{DetectOptions, MultiplexerInfo};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(long, value_enum, default_value = "human")]
    format: OutputFormat,

    #[arg(long)]
    json: bool,

    #[arg(long = "no-commands")]
    no_commands: bool,

    #[arg(long = "no-env")]
    no_env: bool,

    #[arg(long)]
    pretty: bool,
}

fn main() {
    let cli = Cli::parse();
    let env: BTreeMap<_, _> = std::env::vars_os().collect();
    let options = DetectOptions {
        allow_commands: !cli.no_commands,
        capture_env_subset: !cli.no_env,
    };

    let info = detect_terminal::detect_with_options(&env, options);

    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format
    };

    match format {
        OutputFormat::Human => print_human(&info),
        OutputFormat::Json => print_json(&info, cli.pretty),
    }
}

fn print_human(info: &detect_terminal::TerminalInfo) {
    println!("terminal: {}", info.kind);
    if let Some(version) = &info.version {
        println!("version: {version}");
    }
    if let Some(raw) = &info.raw_name {
        println!("raw name: {raw}");
    }
    if let Some(term_program) = &info.term_program {
        println!("term program: {term_program}");
    }
    if let Some(term_program_version) = &info.term_program_version {
        println!("term program version: {term_program_version}");
    }
    if let Some(term) = &info.term {
        println!("term: {term}");
    }

    if let Some(mux) = &info.multiplexer {
        println!("multiplexer: {}", mux.kind);
        if let Some(version) = &mux.version {
            println!("mux version: {version}");
        }
        if let Some(client_term) = &mux.client_term {
            println!("tmux client term: {client_term}");
        }
        if let Some(client_type) = &mux.client_type {
            println!("tmux client type: {client_type}");
        }
    }

    if !info.detected_via.is_empty() {
        println!("detected via:");
        for source in &info.detected_via {
            println!("- {}", source);
        }
    }

    if !info.identifiers.is_empty() {
        println!("identifiers:");
        for identifier in &info.identifiers {
            println!("- {}={}", identifier.key, identifier.value);
        }
    }

    if !info.raw_env_subset.is_empty() {
        println!("env subset:");
        for (key, value) in &info.raw_env_subset {
            println!("- {}={}", key, value);
        }
    }

    if !info.command_probes.is_empty() {
        println!("command probes:");
        for probe in &info.command_probes {
            let status = probe
                .status
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            println!("- {} (status: {})", probe.command, status);
            if !probe.stdout.is_empty() {
                println!("  stdout: {}", probe.stdout);
            }
            if !probe.stderr.is_empty() {
                println!("  stderr: {}", probe.stderr);
            }
        }
    }
}

fn print_json(info: &detect_terminal::TerminalInfo, pretty: bool) {
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
                "status": probe.status
            })
        })
        .collect();

    let value = json!({
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

    if pretty {
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    } else {
        println!("{}", serde_json::to_string(&value).unwrap());
    }
}

fn multiplexer_value(mux: &MultiplexerInfo) -> Value {
    json!({
        "kind": mux.kind.to_string(),
        "version": mux.version,
        "client_term": mux.client_term,
        "client_type": mux.client_type,
    })
}
