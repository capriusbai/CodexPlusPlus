use aethercodex_core::update::{
    Release, download_asset_to, is_newer_version, parse_version_tag, release_from_github_payload,
    release_from_latest_json_payload, safe_asset_name, select_update_asset,
};
use serde_json::json;

#[test]
fn parse_version_tag_accepts_prefix_and_suffix() {
    assert_eq!(parse_version_tag("v1.2.3").unwrap(), vec![1, 2, 3]);
    assert_eq!(parse_version_tag("1.2.3").unwrap(), vec![1, 2, 3]);
    assert_eq!(parse_version_tag("v1.2.3-beta.1").unwrap(), vec![1, 2, 3]);
}

#[test]
fn version_comparison_uses_numeric_segments() {
    assert!(is_newer_version("v1.0.10", "1.0.4").unwrap());
    assert!(!is_newer_version("v1.0.4", "1.0.4").unwrap());
    assert!(!is_newer_version("v1.0.3", "1.0.4").unwrap());
}

#[test]
fn github_payload_selects_platform_installer() {
    let release = release_from_github_payload(&json!({
        "tag_name": "v1.0.9",
        "html_url": "https://github.com/capriusbai/CodexPlusPlus/releases/tag/v1.0.9",
        "body": "fixes",
        "assets": [
            {"name": "source.zip", "browser_download_url": "https://example.test/source.zip"},
            {"name": "aethercodex-manager.exe", "browser_download_url": "https://example.test/manager.exe"},
            {"name": "AetherCodex_1.0.9_x64-setup.exe", "browser_download_url": "https://example.test/setup.exe"},
            {"name": "AetherCodex_1.0.9_x64.dmg", "browser_download_url": "https://example.test/app.dmg"},
            {"name": "AetherCodex-1.0.9-linux-x64.deb", "browser_download_url": "https://example.test/app.deb"},
            {"name": "AetherCodex-1.0.9-linux-arm64.deb", "browser_download_url": "https://example.test/app-arm64.deb"}
        ]
    }))
    .unwrap();

    assert_eq!(release.version, "v1.0.9");
    if cfg!(windows) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex_1.0.9_x64-setup.exe")
        );
    } else if cfg!(target_os = "macos") {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex_1.0.9_x64.dmg")
        );
    } else if cfg!(all(unix, target_arch = "x86_64")) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.0.9-linux-x64.deb")
        );
    } else if cfg!(all(unix, target_arch = "aarch64")) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.0.9-linux-arm64.deb")
        );
    } else {
        assert_eq!(release.asset_name.as_deref(), None);
    }
}

#[test]
fn latest_json_payload_selects_platform_installer_without_github_api_shape() {
    let release = release_from_latest_json_payload(&json!({
        "version": "v1.1.6",
        "url": "https://github.com/capriusbai/CodexPlusPlus/releases/tag/v1.1.6",
        "body": "静态更新描述",
        "assets": [
            {"name": "source.zip", "url": "https://example.test/source.zip"},
            {"name": "AetherCodex-1.1.6-windows-x64-setup.exe", "url": "https://example.test/setup.exe"},
            {"name": "AetherCodex-1.1.6-macos-x64.dmg", "url": "https://example.test/app.dmg"},
            {"name": "AetherCodex-1.1.6-linux-x64.tar.gz", "url": "https://example.test/app-x64.tar.gz"},
            {"name": "AetherCodex-1.1.6-linux-arm64.tar.gz", "url": "https://example.test/app-arm64.tar.gz"}
        ]
    }))
    .unwrap();

    assert_eq!(release.version, "v1.1.6");
    assert_eq!(release.body, "静态更新描述");
    if cfg!(windows) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.1.6-windows-x64-setup.exe")
        );
    } else if cfg!(target_os = "macos") {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.1.6-macos-x64.dmg")
        );
    } else if cfg!(all(unix, target_arch = "x86_64")) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.1.6-linux-x64.tar.gz")
        );
    } else if cfg!(all(unix, target_arch = "aarch64")) {
        assert_eq!(
            release.asset_name.as_deref(),
            Some("AetherCodex-1.1.6-linux-arm64.tar.gz")
        );
    } else {
        assert_eq!(release.asset_name.as_deref(), None);
    }
}

#[test]
fn asset_selection_prefers_current_platform_artifacts() {
    let assets = vec![
        (
            "AetherCodex.zip".to_string(),
            "https://example.test/source.zip".to_string(),
        ),
        (
            "aethercodex-manager.exe".to_string(),
            "https://example.test/manager.exe".to_string(),
        ),
        (
            "AetherCodex_1.0.9_x64-setup.exe".to_string(),
            "https://example.test/setup.exe".to_string(),
        ),
        (
            "AetherCodex_1.0.9_x64.dmg".to_string(),
            "https://example.test/app.dmg".to_string(),
        ),
    ];

    if cfg!(windows) {
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex_1.0.9_x64-setup.exe");
    } else if cfg!(target_os = "macos") {
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex_1.0.9_x64.dmg");
    } else {
        assert!(select_update_asset(&assets).is_none());
    }
}

#[test]
fn linux_asset_selection_prefers_the_matching_architecture_package() {
    let assets = vec![
        (
            "AetherCodex-1.2.5-linux-x64.tar.gz".to_string(),
            "https://example.test/x64.tar.gz".to_string(),
        ),
        (
            "AetherCodex-1.2.5-linux-arm64.tar.gz".to_string(),
            "https://example.test/arm64.tar.gz".to_string(),
        ),
        (
            "AetherCodex-1.2.5-linux-x64.deb".to_string(),
            "https://example.test/x64.deb".to_string(),
        ),
        (
            "AetherCodex-1.2.5-linux-arm64.deb".to_string(),
            "https://example.test/arm64.deb".to_string(),
        ),
    ];

    if cfg!(all(unix, not(target_os = "macos"), target_arch = "x86_64")) {
        // The tarballs come first in the list, but the `.deb` still wins.
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex-1.2.5-linux-x64.deb");
    } else if cfg!(all(unix, not(target_os = "macos"), target_arch = "aarch64")) {
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex-1.2.5-linux-arm64.deb");
    } else {
        assert!(select_update_asset(&assets).is_none());
    }
}

#[test]
fn linux_asset_selection_falls_back_to_the_tarball_when_no_package_matches() {
    let assets = vec![
        (
            "AetherCodex-1.2.5-linux-x64.tar.gz".to_string(),
            "https://example.test/x64.tar.gz".to_string(),
        ),
        (
            "AetherCodex-1.2.5-linux-arm64.tar.gz".to_string(),
            "https://example.test/arm64.tar.gz".to_string(),
        ),
    ];

    if cfg!(all(unix, not(target_os = "macos"), target_arch = "x86_64")) {
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex-1.2.5-linux-x64.tar.gz");
    } else if cfg!(all(unix, not(target_os = "macos"), target_arch = "aarch64")) {
        let selected = select_update_asset(&assets).unwrap();
        assert_eq!(selected.name, "AetherCodex-1.2.5-linux-arm64.tar.gz");
    } else {
        assert!(select_update_asset(&assets).is_none());
    }
}

#[test]
fn safe_asset_name_rejects_path_traversal() {
    assert_eq!(safe_asset_name("pkg.zip").unwrap(), "pkg.zip");
    assert!(safe_asset_name("../pkg.zip").is_err());
    assert!(safe_asset_name("").is_err());
}

#[test]
fn download_asset_to_writes_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let release = Release {
        version: "v1.0.9".to_string(),
        url: "https://example.test".to_string(),
        body: "fixes".to_string(),
        asset_name: Some("pkg.zip".to_string()),
        asset_url: Some("https://example.test/pkg.zip".to_string()),
    };

    let path = download_asset_to(&release, b"abcdef", dir.path()).unwrap();

    assert_eq!(path, dir.path().join("pkg.zip"));
    assert_eq!(std::fs::read(path).unwrap(), b"abcdef");
}
