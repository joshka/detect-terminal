use std::collections::BTreeMap;
use std::ffi::OsString;

/// Environment map used by the detector.
///
/// This is the input type consumed by [`detect_from_env`] and
/// [`detect_with_options`].
///
/// # Example
///
/// ```rust
/// use detect_terminal::EnvMap;
///
/// let env: EnvMap = std::env::vars_os().collect();
/// ```
///
/// [`detect_from_env`]: crate::detect_from_env
/// [`detect_with_options`]: crate::detect_with_options
pub type EnvMap = BTreeMap<OsString, OsString>;

/// Read-only view of the environment with optional capture of accessed keys.
///
/// This is used by the detector to record the specific variables read while
/// still performing lookups from the original map. Captured keys become
/// `TerminalInfo::raw_env_subset`.
pub(crate) struct EnvView<'a> {
    env: &'a EnvMap,
    capture: bool,
    used: BTreeMap<String, String>,
}

impl<'a> EnvView<'a> {
    /// Build a view over the provided environment map.
    ///
    /// When `capture` is true, any lookup via `get` or `contains_key` is stored
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
    /// detection simple.
    pub(crate) fn get(&mut self, key: &str) -> Option<String> {
        let value = self
            .env
            .get(&OsString::from(key))
            .map(|value| value.to_string_lossy().to_string());
        if self.capture
            && let Some(value) = value.clone()
        {
            self.used.insert(key.to_string(), value);
        }
        value
    }

    /// Check whether the environment contains a key, with capture semantics.
    ///
    /// This calls `get`, so it captures the key if capture is enabled.
    pub(crate) fn contains_key(&mut self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Return the captured subset of environment variables.
    ///
    /// The map includes only keys accessed via `get` or `contains_key`.
    pub(crate) fn into_used(self) -> BTreeMap<String, String> {
        self.used
    }
}

/// Build an environment map from the current process.
///
/// This collects `std::env::vars_os` into the `EnvMap` used by the detector.
pub(crate) fn env_map_from_os() -> EnvMap {
    std::env::vars_os().collect()
}
