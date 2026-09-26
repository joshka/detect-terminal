use std::collections::BTreeMap;
use std::ffi::OsString;

/// Environment map used by the detector.
///
/// This is the input type consumed by [`detect_from_env`] and
/// [`detect_with_options`]. Keys and values are stored as `OsString` to match
/// how the OS represents environment variables, and callers can construct their own detection
/// inputs. A `BTreeMap` is used so iteration is stable for debugging and tests.
///
/// Lookup is case-sensitive on Unix and ASCII case-insensitive on Windows. A synthetic Windows
/// snapshot with duplicate spellings prefers an exact match. Diagnostic keys use the detector's
/// canonical spelling. Values remain OS strings here, but detection results decode them lossily.
///
/// # Example
///
/// ```rust
/// use detect_terminal::EnvMap;
///
/// let env: EnvMap = std::env::vars_os().collect();
/// let info = detect_terminal::detect_from_env(&env);
/// println!("Terminal: {}", info.kind);
/// ```
///
/// [`detect_from_env`]: crate::detect_from_env
/// [`detect_with_options`]: crate::detect_with_options
/// [`EnvMap`]: crate::EnvMap
/// [`TerminalInfo::raw_env_subset`]: crate::TerminalInfo::raw_env_subset
pub type EnvMap = BTreeMap<OsString, OsString>;

/// Read-only view of the environment with optional capture of accessed keys.
///
/// This is used by the detector to record the specific variables read while
/// still performing lookups from the original map. Captured keys become
/// [`TerminalInfo::raw_env_subset`] so callers can audit which markers were
/// queried.
///
/// [`TerminalInfo::raw_env_subset`]: crate::TerminalInfo::raw_env_subset
pub(crate) struct EnvView<'a> {
    env: &'a EnvMap,
    capture: bool,
    used: BTreeMap<String, String>,
}

impl<'a> EnvView<'a> {
    /// Build a view over the provided environment map.
    ///
    /// When `capture` is true, any lookup via `get` is stored
    /// in the internal `used` map for later export.
    pub(crate) fn new(env: &'a EnvMap, capture: bool) -> Self {
        Self {
            env,
            capture,
            used: BTreeMap::new(),
        }
    }

    /// Read a key from the environment, recording it if capture is enabled.
    ///
    /// Returns a UTF-8 lossy string for non-Unicode values to keep downstream
    /// detection simple. Returns `None` when the key is absent.
    pub(crate) fn get(&mut self, key: &str) -> Option<String> {
        let value = self.env.get(std::ffi::OsStr::new(key));
        // Windows treats environment names case-insensitively; prefer the exact spelling when
        // a synthetic snapshot contains multiple spellings of the same name.
        #[cfg(windows)]
        let value = value.or_else(|| {
            self.env.iter().find_map(|(candidate, value)| {
                candidate
                    .to_str()
                    .filter(|candidate| candidate.eq_ignore_ascii_case(key))
                    .map(|_| value)
            })
        });
        let value = value.map(|value| value.to_string_lossy().into_owned());
        if self.capture
            && let Some(value) = value.clone()
        {
            self.used.insert(key.to_string(), value);
        }
        value
    }

    /// Return the captured subset of environment variables.
    ///
    /// The map includes only keys accessed via `get`.
    pub(crate) fn into_used(self) -> BTreeMap<String, String> {
        self.used
    }
}

/// Build an environment map from the current process.
///
/// This collects [`std::env::vars_os`] into the [`EnvMap`] used by the detector.
pub(crate) fn env_map_from_os() -> EnvMap {
    std::env::vars_os().collect()
}
