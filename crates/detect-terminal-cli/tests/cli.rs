//! Exercise the installed command's arguments, environment isolation, and output contract.

use std::process::{Command, Output};

/// Launch the CLI with a deliberately small and predictable environment.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_detect-terminal"))
        .args(args)
        .env_clear()
        .env("TERM_PROGRAM", "ghostty")
        .env("TERM_PROGRAM_VERSION", "1.2.3")
        .env("TERM", "xterm-ghostty")
        .env("TMUX", "synthetic-session")
        .output()
        .unwrap()
}

#[test]
fn json_is_complete_and_does_not_probe_by_default() {
    let output = run(&["--json"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["kind"], "Ghostty");
    assert_eq!(value["version"], "1.2.3");
    assert_eq!(value["multiplexer"]["kind"], "tmux");
    assert_eq!(value["command_probes"], serde_json::json!([]));
    assert!(value["raw_env_subset"].get("TERM_PROGRAM").is_some());
    assert!(output.stderr.is_empty());
}

#[test]
fn pretty_and_compact_json_describe_the_same_result() {
    let compact = run(&["--json"]);
    let pretty = run(&["--format", "json", "--pretty"]);
    assert!(pretty.status.success());
    let compact: serde_json::Value = serde_json::from_slice(&compact.stdout).unwrap();
    let pretty_value: serde_json::Value = serde_json::from_slice(&pretty.stdout).unwrap();
    assert_eq!(compact, pretty_value);
    assert!(pretty.stdout.contains(&b'\n'));
}

#[test]
fn no_env_omits_only_the_diagnostic_map() {
    let output = run(&["--json", "--no-env"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["raw_env_subset"], serde_json::json!({}));
    assert_eq!(value["identifiers"][0]["key"], "TERM_PROGRAM");
}

#[test]
fn human_output_has_identity_and_mux_metadata() {
    let output = run(&[]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("terminal: Ghostty\n"));
    assert!(text.contains("multiplexer: tmux\n"));
    assert!(text.contains("version: 1.2.3\n"));
}

#[test]
fn rejects_conflicting_or_ineffective_options() {
    for args in [
        &["--pretty"][..],
        &["--json", "--format", "human"],
        &["--format", "yaml"],
    ] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2));
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn help_explains_command_side_effects() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("--commands"));
    assert!(help.contains("no timeout"));
    assert!(help.contains("matched identifiers remain visible"));
}

#[test]
fn human_output_escapes_environment_controls_but_json_preserves_values() {
    let raw = "custom\nterminal: FORGED\r\t\u{1b}[31m café";
    for args in [&[][..], &["--json"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_detect-terminal"))
            .args(args)
            .env_clear()
            .env("TERM_PROGRAM", raw)
            .env("TERM_PROGRAM_VERSION", raw)
            .env("TERM", raw)
            .env("KITTY_WINDOW_ID", raw)
            .output()
            .unwrap();
        assert!(output.status.success());
        if args.is_empty() {
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(!text.contains("\nterminal: FORGED"));
            assert!(!text.chars().any(|c| c.is_control() && c != '\n'));
            assert!(text.contains(&raw.escape_debug().to_string()));
            assert!(text.contains("café"));
        } else {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["term_program"], raw);
            assert_eq!(value["identifiers"][0]["value"], raw);
        }
    }
}
