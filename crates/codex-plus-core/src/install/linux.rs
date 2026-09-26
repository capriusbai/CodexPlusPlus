#[cfg(all(unix, not(target_os = "macos")))]
use std::fs;
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::{
    InstallOptions, LINUX_ICON_NAME, LINUX_MANAGER_DESKTOP_ID, LINUX_SILENT_DESKTOP_ID,
    MANAGER_BINARY, MANAGER_NAME, SILENT_BINARY, SILENT_NAME, install_root_or_default,
    option_or_current_exe,
};

/// Size bucket used for the hicolor icon theme. The shipped
/// `codex-plus-plus.png` is a 256x256 image, so it is installed unscaled.
const ICON_THEME_SIZE: &str = "256x256";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxDesktopEntry {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxEntrypointPlan {
    pub silent_entry: LinuxDesktopEntry,
    pub manager_entry: LinuxDesktopEntry,
    pub launcher_path: String,
    pub manager_path: String,
    pub icon_name: String,
    pub icon_source: Option<PathBuf>,
    pub icon_target: Option<PathBuf>,
    pub remove_owned_data: bool,
}

pub fn build_linux_entrypoint_plan(options: &InstallOptions) -> LinuxEntrypointPlan {
    let applications_dir = install_root_or_default(options);
    let launcher = option_or_current_exe(&options.launcher_path, SILENT_BINARY);
    let manager = option_or_current_exe(&options.manager_path, MANAGER_BINARY);
    let icon_source = icon_source_next_to(&manager).or_else(|| icon_source_next_to(&launcher));
    LinuxEntrypointPlan {
        silent_entry: LinuxDesktopEntry {
            path: applications_dir.join(format!("{LINUX_SILENT_DESKTOP_ID}.desktop")),
            contents: desktop_entry_contents(&launcher, false),
        },
        manager_entry: LinuxDesktopEntry {
            path: applications_dir.join(format!("{LINUX_MANAGER_DESKTOP_ID}.desktop")),
            contents: desktop_entry_contents(&manager, true),
        },
        launcher_path: launcher.to_string_lossy().to_string(),
        manager_path: manager.to_string_lossy().to_string(),
        icon_name: LINUX_ICON_NAME.to_string(),
        icon_source: icon_source.clone(),
        icon_target: icon_source.map(|_| user_icon_target()),
        remove_owned_data: options.remove_owned_data,
    }
}

/// `$XDG_DATA_HOME`, falling back to `~/.local/share`.
pub fn user_data_dir() -> PathBuf {
    if let Some(value) = std::env::var_os("XDG_DATA_HOME") {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return path;
        }
    }
    directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(".local").join("share"))
        .unwrap_or_else(|| PathBuf::from(".local/share"))
}

pub fn user_applications_dir() -> PathBuf {
    user_data_dir().join("applications")
}

/// Directories a packaged install (`.deb`, distro package) may own.
pub fn system_applications_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ]
}

fn user_icon_target() -> PathBuf {
    user_data_dir()
        .join("icons")
        .join("hicolor")
        .join(ICON_THEME_SIZE)
        .join("apps")
        .join(format!("{LINUX_ICON_NAME}.png"))
}

fn icon_source_next_to(binary: &Path) -> Option<PathBuf> {
    let candidate = binary
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{LINUX_ICON_NAME}.png"));
    candidate.exists().then_some(candidate)
}

fn desktop_entry_contents(target: &Path, manager: bool) -> String {
    let version = crate::version::VERSION;
    let exec = escape_exec(&target.to_string_lossy());
    if manager {
        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name=Codex++ Manager\n\
             Name[zh_CN]={MANAGER_NAME}\n\
             GenericName=Codex++ control panel\n\
             GenericName[zh_CN]=Codex++ 管理工具\n\
             Comment=Launch, repair, configure and update the Codex++ enhancements\n\
             Comment[zh_CN]=启动、检查、修复、更新 Codex++ 增强功能\n\
             Exec=\"{exec}\" %U\n\
             Icon={LINUX_ICON_NAME}\n\
             Terminal=false\n\
             StartupNotify=true\n\
             StartupWMClass=codex-plus-plus-manager\n\
             Categories=Development;Utility;\n\
             Keywords=codex;codex++;launcher;\n\
             X-Codex-Plus-Plus-Version={version}\n"
        )
    } else {
        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Version=1.0\n\
             Name={SILENT_NAME}\n\
             GenericName=Codex++ silent launcher\n\
             GenericName[zh_CN]=Codex++ 静默启动入口\n\
             Comment=Start Codex and inject the Codex++ enhancements\n\
             Comment[zh_CN]=启动 Codex 并注入 Codex++ 增强功能\n\
             Exec=\"{exec}\"\n\
             Icon={LINUX_ICON_NAME}\n\
             Terminal=false\n\
             StartupNotify=false\n\
             Categories=Development;Utility;\n\
             Keywords=codex;codex++;launcher;\n\
             X-Codex-Plus-Plus-Version={version}\n"
        )
    }
}

/// Desktop entry `Exec=` values are unquoted by the launcher, so a literal
/// backslash or double quote inside the path has to stay escaped.
fn escape_exec(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn install_desktop_entries(options: &InstallOptions) -> anyhow::Result<()> {
    let plan = build_linux_entrypoint_plan(options);
    write_desktop_entry(&plan.silent_entry)?;
    write_desktop_entry(&plan.manager_entry)?;
    if let (Some(source), Some(target)) = (&plan.icon_source, &plan.icon_target) {
        install_icon(source, target)?;
    }
    refresh_desktop_caches(&plan);
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn uninstall_desktop_entries(options: &InstallOptions) -> anyhow::Result<()> {
    let plan = build_linux_entrypoint_plan(options);
    for entry in [&plan.silent_entry, &plan.manager_entry] {
        if entry.path.exists() {
            fs::remove_file(&entry.path)?;
        }
    }
    if let Some(target) = plan.icon_target.as_ref().filter(|path| path.exists()) {
        let _ = fs::remove_file(target);
    }
    refresh_desktop_caches(&plan);
    Ok(())
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
pub fn install_desktop_entries(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("XDG desktop entries are only supported on Linux")
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
pub fn uninstall_desktop_entries(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("XDG desktop entries are only supported on Linux")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn write_desktop_entry(entry: &LinuxDesktopEntry) -> anyhow::Result<()> {
    if let Some(parent) = entry.path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&entry.path, &entry.contents)?;
    // Desktop files dropped on the desktop itself are only offered to the user
    // when they carry the executable bit.
    let mut permissions = fs::metadata(&entry.path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&entry.path, permissions)?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn install_icon(source: &Path, target: &Path) -> anyhow::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    let mut permissions = fs::metadata(target)?.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(target, permissions)?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn refresh_desktop_caches(plan: &LinuxEntrypointPlan) {
    if let Some(applications_dir) = plan.silent_entry.path.parent() {
        let _ = std::process::Command::new("update-desktop-database")
            .arg(applications_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    if let Some(theme_dir) = plan
        .icon_target
        .as_ref()
        .and_then(|path| path.parent()?.parent()?.parent().map(Path::to_path_buf))
    {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .args(["--quiet", "--ignore-theme-index"])
            .arg(theme_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> InstallOptions {
        InstallOptions {
            install_root: Some("/home/tester/.local/share/applications".into()),
            launcher_path: Some("/usr/lib/codex-plus-plus/codex-plus-plus".into()),
            manager_path: Some("/usr/lib/codex-plus-plus/codex-plus-plus-manager".into()),
            remove_owned_data: false,
        }
    }

    #[test]
    fn plan_uses_stable_desktop_file_ids() {
        let plan = build_linux_entrypoint_plan(&options());

        assert!(plan.silent_entry.path.ends_with("codex-plus-plus.desktop"));
        assert!(
            plan.manager_entry
                .path
                .ends_with("codex-plus-plus-manager.desktop")
        );
    }

    #[test]
    fn desktop_entries_point_at_the_installed_binaries() {
        let plan = build_linux_entrypoint_plan(&options());

        assert!(
            plan.silent_entry
                .contents
                .contains("Exec=\"/usr/lib/codex-plus-plus/codex-plus-plus\"\n")
        );
        assert!(
            plan.manager_entry
                .contents
                .contains("Exec=\"/usr/lib/codex-plus-plus/codex-plus-plus-manager\" %U")
        );
        assert!(plan.silent_entry.contents.starts_with("[Desktop Entry]\n"));
        assert!(
            plan.manager_entry
                .contents
                .contains("Icon=codex-plus-plus\n")
        );
    }

    #[test]
    fn desktop_entries_keep_localized_names() {
        let plan = build_linux_entrypoint_plan(&options());

        assert!(plan.silent_entry.contents.contains("Name=Codex++\n"));
        assert!(
            plan.manager_entry
                .contents
                .contains("Name=Codex++ Manager\n")
        );
        assert!(
            plan.manager_entry
                .contents
                .contains("Name[zh_CN]=Codex++ 管理工具\n")
        );
    }

    #[test]
    fn exec_escaping_keeps_quotes_and_backslashes_literal() {
        assert_eq!(escape_exec("/opt/co\"dex"), "/opt/co\\\"dex");
        assert_eq!(escape_exec("/opt/co\\dex"), "/opt/co\\\\dex");
    }

    #[test]
    fn icon_is_skipped_when_no_source_image_ships_next_to_the_binaries() {
        let plan = build_linux_entrypoint_plan(&options());

        assert_eq!(plan.icon_source, None);
        assert_eq!(plan.icon_target, None);
    }

    #[test]
    fn icon_target_lands_in_the_hicolor_theme_when_a_source_image_exists() {
        let temp = tempfile::tempdir().unwrap();
        let binary = temp.path().join(MANAGER_BINARY);
        std::fs::write(&binary, b"").unwrap();
        std::fs::write(temp.path().join("codex-plus-plus.png"), b"").unwrap();

        let plan = build_linux_entrypoint_plan(&InstallOptions {
            install_root: Some(temp.path().join("applications")),
            launcher_path: Some(temp.path().join(SILENT_BINARY)),
            manager_path: Some(binary),
            remove_owned_data: false,
        });

        assert!(plan.icon_source.is_some());
        let target = plan.icon_target.expect("icon target");
        assert!(target.ends_with("icons/hicolor/256x256/apps/codex-plus-plus.png"));
    }
}
