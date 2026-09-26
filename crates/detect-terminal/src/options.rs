/// Controls optional command probes and diagnostic environment capture.
///
/// Defaults perform environment-only detection and capture the variables consulted by the
/// detector. Command probes are opt-in, synchronous, and have no timeout.
/// `capture_env_subset` controls the diagnostic map; matched identifiers and raw terminal names
/// remain in the result regardless of this setting.
///
/// # Example
///
/// ```no_run
/// use detect_terminal::{DetectOptions, detect_with_options};
///
/// let env = std::env::vars_os().collect();
/// let options = DetectOptions {
///     allow_commands: true,
///     ..DetectOptions::default()
/// };
/// let info = detect_with_options(&env, options);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectOptions {
    /// Run tmux, screen, or zellij probes using the supplied environment. Defaults to `false`.
    pub allow_commands: bool,

    /// Retain accessed environment values in the diagnostic map. Defaults to `true`.
    pub capture_env_subset: bool,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            allow_commands: false,
            capture_env_subset: true,
        }
    }
}
