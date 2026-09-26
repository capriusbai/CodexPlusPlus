use aethercodex_core::install::{
    InstallOptions, SILENT_BINARY, app_bundle_names, build_linux_entrypoint_plan,
    build_macos_app_bundle, build_windows_entrypoint_plan, companion_binary_path_from_exe,
    default_install_root_strategy, shortcut_names,
};

#[test]
fn windows_entrypoint_plan_contains_silent_and_manager_entrypoints() {
    let options = InstallOptions {
        install_root: Some("C:/Users/A/Desktop".into()),
        launcher_path: Some("C:/Tools/aethercodex.exe".into()),
        manager_path: Some("C:/Tools/aethercodex-manager.exe".into()),
        remove_owned_data: false,
    };

    let plan = build_windows_entrypoint_plan(&options);

    assert!(plan.silent_shortcut.ends_with("AetherCodex.lnk"));
    assert!(plan.manager_shortcut.ends_with("AetherCodex 管理工具.lnk"));
    assert_eq!(plan.launcher_path, "C:/Tools/aethercodex.exe");
    assert_eq!(plan.manager_path, "C:/Tools/aethercodex-manager.exe");
    assert_eq!(plan.silent_icon_path, "C:/Tools/aethercodex.exe");
    assert_eq!(plan.manager_icon_path, "C:/Tools/aethercodex-manager.exe");
    assert_eq!(plan.uninstall_key, "AetherCodex");
    // Both pre-rename uninstall keys are cleaned up on upgrade.
    assert_eq!(plan.legacy_uninstall_keys, vec!["CodexPlusPlus", "Codex++"]);
    assert!(
        plan.legacy_shortcuts
            .iter()
            .any(|path| path.ends_with("Codex++.lnk"))
    );
    assert!(
        plan.legacy_shortcuts
            .iter()
            .any(|path| path.ends_with("Codex++ 管理工具.lnk"))
    );
}

#[test]
fn windows_entrypoint_plan_can_request_owned_data_removal_without_shell_script() {
    let options = InstallOptions {
        install_root: Some("C:/Users/A/Desktop".into()),
        launcher_path: None,
        manager_path: None,
        remove_owned_data: true,
    };

    let plan = build_windows_entrypoint_plan(&options);

    assert!(plan.silent_shortcut.ends_with("AetherCodex.lnk"));
    assert!(plan.manager_shortcut.ends_with("AetherCodex 管理工具.lnk"));
    assert!(plan.remove_owned_data);
}

#[test]
fn macos_bundle_metadata_contains_silent_and_manager_apps() {
    let options = InstallOptions {
        install_root: Some("/Applications".into()),
        launcher_path: Some("/opt/AetherCodex/aethercodex".into()),
        manager_path: Some("/opt/AetherCodex/aethercodex-manager".into()),
        remove_owned_data: false,
    };

    let silent = build_macos_app_bundle(&options, false);
    let manager = build_macos_app_bundle(&options, true);

    assert!(silent.app_path.ends_with("AetherCodex.app"));
    assert!(manager.app_path.ends_with("AetherCodex 管理工具.app"));
    assert!(silent.info_plist.contains("<string>AetherCodex</string>"));
    assert!(
        manager
            .info_plist
            .contains("<string>AetherCodex 管理工具</string>")
    );
    assert!(silent.launch_script.contains("aethercodex"));
    assert!(manager.launch_script.contains("aethercodex-manager"));
}

#[test]
fn installer_exports_expected_two_entrypoint_names() {
    assert_eq!(
        shortcut_names(),
        ("AetherCodex.lnk", "AetherCodex 管理工具.lnk")
    );
    assert_eq!(
        app_bundle_names(),
        ("AetherCodex.app", "AetherCodex 管理工具.app")
    );
}

#[test]
fn companion_binary_path_resolves_macos_silent_app_next_to_manager_app() {
    let manager_exe = std::path::Path::new(
        "/Applications/AetherCodex 管理工具.app/Contents/MacOS/AetherCodexManager",
    );

    let companion = companion_binary_path_from_exe(manager_exe, SILENT_BINARY);

    assert_eq!(
        companion,
        std::path::PathBuf::from("/Applications/AetherCodex.app/Contents/MacOS/AetherCodex")
    );
    assert_ne!(
        companion,
        std::path::PathBuf::from(
            "/Applications/AetherCodex 管理工具.app/Contents/MacOS/aethercodex"
        )
    );
}

#[test]
fn macos_bundle_does_not_wrap_the_bundle_executable_in_itself() {
    let options = InstallOptions {
        install_root: Some("/Applications".into()),
        launcher_path: Some("/Applications/AetherCodex.app/Contents/MacOS/AetherCodex".into()),
        manager_path: Some(
            "/Applications/AetherCodex 管理工具.app/Contents/MacOS/AetherCodexManager".into(),
        ),
        remove_owned_data: false,
    };

    let silent = build_macos_app_bundle(&options, false);
    let manager = build_macos_app_bundle(&options, true);

    assert!(!silent.launch_script.contains("AetherCodex\""));
    assert!(!manager.launch_script.contains("AetherCodexManager\""));
    assert!(silent.launch_script.contains("aethercodex"));
    assert!(manager.launch_script.contains("aethercodex-manager"));
}

#[test]
fn windows_default_install_root_uses_known_folder_before_userprofile_desktop() {
    let strategy = default_install_root_strategy();

    if cfg!(windows) {
        assert_eq!(strategy, "windows-known-folder");
    } else if cfg!(target_os = "macos") {
        assert_eq!(strategy, "macos-applications");
    } else {
        assert_eq!(strategy, "xdg-applications");
    }
}

#[test]
fn linux_entrypoint_plan_contains_both_desktop_entries() {
    let options = InstallOptions {
        install_root: Some("/home/tester/.local/share/applications".into()),
        launcher_path: Some("/usr/lib/aethercodex/aethercodex".into()),
        manager_path: Some("/usr/lib/aethercodex/aethercodex-manager".into()),
        remove_owned_data: false,
    };

    let plan = build_linux_entrypoint_plan(&options);

    assert_eq!(
        plan.silent_entry.path,
        std::path::Path::new("/home/tester/.local/share/applications/aethercodex.desktop")
    );
    assert_eq!(
        plan.manager_entry.path,
        std::path::Path::new("/home/tester/.local/share/applications/aethercodex-manager.desktop")
    );
    assert_eq!(plan.launcher_path, "/usr/lib/aethercodex/aethercodex");
    assert_eq!(
        plan.manager_path,
        "/usr/lib/aethercodex/aethercodex-manager"
    );
    assert_eq!(plan.icon_name, "aethercodex");
    assert!(
        plan.silent_entry
            .contents
            .contains("Exec=\"/usr/lib/aethercodex/aethercodex\"")
    );
    assert!(
        plan.manager_entry
            .contents
            .contains("Exec=\"/usr/lib/aethercodex/aethercodex-manager\"")
    );
}
