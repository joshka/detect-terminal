/// Options controlling command probing and environment capture.
///
/// This struct feeds into [`detect_with_options`] to control the extra debug
/// output and whether the detector shells out for tmux/screen/zellij metadata.
/// Defaults enable command probing and env capture so `detect()` returns richer
/// multiplexer details and [`TerminalInfo::raw_env_subset`] is populated.
/// Fields default to `true`.
/// Use `allow_commands = false` for latency-sensitive or sandboxed contexts.
/// Use `capture_env_subset = false` to avoid retaining sensitive values.
///
/// # Example
///
/// ```rust
/// use detect_terminal::{detect_with_options, DetectOptions};
///
/// let env = std::env::vars_os().collect();
/// let options = DetectOptions {
///     allow_commands: false,
///     capture_env_subset: true,
/// };
/// let info = detect_with_options(&env, options);
/// ```
///
/// [`detect_with_options`]: crate::detect_with_options
/// [`TerminalInfo::raw_env_subset`]: crate::TerminalInfo::raw_env_subset
#[derive(Debug, Clone)]
pub struct DetectOptions {
    /// Allow running tmux/screen/zellij commands for extra metadata.
    ///
    /// When disabled, the detector only uses environment variables and skips
    /// command probes like `tmux -V` or `zellij --version`.
    ///
    /// Disabling commands can reduce latency in hot paths.
    pub allow_commands: bool,
    /// Capture only the environment variables read during detection.
    ///
    /// When enabled, the [`TerminalInfo::raw_env_subset`] map contains
    /// only the keys accessed by the detector.
    ///
    /// [`TerminalInfo::raw_env_subset`]: crate::TerminalInfo::raw_env_subset
    pub capture_env_subset: bool,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            allow_commands: true,
            capture_env_subset: true,
        }
    }
}
