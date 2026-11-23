//! Color support inference. See [Color support](crate#color-support) for its limits.

use std::fmt;

use crate::{DetectionSource, MultiplexerKind, TerminalInfo, TerminalKind};

/// First Apple Terminal build reported to render RGB sequences, from macOS 26.0 (Tahoe).
///
/// No public release is known to use builds 456 through 463. Treating them as 256-color means that
/// at worst an early Tahoe beta uses indexed colors, rather than an older build garbling RGB.
const APPLE_FIRST_TRUECOLOR_BUILD: u32 = 464;

/// First iTerm2 major version with 24-bit color.
const ITERM2_FIRST_TRUECOLOR_MAJOR: u32 = 3;

/// First tmux release that identifies itself and its version to panes.
///
/// Earlier releases can also accept RGB, but their environment does not establish the version.
const TMUX_FIRST_IDENTIFIED: (u32, u32) = (3, 2);

/// Color encodings a terminal is inferred to support, not a measured capability.
///
/// [`Unknown`](Self::Unknown) means there is not enough evidence. It does not mean that color is
/// unsupported, so each application should choose its own default. The enum is non-exhaustive.
///
/// User preferences such as `NO_COLOR` and whether output is a TTY do not affect this value.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSupport {
    /// The available hints do not establish color support.
    Unknown,
    /// The 16 ANSI colors. No current rule selects this variant.
    Ansi16,
    /// The 256-color indexed palette, without reliable RGB support.
    Ansi256,
    /// 24-bit RGB colors are accepted, though tmux can convert them for its outer terminal.
    TrueColor,
}

impl fmt::Display for ColorSupport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unknown => "unknown",
            Self::Ansi16 => "ansi16",
            Self::Ansi256 => "ansi256",
            Self::TrueColor => "truecolor",
        })
    }
}

/// Inferred color support and its evidence, separate from terminal identity evidence.
///
/// See [Color support](crate#color-support) for the rules.
///
/// # Example
///
/// ```
/// use detect_terminal::{ColorSupport, EnvMap, detect_from_env};
///
/// let env = EnvMap::from([
///     ("TERM_PROGRAM".into(), "Apple_Terminal".into()),
///     ("TERM_PROGRAM_VERSION".into(), "455.1".into()),
/// ]);
/// assert_eq!(detect_from_env(&env).color.support, ColorSupport::Ansi256);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorInfo {
    /// Inferred color support.
    pub support: ColorSupport,

    /// Evidence for the inference, empty when support is unknown.
    ///
    /// The raw values are in [`TerminalInfo`], including [`TerminalInfo::colorterm`].
    pub inferred_via: Vec<DetectionSource>,
}

impl ColorInfo {
    /// Start without a claim about color support.
    pub(crate) fn unknown() -> Self {
        Self::from_env(ColorSupport::Unknown, &[])
    }

    /// Keep the inference and the variables that support it together.
    fn from_env(support: ColorSupport, keys: &[&str]) -> Self {
        let inferred_via = keys
            .iter()
            .map(|key| DetectionSource::EnvVar((*key).into()))
            .collect();
        Self {
            support,
            inferred_via,
        }
    }
}

/// Infer color support after terminal identity and raw hints have been assigned.
pub(crate) fn infer_color(info: &TerminalInfo) -> ColorInfo {
    if tmux_accepts_truecolor(info) {
        let keys = ["TMUX", "TERM_PROGRAM", "TERM_PROGRAM_VERSION"];
        return ColorInfo::from_env(ColorSupport::TrueColor, &keys);
    }
    if hints_may_describe_another_terminal(info) {
        return ColorInfo::unknown();
    }

    let is_apple = info.kind == TerminalKind::AppleTerminal;
    let major = info.version.as_deref().and_then(leading_integer);
    if is_apple && let Some(build) = major {
        let keys = ["TERM_PROGRAM", "TERM_PROGRAM_VERSION"];
        let support = if build < APPLE_FIRST_TRUECOLOR_BUILD {
            ColorSupport::Ansi256
        } else {
            ColorSupport::TrueColor
        };
        return ColorInfo::from_env(support, &keys);
    }

    if info.kind == TerminalKind::WindowsTerminal {
        return ColorInfo {
            support: ColorSupport::TrueColor,
            inferred_via: info.detected_via.clone(),
        };
    }
    if info.kind == TerminalKind::ITerm2
        && major.is_some_and(|major| major >= ITERM2_FIRST_TRUECOLOR_MAJOR)
    {
        let keys = ["TERM_PROGRAM", "TERM_PROGRAM_VERSION"];
        return ColorInfo::from_env(ColorSupport::TrueColor, &keys);
    }

    let advertises_truecolor = info.colorterm.as_deref().is_some_and(|value| {
        value.eq_ignore_ascii_case("truecolor") || value.eq_ignore_ascii_case("24bit")
    });
    if advertises_truecolor {
        return ColorInfo::from_env(ColorSupport::TrueColor, &["COLORTERM"]);
    }

    // An Apple Terminal of unknown age cannot be assumed to honor a configured `TERM`.
    let term = info.term.as_deref().unwrap_or_default();
    if !is_apple && term.to_ascii_lowercase().ends_with("-direct") {
        return ColorInfo::from_env(ColorSupport::TrueColor, &["TERM"]);
    }
    ColorInfo::unknown()
}

/// Whether the detected tmux supplied the program hints and is known to accept RGB.
///
/// A terminal launched from a tmux pane can inherit `TMUX`, but replaces `TERM_PROGRAM` if it sets
/// one. Requiring tmux's own value also ties the version to tmux.
fn tmux_accepts_truecolor(info: &TerminalInfo) -> bool {
    let Some(mux) = &info.multiplexer else {
        return false;
    };
    // A version known only from a probe may belong to a tmux that does not own this environment.
    let version_is_from_env = info.term_program.as_deref() == Some("tmux")
        && info
            .term_program_version
            .as_deref()
            .is_some_and(|version| !version.is_empty());
    let version = mux.version.as_deref().and_then(tmux_version);
    mux.kind == MultiplexerKind::Tmux
        && version_is_from_env
        && info.term.as_deref() != Some("dumb")
        && version.is_some_and(|version| version >= TMUX_FIRST_IDENTIFIED)
}

/// Parse a tmux release such as `3.6a`; development versions such as `next-3.7` are not releases.
fn tmux_version(version: &str) -> Option<(u32, u32)> {
    let (major, minor) = version.split_once('.')?;
    let minor = minor.trim_end_matches(|c: char| c.is_ascii_lowercase());
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Multiplexers, their emulation families, and `TERM=dumb` sessions can inherit outer hints.
///
/// `TERM` is checked separately because it names emulation, so it does not select a multiplexer.
fn hints_may_describe_another_terminal(info: &TerminalInfo) -> bool {
    let term = info.term.as_deref().unwrap_or_default();
    let term = term.to_ascii_lowercase();
    let family = term.split(['-', '.']).next().unwrap_or_default();
    info.multiplexer.is_some() || matches!(family, "tmux" | "screen" | "dumb")
}

/// Parse the part of a dotted version before its first dot.
fn leading_integer(version: &str) -> Option<u32> {
    version.split('.').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::{DetectOptions, EnvMap, detect_from_env, detect_with_options};

    /// Detect color from a controlled environment.
    fn color(pairs: &[(&str, &str)]) -> ColorInfo {
        let env: EnvMap = pairs
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect();
        detect_from_env(&env).color
    }

    #[rstest]
    #[case::old("447.1", ColorSupport::Ansi256)]
    #[case::macos_15("455.1", ColorSupport::Ansi256)]
    #[case::below_cutoff("463.9", ColorSupport::Ansi256)]
    #[case::macos_26_0("464", ColorSupport::TrueColor)]
    #[case::macos_26_1("465", ColorSupport::TrueColor)]
    #[case::macos_26_2("466", ColorSupport::TrueColor)]
    #[case::dotted("470.2", ColorSupport::TrueColor)]
    fn known_apple_builds_do_not_depend_on_colorterm(
        #[case] version: &str,
        #[case] support: ColorSupport,
        #[values(None, Some(""), Some("truecolor"))] colorterm: Option<&str>,
    ) {
        let mut pairs = vec![
            ("TERM_PROGRAM", "Apple_Terminal"),
            ("TERM_PROGRAM_VERSION", version),
            ("TERM", "xterm-256color"),
        ];
        pairs.extend(colorterm.map(|value| ("COLORTERM", value)));
        let expected = ColorInfo::from_env(support, &["TERM_PROGRAM", "TERM_PROGRAM_VERSION"]);
        assert_eq!(color(&pairs), expected);
    }

    #[rstest]
    fn unparsable_apple_builds_rely_on_colorterm(
        #[values("", "unknown", "470beta", "99999999999")] version: &str,
    ) {
        let pairs = [
            ("TERM_PROGRAM", "Apple_Terminal"),
            ("TERM_PROGRAM_VERSION", version),
            ("TERM", "xterm-direct"),
        ];
        assert_eq!(color(&pairs), ColorInfo::unknown());

        let mut pairs = pairs.to_vec();
        pairs.push(("COLORTERM", "truecolor"));
        let expected = ColorInfo::from_env(ColorSupport::TrueColor, &["COLORTERM"]);
        assert_eq!(color(&pairs), expected);
    }

    #[test]
    fn apple_without_a_version_is_unknown() {
        let pairs = [("TERM_PROGRAM", "Apple_Terminal")];
        assert_eq!(color(&pairs), ColorInfo::unknown());
    }

    #[rstest]
    #[case("COLORTERM", "truecolor")]
    #[case("COLORTERM", "24BIT")]
    #[case("TERM", "xterm-direct")]
    fn generic_truecolor_hints(#[case] key: &str, #[case] value: &str) {
        let expected = ColorInfo::from_env(ColorSupport::TrueColor, &[key]);
        assert_eq!(color(&[(key, value)]), expected);
    }

    #[rstest]
    #[case("COLORTERM", "")]
    #[case("COLORTERM", "1")]
    #[case("COLORTERM", "not-truecolor")]
    #[case("TERM", "xterm-256color")]
    #[case("TERM", "xterm-indirect")]
    fn weak_hints_are_unknown(#[case] key: &str, #[case] value: &str) {
        assert_eq!(color(&[(key, value)]), ColorInfo::unknown());
    }

    #[rstest]
    #[case("TMUX", "session")]
    #[case("ZELLIJ", "0")]
    #[case("STY", "session")]
    #[case("TERM", "tmux-direct")]
    #[case("TERM", "screen.xterm-256color")]
    #[case("TERM", "dumb")]
    fn possibly_inherited_hints_are_unknown(#[case] key: &str, #[case] value: &str) {
        let mut pairs = vec![
            ("TERM_PROGRAM", "Apple_Terminal"),
            ("TERM_PROGRAM_VERSION", "470.2"),
            ("COLORTERM", "truecolor"),
        ];
        pairs.push((key, value));
        assert_eq!(color(&pairs), ColorInfo::unknown());
    }

    #[rstest]
    #[case("3.1c", None, ColorSupport::Unknown)]
    #[case("3.2", None, ColorSupport::TrueColor)]
    #[case("3.5a", None, ColorSupport::TrueColor)]
    #[case("3.6a", Some("screen-256color"), ColorSupport::TrueColor)]
    #[case("3.10", Some("tmux-256color"), ColorSupport::TrueColor)]
    #[case("4.0", None, ColorSupport::TrueColor)]
    #[case("3.8", Some("dumb"), ColorSupport::Unknown)]
    #[case("next-3.7", None, ColorSupport::Unknown)]
    #[case("", None, ColorSupport::Unknown)]
    fn tmux_support_depends_on_its_version(
        #[case] version: &str,
        #[case] term: Option<&str>,
        #[case] support: ColorSupport,
    ) {
        let mut pairs = vec![
            ("TMUX", "session"),
            ("TERM_PROGRAM", "tmux"),
            ("TERM_PROGRAM_VERSION", version),
            ("COLORTERM", "truecolor"),
        ];
        pairs.extend(term.map(|value| ("TERM", value)));
        let keys: &[&str] = match support {
            ColorSupport::TrueColor => &["TMUX", "TERM_PROGRAM", "TERM_PROGRAM_VERSION"],
            _ => &[],
        };
        assert_eq!(color(&pairs), ColorInfo::from_env(support, keys));
    }

    #[rstest]
    #[case("WT_SESSION", "session")]
    #[case("TERM_PROGRAM", "WindowsTerminal")]
    fn windows_terminal_reuses_identity_evidence(#[case] key: &str, #[case] value: &str) {
        let expected = ColorInfo::from_env(ColorSupport::TrueColor, &[key]);
        assert_eq!(color(&[(key, value)]), expected);
    }

    #[rstest]
    #[case("2.9.3", ColorSupport::Unknown)]
    #[case("3.0", ColorSupport::TrueColor)]
    #[case("3.6.5", ColorSupport::TrueColor)]
    #[case("", ColorSupport::Unknown)]
    fn iterm2_support_depends_on_its_version(#[case] version: &str, #[case] support: ColorSupport) {
        let pairs = [
            ("TERM_PROGRAM", "iTerm.app"),
            ("TERM_PROGRAM_VERSION", version),
        ];
        let keys: &[&str] = match support {
            ColorSupport::TrueColor => &["TERM_PROGRAM", "TERM_PROGRAM_VERSION"],
            _ => &[],
        };
        assert_eq!(color(&pairs), ColorInfo::from_env(support, keys));
    }

    #[rstest]
    #[case("TMUX", "session")]
    #[case("TERM", "dumb")]
    fn windows_terminal_hint_can_be_inherited(#[case] key: &str, #[case] value: &str) {
        let pairs = [("WT_SESSION", "session"), (key, value)];
        assert_eq!(color(&pairs), ColorInfo::unknown());
    }

    #[test]
    fn inference_does_not_require_diagnostic_capture() {
        let env = EnvMap::from([("COLORTERM".into(), "truecolor".into())]);
        let options = DetectOptions {
            capture_env_subset: false,
            ..DetectOptions::default()
        };
        let info = detect_with_options(&env, options);
        assert_eq!(info.color.support, ColorSupport::TrueColor);
        assert_eq!(info.colorterm.as_deref(), Some("truecolor"));
        assert!(info.raw_env_subset.is_empty());
        assert!(info.detected_via.is_empty());
    }
}
