use std::path::{Path, PathBuf};

use super::{
    InstallOptions, MANAGER_BINARY, MANAGER_NAME, SILENT_BINARY, SILENT_NAME,
    install_root_or_default, option_or_current_exe,
};

#[cfg(windows)]
const UNINSTALL_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\AetherCodex";
/// Uninstall keys written by releases published under the previous product
/// names. They are removed on install and uninstall so an upgrade does not
/// leave a second entry in "Apps & features".
#[cfg(windows)]
const LEGACY_UNINSTALL_SUBKEYS: &[&str] = &[
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\CodexPlusPlus",
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Codex++",
];

/// Shortcut file names shipped by releases under the previous product name.
const LEGACY_SHORTCUT_NAMES: &[&str] = &["Codex++.lnk", "Codex++ 管理工具.lnk"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsEntrypointPlan {
    pub install_root: String,
    pub silent_shortcut: String,
    pub manager_shortcut: String,
    pub launcher_path: String,
    pub manager_path: String,
    pub icon_path: String,
    pub silent_icon_path: String,
    pub manager_icon_path: String,
    pub uninstall_key: String,
    /// Uninstall keys from previous product names, cleaned up on upgrade.
    pub legacy_uninstall_keys: Vec<String>,
    /// Desktop/Start Menu shortcuts from previous product names, removed on
    /// upgrade so the user is not left with two sets of icons.
    pub legacy_shortcuts: Vec<String>,
    pub remove_owned_data: bool,
}

pub fn build_windows_entrypoint_plan(options: &InstallOptions) -> WindowsEntrypointPlan {
    let install_root = install_root_or_default(options);
    let launcher_path = option_or_current_exe(&options.launcher_path, SILENT_BINARY);
    let manager_path = option_or_current_exe(&options.manager_path, MANAGER_BINARY);
    let icon_path = default_icon_path();
    WindowsEntrypointPlan {
        silent_shortcut: install_root
            .join("AetherCodex.lnk")
            .to_string_lossy()
            .to_string(),
        manager_shortcut: install_root
            .join("AetherCodex 管理工具.lnk")
            .to_string_lossy()
            .to_string(),
        install_root: install_root.to_string_lossy().to_string(),
        launcher_path: launcher_path.to_string_lossy().to_string(),
        manager_path: manager_path.to_string_lossy().to_string(),
        icon_path: icon_path.to_string_lossy().to_string(),
        silent_icon_path: launcher_path.to_string_lossy().to_string(),
        manager_icon_path: manager_path.to_string_lossy().to_string(),
        uninstall_key: "AetherCodex".to_string(),
        legacy_uninstall_keys: vec!["CodexPlusPlus".to_string(), "Codex++".to_string()],
        legacy_shortcuts: LEGACY_SHORTCUT_NAMES
            .iter()
            .map(|name| install_root.join(name).to_string_lossy().to_string())
            .collect(),
        remove_owned_data: options.remove_owned_data,
    }
}

#[cfg(windows)]
pub fn install_shortcuts(options: &InstallOptions) -> anyhow::Result<()> {
    let plan = build_windows_entrypoint_plan(options);
    let install_root = PathBuf::from(&plan.install_root);
    std::fs::create_dir_all(&install_root)?;
    create_entrypoint_shortcut(
        PathBuf::from(&plan.silent_shortcut),
        PathBuf::from(&plan.launcher_path),
        "Launch AetherCodex silently",
        PathBuf::from(&plan.silent_icon_path),
    )?;
    create_entrypoint_shortcut(
        PathBuf::from(&plan.manager_shortcut),
        PathBuf::from(&plan.manager_path),
        "Open AetherCodex management tool",
        PathBuf::from(&plan.manager_icon_path),
    )?;
    remove_legacy_entrypoints(&plan);
    write_uninstall_registration(&plan)?;
    Ok(())
}

#[cfg(windows)]
pub fn uninstall_shortcuts(options: &InstallOptions) -> anyhow::Result<()> {
    let plan = build_windows_entrypoint_plan(options);
    let _ = std::fs::remove_file(&plan.silent_shortcut);
    let _ = std::fs::remove_file(&plan.manager_shortcut);
    remove_legacy_entrypoints(&plan);
    let _ = crate::windows_integration::delete_current_user_key(UNINSTALL_SUBKEY);
    Ok(())
}

#[cfg(windows)]
fn remove_legacy_entrypoints(plan: &WindowsEntrypointPlan) {
    for shortcut in &plan.legacy_shortcuts {
        let _ = std::fs::remove_file(shortcut);
    }
    for subkey in LEGACY_UNINSTALL_SUBKEYS {
        let _ = crate::windows_integration::delete_current_user_key(subkey);
    }
}

#[cfg(not(windows))]
pub fn install_shortcuts(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("Windows shortcuts are only supported on Windows")
}

#[cfg(not(windows))]
pub fn uninstall_shortcuts(_options: &InstallOptions) -> anyhow::Result<()> {
    anyhow::bail!("Windows shortcuts are only supported on Windows")
}

#[cfg(windows)]
fn create_entrypoint_shortcut(
    path: PathBuf,
    target: PathBuf,
    description: &str,
    icon: PathBuf,
) -> anyhow::Result<()> {
    crate::windows_integration::create_shortcut(&crate::windows_integration::ShortcutSpec {
        working_directory: target.parent().map(Path::to_path_buf),
        path,
        target,
        arguments: String::new(),
        description: description.to_string(),
        icon: Some(icon),
        show_minimized: false,
    })
}

#[cfg(windows)]
fn write_uninstall_registration(plan: &WindowsEntrypointPlan) -> anyhow::Result<()> {
    let uninstall_command = format!("\"{}\"", plan.manager_path);
    let install_location = Path::new(&plan.manager_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(&plan.install_root))
        .to_string_lossy()
        .to_string();
    for (name, value) in [
        ("DisplayName", "AetherCodex".to_string()),
        ("DisplayVersion", crate::version::VERSION.to_string()),
        ("Publisher", "Archai".to_string()),
        ("DisplayIcon", plan.manager_icon_path.clone()),
        ("InstallLocation", install_location),
        ("UninstallString", uninstall_command.clone()),
        ("QuietUninstallString", uninstall_command),
    ] {
        crate::windows_integration::set_current_user_string_value(UNINSTALL_SUBKEY, name, &value)?;
    }
    Ok(())
}

fn default_icon_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.join("aethercodex.ico"))
        .unwrap_or_else(|| PathBuf::from("aethercodex.ico"))
}

#[allow(dead_code)]
fn _entrypoint_names() -> (&'static str, &'static str) {
    (SILENT_NAME, MANAGER_NAME)
}
