use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const APP_STATE_DIR: &str = ".aethercodex";
/// State directory used by releases published under the previous product name.
/// Settings, status and logs are moved out of it on first run.
const LEGACY_APP_STATE_DIR: &str = ".codex-session-delete";
const SETTINGS_FILE: &str = "settings.json";
const LATEST_STATUS_FILE: &str = "latest-status.json";
const DIAGNOSTIC_LOG_FILE: &str = "aethercodex.log";
const LEGACY_DIAGNOSTIC_LOG_FILE: &str = "codex-plus.log";
const CONFIG_DIR_NAME: &str = "AetherCodex";
/// Config directory name used by releases published under the previous product
/// name. Installed user scripts are moved out of it on first use.
const LEGACY_CONFIG_DIR_NAME: &str = "Codex++";

pub fn default_app_state_dir() -> PathBuf {
    let Some(home_dir) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
    else {
        return PathBuf::from(APP_STATE_DIR);
    };
    let state_dir = home_dir.join(APP_STATE_DIR);
    // Guarded so the filesystem probes below run once per process rather than
    // on every settings read.
    static MIGRATED: OnceLock<()> = OnceLock::new();
    if MIGRATED.set(()).is_ok() {
        let legacy_dir = home_dir.join(LEGACY_APP_STATE_DIR);
        if migrate_legacy_dir(&legacy_dir, &state_dir) {
            rename_legacy_diagnostic_log(&state_dir);
        }
    }
    state_dir
}

/// Move a directory owned by a pre-rename release to its new location.
///
/// Returns whether anything was moved. The move is skipped when the new
/// directory already exists, so state written by this version always wins even
/// if an older build later recreates the legacy directory. Failures are
/// deliberately quiet: the caller only wants a path back, and a missing
/// settings file is recoverable.
pub fn migrate_legacy_dir(legacy_dir: &Path, target_dir: &Path) -> bool {
    if target_dir.exists() || !legacy_dir.is_dir() {
        return false;
    }
    if std::fs::rename(legacy_dir, target_dir).is_ok() {
        return true;
    }
    // `rename` fails across filesystems, so fall back to copying.
    copy_dir_recursive(legacy_dir, target_dir).is_ok()
}

fn rename_legacy_diagnostic_log(state_dir: &Path) {
    let legacy_log = state_dir.join(LEGACY_DIAGNOSTIC_LOG_FILE);
    if legacy_log.is_file() && !state_dir.join(DIAGNOSTIC_LOG_FILE).exists() {
        let _ = std::fs::rename(legacy_log, state_dir.join(DIAGNOSTIC_LOG_FILE));
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Per-user config root for user scripts: `%APPDATA%\AetherCodex` on Windows,
/// `$XDG_CONFIG_HOME/AetherCodex` (or `~/.config/AetherCodex`) elsewhere.
///
/// A pre-rename directory is migrated on first use so installed user scripts
/// survive the rename.
pub fn default_user_scripts_config_dir() -> PathBuf {
    let root = user_config_root();
    let config_dir = root.join(CONFIG_DIR_NAME);
    static MIGRATED: OnceLock<()> = OnceLock::new();
    if MIGRATED.set(()).is_ok() {
        migrate_legacy_dir(&root.join(LEGACY_CONFIG_DIR_NAME), &config_dir);
    }
    config_dir
}

fn user_config_root() -> PathBuf {
    if cfg!(windows) {
        if let Some(roaming) = std::env::var_os("APPDATA") {
            return PathBuf::from(roaming);
        }
        if let Some(home) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()) {
            return home.join("AppData").join("Roaming");
        }
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| directories::BaseDirs::new().map(|dirs| dirs.home_dir().join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"))
}

pub fn default_settings_path() -> PathBuf {
    if let Some(path) = settings_path_for_tests() {
        return path;
    }
    default_app_state_dir().join(SETTINGS_FILE)
}

pub fn default_latest_status_path() -> PathBuf {
    default_app_state_dir().join(LATEST_STATUS_FILE)
}

pub fn default_diagnostic_log_path() -> PathBuf {
    default_app_state_dir().join(DIAGNOSTIC_LOG_FILE)
}

fn settings_path_for_tests() -> Option<PathBuf> {
    SETTINGS_PATH_FOR_TESTS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|path| path.clone())
}

static SETTINGS_PATH_FOR_TESTS: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

pub fn set_settings_path_for_tests(path: Option<PathBuf>) -> Option<PathBuf> {
    SETTINGS_PATH_FOR_TESTS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|mut current| std::mem::replace(&mut *current, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_path_uses_app_state_directory() {
        let path = default_settings_path();

        assert!(path.ends_with(".aethercodex/settings.json"));
    }

    #[test]
    fn default_latest_status_path_uses_app_state_directory() {
        let path = default_latest_status_path();

        assert!(path.ends_with(".aethercodex/latest-status.json"));
    }

    #[test]
    fn legacy_state_directory_is_moved_and_its_log_renamed() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join(LEGACY_APP_STATE_DIR);
        let target = temp.path().join(APP_STATE_DIR);
        std::fs::create_dir_all(legacy.join("user_scripts")).unwrap();
        std::fs::write(legacy.join(SETTINGS_FILE), b"{\"a\":1}").unwrap();
        std::fs::write(legacy.join(LEGACY_DIAGNOSTIC_LOG_FILE), b"old log").unwrap();

        assert!(migrate_legacy_dir(&legacy, &target));
        rename_legacy_diagnostic_log(&target);

        assert_eq!(
            std::fs::read_to_string(target.join(SETTINGS_FILE)).unwrap(),
            "{\"a\":1}"
        );
        assert_eq!(
            std::fs::read_to_string(target.join(DIAGNOSTIC_LOG_FILE)).unwrap(),
            "old log"
        );
        assert!(target.join("user_scripts").is_dir());
        assert!(!legacy.exists());
    }

    #[test]
    fn migration_never_overwrites_state_written_by_this_version() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join(LEGACY_APP_STATE_DIR);
        let target = temp.path().join(APP_STATE_DIR);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join(SETTINGS_FILE), b"legacy").unwrap();
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join(SETTINGS_FILE), b"current").unwrap();

        assert!(!migrate_legacy_dir(&legacy, &target));
        assert_eq!(
            std::fs::read_to_string(target.join(SETTINGS_FILE)).unwrap(),
            "current"
        );
        assert!(legacy.exists());
    }

    #[test]
    fn migration_is_a_no_op_without_a_legacy_directory() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join(APP_STATE_DIR);

        assert!(!migrate_legacy_dir(
            &temp.path().join(LEGACY_APP_STATE_DIR),
            &target
        ));
        assert!(!target.exists());
    }

    #[test]
    fn default_diagnostic_log_path_uses_app_state_directory() {
        let path = default_diagnostic_log_path();

        assert!(path.ends_with(".aethercodex/aethercodex.log"));
    }
}
