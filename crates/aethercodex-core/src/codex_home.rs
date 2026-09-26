//! Which Codex configuration a launched Codex App reads.
//!
//! Codex resolves its configuration, credentials and session database from
//! `CODEX_HOME`, falling back to `~/.codex`. AetherCodex uses that to keep two
//! clearly separated setups rather than rewriting the one the user already
//! has:
//!
//! - [`CodexHomeProfile::Official`] — `~/.codex`. The user's own setup, shared
//!   with the Codex App they launch themselves and with anything else on the
//!   machine. AetherCodex **never writes here**; relay injection is off.
//! - [`CodexHomeProfile::Proxy`] — `~/.aethercodex/codex-home`. AetherCodex's
//!   own home, where relay injection writes. Invisible to the official setup.
//!
//! The profile is chosen per launch, so "which Codex am I looking at" is
//! always an explicit answer rather than a remembered mode.

use std::path::{Path, PathBuf};

/// Directory name of the proxy home, under the app state directory.
const PROXY_HOME_DIR: &str = "codex-home";
/// Credential file seeded from the official home so a proxy launch starts signed in.
const AUTH_FILE: &str = "auth.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum CodexHomeProfile {
    /// The user's own `~/.codex`. Read-only for AetherCodex.
    #[default]
    Official,
    /// AetherCodex's own home, where relay injection is applied.
    Proxy,
}

impl CodexHomeProfile {
    /// Stable identifier used in settings, the CLI and the bridge.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Proxy => "proxy",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "official" => Some(Self::Official),
            "proxy" => Some(Self::Proxy),
            _ => None,
        }
    }

    /// Whether AetherCodex may write configuration for this profile.
    ///
    /// The official home belongs to the user and to the Codex App they launch
    /// themselves, so relay injection is only ever applied to the proxy home.
    pub fn is_writable(self) -> bool {
        matches!(self, Self::Proxy)
    }
}

/// The official home: `~/.codex`.
pub fn official_home_dir() -> PathBuf {
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(".codex"))
        .unwrap_or_else(|| PathBuf::from(".codex"))
}

/// The proxy home: `~/.aethercodex/codex-home`.
pub fn proxy_home_dir() -> PathBuf {
    crate::paths::default_app_state_dir().join(PROXY_HOME_DIR)
}

pub fn home_dir(profile: CodexHomeProfile) -> PathBuf {
    match profile {
        CodexHomeProfile::Official => official_home_dir(),
        CodexHomeProfile::Proxy => proxy_home_dir(),
    }
}

/// Create the proxy home and seed it from the official one on first use.
///
/// Only `auth.json` is copied, and only when the proxy home does not have one:
/// a proxy launch should start signed in rather than dropping the user at a
/// login screen, but after that the two credential sets are independent. The
/// official home is opened read-only and is never modified.
pub fn prepare_proxy_home() -> std::io::Result<PathBuf> {
    let proxy = proxy_home_dir();
    std::fs::create_dir_all(&proxy)?;
    seed_credentials(&official_home_dir(), &proxy);
    Ok(proxy)
}

fn seed_credentials(official: &Path, proxy: &Path) {
    let target = proxy.join(AUTH_FILE);
    if target.exists() {
        return;
    }
    let source = official.join(AUTH_FILE);
    if source.is_file() {
        let _ = std::fs::copy(source, target);
    }
}

/// Prepare the home for a launch and return the `CODEX_HOME` to export.
///
/// `Official` returns `None` so the launched Codex resolves its home exactly
/// as it would without AetherCodex — no environment is imposed on it, and
/// nothing is created.
///
/// `Proxy` creates and seeds the home first: exporting a `CODEX_HOME` that
/// does not exist yet would drop the user at a login screen. If preparation
/// fails the launch falls back to `None` rather than pointing Codex at a
/// broken directory.
pub fn prepare_launch_env(profile: CodexHomeProfile) -> Option<PathBuf> {
    match profile {
        CodexHomeProfile::Official => None,
        CodexHomeProfile::Proxy => prepare_proxy_home().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_round_trips_through_its_identifier() {
        for profile in [CodexHomeProfile::Official, CodexHomeProfile::Proxy] {
            assert_eq!(CodexHomeProfile::parse(profile.as_str()), Some(profile));
        }
        assert_eq!(
            CodexHomeProfile::parse("OFFICIAL"),
            Some(CodexHomeProfile::Official)
        );
        assert_eq!(CodexHomeProfile::parse("nonsense"), None);
    }

    #[test]
    fn only_the_proxy_profile_is_writable() {
        assert!(!CodexHomeProfile::Official.is_writable());
        assert!(CodexHomeProfile::Proxy.is_writable());
    }

    #[test]
    fn official_launch_imposes_no_environment() {
        assert_eq!(prepare_launch_env(CodexHomeProfile::Official), None);
    }

    #[test]
    fn default_profile_is_the_untouched_official_home() {
        assert_eq!(CodexHomeProfile::default(), CodexHomeProfile::Official);
    }

    #[test]
    fn seeding_copies_credentials_once_and_never_writes_the_official_home() {
        let temp = tempfile::tempdir().unwrap();
        let official = temp.path().join(".codex");
        let proxy = temp.path().join("proxy-home");
        std::fs::create_dir_all(&official).unwrap();
        std::fs::create_dir_all(&proxy).unwrap();
        std::fs::write(official.join(AUTH_FILE), b"official-token").unwrap();

        seed_credentials(&official, &proxy);
        assert_eq!(
            std::fs::read_to_string(proxy.join(AUTH_FILE)).unwrap(),
            "official-token"
        );

        // A later proxy sign-in must not be overwritten by a reseed.
        std::fs::write(proxy.join(AUTH_FILE), b"proxy-token").unwrap();
        seed_credentials(&official, &proxy);
        assert_eq!(
            std::fs::read_to_string(proxy.join(AUTH_FILE)).unwrap(),
            "proxy-token"
        );

        // The official home is untouched throughout.
        assert_eq!(
            std::fs::read_to_string(official.join(AUTH_FILE)).unwrap(),
            "official-token"
        );
        assert_eq!(std::fs::read_dir(&official).unwrap().count(), 1);
    }

    #[test]
    fn seeding_is_a_no_op_without_official_credentials() {
        let temp = tempfile::tempdir().unwrap();
        let official = temp.path().join(".codex");
        let proxy = temp.path().join("proxy-home");
        std::fs::create_dir_all(&official).unwrap();
        std::fs::create_dir_all(&proxy).unwrap();

        seed_credentials(&official, &proxy);

        assert!(!proxy.join(AUTH_FILE).exists());
    }
}
