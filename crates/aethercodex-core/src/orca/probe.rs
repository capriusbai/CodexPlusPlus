//! Find the Orca IDE CLI, and never mistake something else for it.
//!
//! Ubuntu ships GNOME Orca — a screen reader — as the `orca` package at
//! `/usr/bin/orca`. Orca IDE knows this and installs itself as `orca-ide` on
//! Linux. Two rules follow, and both are enforced here rather than left to
//! callers:
//!
//! 1. `orca-ide` is tried before `orca`.
//! 2. A binary whose `--version` identifies it as a screen reader is refused,
//!    so no subcommand is ever addressed to a user's assistive technology.
//!
//! Probing is read-only: it runs `--version` and nothing else.

use std::path::PathBuf;
use std::time::Duration;

use super::command;

/// `--version` should answer immediately; anything slower is not worth waiting for.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Executable names to try, in order.
///
/// On Linux only `orca-ide` is considered. Orca IDE's own packaging sets
/// `executableName: 'orca-ide'` there precisely because the distro `orca` is
/// the screen reader, so a bare `orca` on a Linux PATH is never Orca IDE and
/// is not worth even asking `--version`. Elsewhere GNOME Orca does not exist
/// and a bare `orca` is a plausible Orca IDE.
fn cli_names() -> &'static [&'static str] {
    if cfg!(target_os = "linux") {
        &["orca-ide"]
    } else {
        &["orca-ide", "orca"]
    }
}

/// Paths that are the distro screen reader, never Orca IDE. Refused outright.
const SCREEN_READER_PATHS: [&str; 2] = ["/usr/bin/orca", "/bin/orca"];

/// A major version at or above this cannot be Orca IDE, which is 1.x, but is
/// ordinary for GNOME Orca, which tracks GNOME's 40+ numbering. This catches a
/// screen reader whose `--version` says only "Orca 50.2", with no wording to
/// match on. Revisit if Orca IDE ever reaches double digits.
const IMPLAUSIBLE_MAJOR_VERSION: u64 = 10;

/// Phrases that identify GNOME Orca, the screen reader, rather than Orca IDE.
const SCREEN_READER_MARKERS: [&str; 3] = ["at-spi", "screen reader", "screen-reader"];

/// The Orca version this integration was designed against. Reported so the UI
/// can say "older than what we verified" instead of silently assuming
/// behaviour; the plugin API is not frozen, so this is information, not a gate.
pub const VALIDATED_VERSION: &str = "1.4.210";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcaCli {
    /// What to invoke. An absolute path when found off `PATH`.
    pub program: String,
    /// Version string as reported, when it could be parsed out.
    pub version: Option<String>,
}

/// Whether a `--version` output belongs to GNOME Orca rather than Orca IDE.
pub fn is_screen_reader(version_output: &str) -> bool {
    let haystack = version_output.to_ascii_lowercase();
    SCREEN_READER_MARKERS
        .iter()
        .any(|marker| haystack.contains(marker))
}

/// Pull a dotted version out of a `--version` line.
pub fn parse_version(version_output: &str) -> Option<String> {
    let mut current = String::new();
    let mut best: Option<String> = None;
    for ch in version_output.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
            continue;
        }
        consider(&mut current, &mut best);
    }
    consider(&mut current, &mut best);
    best
}

fn consider(current: &mut String, best: &mut Option<String>) {
    let candidate = std::mem::take(current);
    let trimmed = candidate.trim_matches('.');
    // At least two components, so a bare year or a lone number is not a version.
    if trimmed.split('.').filter(|part| !part.is_empty()).count() < 2 {
        return;
    }
    if best.is_none() {
        *best = Some(trimmed.to_string());
    }
}

/// Locations the packaged app leaves a CLI when the `PATH` symlink is missing.
pub fn unlinked_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("/opt/orca-ide/bin/orca-ide"),
        PathBuf::from("/opt/Orca/bin/orca-ide"),
        PathBuf::from("/usr/lib/orca-ide/bin/orca-ide"),
        PathBuf::from("/usr/local/bin/orca-ide"),
    ];
    if let Some(home) = home_dir() {
        candidates.push(home.join(".local/bin/orca-ide"));
    }
    candidates.push(PathBuf::from(
        "/Applications/Orca.app/Contents/Resources/app/out/cli/index.js",
    ));
    candidates
}

/// Find an Orca IDE CLI, or `None` when Orca is not installed.
///
/// Returning `None` is the ordinary case on a machine without Orca and is not
/// an error: every Orca capability hides itself silently when this is `None`.
pub fn probe() -> Option<OrcaCli> {
    for name in cli_names() {
        if let Some(cli) = identify(name) {
            return Some(cli);
        }
    }
    for candidate in unlinked_candidates() {
        if !candidate.exists() {
            continue;
        }
        if let Some(cli) = identify(&candidate.to_string_lossy()) {
            return Some(cli);
        }
    }
    None
}

/// Ask one candidate what it is, and accept it only if it is Orca IDE.
///
/// Three independent refusals, because getting this wrong means sending
/// commands to someone's screen reader:
///
/// 1. A path that is the distro screen reader is refused without running it.
/// 2. Wording that identifies a screen reader is refused.
/// 3. A version too high to be Orca IDE is refused, which covers a screen
///    reader that reports nothing but its version.
fn identify(program: &str) -> Option<OrcaCli> {
    if SCREEN_READER_PATHS.contains(&program) {
        return None;
    }
    let output = command::run(program, &["--version"], PROBE_TIMEOUT).ok()?;
    let reported = if output.stdout.trim().is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    if is_screen_reader(&reported) {
        return None;
    }
    let version = parse_version(&reported);
    if version.as_deref().is_some_and(major_is_implausible) {
        return None;
    }
    Some(OrcaCli {
        program: program.to_string(),
        version,
    })
}

/// Whether a version's major component is too high for Orca IDE.
fn major_is_implausible(version: &str) -> bool {
    version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u64>().ok())
        .is_some_and(|major| major >= IMPLAUSIBLE_MAJOR_VERSION)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Whether `version` is older than the version this integration was verified against.
pub fn older_than_validated(version: &str) -> bool {
    matches!(
        compare_versions(version, VALIDATED_VERSION),
        Some(std::cmp::Ordering::Less)
    )
}

fn compare_versions(left: &str, right: &str) -> Option<std::cmp::Ordering> {
    // Every component must parse. Treating an unreadable version as 0 would
    // report it as older than anything, which is a claim we cannot make.
    let parse = |text: &str| -> Option<Vec<u64>> {
        let parts: Vec<u64> = text
            .split('.')
            .map(|part| part.parse::<u64>().ok())
            .collect::<Option<_>>()?;
        (!parts.is_empty()).then_some(parts)
    };
    let (left, right) = (parse(left)?, parse(right)?);
    let width = left.len().max(right.len());
    for index in 0..width {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a != b {
            return Some(a.cmp(&b));
        }
    }
    Some(std::cmp::Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnome_orca_is_refused_by_wording_or_by_version() {
        // Shapes GNOME Orca's --version can produce. The bare "Orca 50.2" case
        // is the one wording alone does not catch: it is refused on its major
        // version instead. Both paths must hold, since either alone leaves a
        // hole through which subcommands reach a screen reader.
        for output in [
            "Orca 50.2",
            "Orca 46.0",
            "Orca screen reader version 50.2",
            "Orca requires AT-SPI to be running",
            "orca: a screen reader for the GNOME desktop",
        ] {
            let refused = is_screen_reader(output)
                || parse_version(output)
                    .as_deref()
                    .is_some_and(major_is_implausible);
            assert!(refused, "{output:?} would be accepted as Orca IDE");
        }
        assert!(is_screen_reader("Orca requires AT-SPI to be running"));
        assert!(is_screen_reader("a SCREEN READER for GNOME"));
    }

    #[test]
    fn linux_never_considers_the_bare_orca_name() {
        // Orca IDE installs as orca-ide on Linux by its own packaging, so a
        // bare `orca` there is the screen reader and is not probed at all.
        if cfg!(target_os = "linux") {
            assert_eq!(cli_names(), &["orca-ide"]);
        } else {
            assert_eq!(cli_names(), &["orca-ide", "orca"]);
        }
        assert_eq!(cli_names()[0], "orca-ide");
    }

    #[test]
    fn the_screen_readers_own_paths_are_refused_without_running_them() {
        for path in SCREEN_READER_PATHS {
            assert_eq!(identify(path), None, "{path} must not be probed");
        }
    }

    #[test]
    fn an_orca_ide_version_is_not_refused_as_implausible() {
        assert!(!major_is_implausible("1.4.210"));
        assert!(!major_is_implausible("2.0.0"));
        assert!(major_is_implausible("50.2"));
        assert!(major_is_implausible("10.0"));
    }

    #[test]
    fn orca_ide_is_not_mistaken_for_a_screen_reader() {
        assert!(!is_screen_reader("1.4.210"));
        assert!(!is_screen_reader("Orca IDE 1.4.210"));
    }

    #[test]
    fn version_is_parsed_from_the_shapes_the_cli_prints() {
        assert_eq!(parse_version("1.4.210"), Some("1.4.210".to_string()));
        assert_eq!(
            parse_version("orca-ide 1.4.210\n"),
            Some("1.4.210".to_string())
        );
        assert_eq!(
            parse_version("Orca IDE\nversion 1.4.210 (build 9)"),
            Some("1.4.210".to_string())
        );
        // A single number is not a version, so a bare "50" cannot pass as one.
        assert_eq!(parse_version("Orca 50"), None);
        assert_eq!(parse_version("no digits here"), None);
    }

    #[test]
    fn versions_compare_by_component_not_by_string() {
        // The string comparison this guards against: "1.4.9" > "1.4.210".
        assert!(older_than_validated("1.4.9"));
        assert!(older_than_validated("1.3.999"));
        assert!(!older_than_validated("1.4.210"));
        assert!(!older_than_validated("1.5.0"));
        assert!(!older_than_validated("2.0.0"));
        // Unparseable input is never reported as old: we cannot claim an
        // ordering for a version we could not read.
        for unreadable in ["", "unknown", "1.x", "v1.4.210", "1..4"] {
            assert!(
                !older_than_validated(unreadable),
                "{unreadable:?} is unreadable, not old"
            );
        }
    }

    #[test]
    fn a_screen_reader_candidate_is_rejected_by_identify() {
        // A stand-in that answers --version the way GNOME Orca does. Proves the
        // refusal happens in identify(), not only in the pure predicate.
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("orca");
        std::fs::write(
            &fake,
            "#!/bin/sh\necho 'Orca requires AT-SPI to be running'\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
            assert_eq!(identify(&fake.to_string_lossy()), None);
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_well_behaved_candidate_is_accepted_with_its_version() {
        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("orca-ide");
        std::fs::write(&fake, "#!/bin/sh\necho '1.4.210'\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();

        let cli = identify(&fake.to_string_lossy()).expect("should be accepted");
        assert_eq!(cli.version, Some("1.4.210".to_string()));
    }

    #[test]
    fn unlinked_candidates_prefer_the_ide_executable_name() {
        for candidate in unlinked_candidates() {
            let text = candidate.to_string_lossy().to_string();
            assert!(
                text.contains("orca-ide") || text.contains("Orca.app"),
                "{text} could resolve to the screen reader"
            );
        }
    }

    #[test]
    fn a_path_without_orca_probes_to_none() {
        let dir = tempfile::tempdir().unwrap();
        let previous = std::env::var_os("PATH");
        // Safety: single-threaded assertion around one process-wide variable,
        // restored before returning.
        unsafe { std::env::set_var("PATH", dir.path()) };
        let found = probe();
        match previous {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        // Only meaningful when the host has no Orca in the unlinked locations.
        if unlinked_candidates().iter().all(|path| !path.exists()) {
            assert_eq!(found, None);
        }
    }
}
