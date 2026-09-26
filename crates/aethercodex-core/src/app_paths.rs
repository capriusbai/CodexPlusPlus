use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn find_latest_codex_app_dir(root: &Path) -> Option<PathBuf> {
    let mut matches = std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| version_tuple(&path).map(|version| (version, path)))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| left.0.cmp(&right.0));
    let (_, latest) = matches.pop()?;
    let app = latest.join("app");
    Some(if app.is_dir() { app } else { latest })
}

pub fn find_latest_codex_app_dir_from_roots(roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .filter_map(|root| find_latest_codex_app_dir(root))
        .max_by(|left, right| {
            version_tuple(left.parent().unwrap_or(left))
                .cmp(&version_tuple(right.parent().unwrap_or(right)))
        })
}

pub fn find_latest_codex_app_dir_default() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        find_latest_codex_app_dir_from_roots(&windows_app_package_roots())
    }

    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn windows_app_package_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        roots.push(PathBuf::from(program_files).join("WindowsApps"));
    }
    if let Some(program_files) = std::env::var_os("ProgramW6432") {
        roots.push(PathBuf::from(program_files).join("WindowsApps"));
    }
    roots.push(PathBuf::from(r"C:\Program Files\WindowsApps"));
    roots.sort();
    roots.dedup();
    roots
}

/// Executable names an unpacked Codex App uses on Linux/BSD. Windows ships
/// `Codex.exe`, macOS ships `Codex` inside the bundle, and the Linux builds
/// ship an extension-less binary next to the Electron resources.
pub const UNIX_CODEX_BINARY_NAMES: [&str; 3] = ["Codex", "codex", "codex-app"];

/// Directory names a Codex App install is commonly unpacked into on Linux.
const LINUX_CODEX_APP_DIR_NAMES: [&str; 6] = [
    "Codex",
    "codex",
    "OpenAI/Codex",
    "openai-codex",
    "codex-app",
    "Codex/app",
];

pub fn linux_app_search_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/opt"),
        PathBuf::from("/usr/lib"),
        PathBuf::from("/usr/lib64"),
        PathBuf::from("/usr/share"),
        PathBuf::from("/usr/local/lib"),
        PathBuf::from("/usr/local/share"),
    ];
    if let Some(home) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()) {
        roots.push(home.join(".local/share"));
        roots.push(home.join(".local/lib"));
        roots.push(home.join("Applications"));
        roots.push(home.join("applications"));
        roots.push(home.join(".local/bin"));
    }
    roots
}

/// Look for an unpacked Codex App (or a Codex `.AppImage`) under `search_roots`.
///
/// Only directories that actually hold a Codex executable are accepted, so a
/// bare `codex` CLI on `PATH` is never mistaken for the desktop app.
pub fn find_linux_codex_app(search_roots: &[PathBuf]) -> Option<PathBuf> {
    for root in search_roots {
        for candidate in linux_app_candidates(root) {
            if candidate.is_dir() && unix_codex_binary_in(&candidate).is_some() {
                let nested = candidate.join("app");
                if unix_codex_binary_in(&nested).is_some() {
                    return Some(nested);
                }
                return Some(candidate);
            }
        }
        if let Some(app_image) = find_codex_app_image(root) {
            return Some(app_image);
        }
    }
    None
}

pub fn find_linux_codex_app_default() -> Option<PathBuf> {
    find_linux_codex_app(&linux_app_search_roots())
}

fn linux_app_candidates(root: &Path) -> Vec<PathBuf> {
    if is_app_image(root) {
        return vec![root.to_path_buf()];
    }
    let mut candidates = vec![root.to_path_buf()];
    candidates.extend(LINUX_CODEX_APP_DIR_NAMES.iter().map(|name| root.join(name)));
    candidates
}

fn find_codex_app_image(root: &Path) -> Option<PathBuf> {
    if is_app_image(root) {
        return root.is_file().then(|| root.to_path_buf());
    }
    let mut matches = std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_app_image(path))
        .filter(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.to_ascii_lowercase().contains("codex"))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.pop()
}

pub fn is_app_image(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("AppImage"))
}

fn unix_codex_binary_in(app_dir: &Path) -> Option<PathBuf> {
    UNIX_CODEX_BINARY_NAMES
        .iter()
        .map(|name| app_dir.join(name))
        .find(|candidate| candidate.is_file())
}

pub fn user_data_candidates() -> Vec<PathBuf> {
    user_data_candidates_from(
        std::env::var_os("LOCALAPPDATA").as_deref().map(Path::new),
        std::env::var_os("APPDATA").as_deref().map(Path::new),
    )
}

pub fn user_data_candidates_from(local: Option<&Path>, roaming: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(local) = local {
        append_user_data_variants(&mut candidates, local);
    }
    if let Some(roaming) = roaming {
        append_user_data_variants(&mut candidates, roaming);
    }
    candidates
}

pub fn find_macos_codex_app(search_roots: &[PathBuf]) -> Option<PathBuf> {
    for root in search_roots {
        for candidate in macos_app_candidates(root) {
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn find_macos_codex_app_default() -> Option<PathBuf> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(home) = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()) {
        roots.push(home.join("Applications"));
    }
    find_macos_codex_app(&roots)
}

pub fn resolve_codex_app_dir(app_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(app_dir) = app_dir {
        return normalize_codex_app_path(app_dir);
    }
    if cfg!(target_os = "macos") {
        return find_macos_codex_app_default();
    }
    if cfg!(all(unix, not(target_os = "macos"))) {
        return find_linux_codex_app_default();
    }
    // Windows: try MS Store version first, then standalone install
    find_latest_codex_app_dir_default().or_else(|| find_standalone_codex_app_dir())
}

/// Search for standalone Codex installations (non-MS Store).
///
/// Common paths:
/// - %LOCALAPPDATA%\OpenAI\Codex\bin\  (standalone installer)
/// - %LOCALAPPDATA%\OpenAI\Codex\      (user data root)
/// - %LOCALAPPDATA%\Programs\OpenAI\Codex\ (alternative)
pub fn find_standalone_codex_app_dir() -> Option<PathBuf> {
    let local_appdata = std::env::var_os("LOCALAPPDATA")?;

    let candidates: &[PathBuf] = &[
        PathBuf::from(&local_appdata)
            .join("OpenAI")
            .join("Codex")
            .join("bin"),
        PathBuf::from(&local_appdata).join("OpenAI").join("Codex"),
        PathBuf::from(&local_appdata)
            .join("Programs")
            .join("OpenAI")
            .join("Codex"),
    ];

    for candidate in candidates {
        if let Some(path) = normalize_codex_app_path(candidate) {
            if build_codex_executable(&path).exists() {
                return Some(path);
            }
        }
    }
    None
}

pub fn resolve_codex_app_dir_with_saved(
    app_dir: Option<&Path>,
    saved_app_path: Option<&str>,
) -> Option<PathBuf> {
    if let Some(app_dir) = app_dir {
        return normalize_codex_app_path(app_dir);
    }
    if let Some(saved) = saved_app_path
        .map(str::trim)
        .filter(|saved| !saved.is_empty())
    {
        if let Some(path) = normalize_codex_app_path(Path::new(saved)) {
            return Some(path);
        }
    }
    resolve_codex_app_dir(None)
}

pub fn normalize_codex_app_path(path: &Path) -> Option<PathBuf> {
    if path.as_os_str().is_empty() {
        return None;
    }

    let file_name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
    if file_name.eq_ignore_ascii_case("Codex.exe") || file_name.eq_ignore_ascii_case("codex.exe") {
        return path.parent().map(Path::to_path_buf);
    }

    if path.extension() == Some(OsStr::new("app")) {
        return Some(path.to_path_buf());
    }

    // A Codex `.AppImage` is a single self-contained executable, so it acts as
    // its own app directory.
    if is_app_image(path) {
        return Some(path.to_path_buf());
    }

    if path.is_file() {
        return path.parent().map(Path::to_path_buf);
    }

    let upper = path.join("Codex.exe");
    let lower = path.join("codex.exe");
    if upper.exists() || lower.exists() || unix_codex_binary_in(path).is_some() {
        return Some(path.to_path_buf());
    }

    let nested_app = path.join("app");
    if nested_app.is_dir() {
        let upper = nested_app.join("Codex.exe");
        let lower = nested_app.join("codex.exe");
        if upper.exists() || lower.exists() || unix_codex_binary_in(&nested_app).is_some() {
            return Some(nested_app);
        }
    }

    if path.is_dir() {
        return Some(path.to_path_buf());
    }

    None
}

pub fn build_codex_executable(app_dir: &Path) -> PathBuf {
    if app_dir.extension() == Some(OsStr::new("app")) {
        return app_dir.join("Contents").join("MacOS").join("Codex");
    }
    if is_app_image(app_dir) {
        return app_dir.to_path_buf();
    }
    let upper = app_dir.join("Codex.exe");
    if upper.exists() {
        return upper;
    }
    let lower = app_dir.join("codex.exe");
    if lower.exists() || cfg!(windows) {
        return lower;
    }
    unix_codex_binary_in(app_dir).unwrap_or_else(|| app_dir.join("codex"))
}

pub fn codex_app_version(app_dir: &Path) -> Option<String> {
    if app_dir.extension() == Some(OsStr::new("app")) {
        return macos_app_version(app_dir);
    }
    let package_dir = if app_dir
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.eq_ignore_ascii_case("app"))
    {
        app_dir.parent()?
    } else {
        app_dir
    };
    codex_package_version(package_dir)
}

pub fn packaged_app_user_model_id(app_dir: &Path) -> Option<String> {
    let package_name = package_name_from_app_dir(app_dir)?;
    if !package_name.starts_with("OpenAI.Codex_") || !package_name.contains("__") {
        return None;
    }
    let identity_name = package_name.split_once('_')?.0;
    let publisher_id = package_name.rsplit_once("__")?.1;
    if publisher_id.is_empty() {
        return None;
    }
    Some(format!("{identity_name}_{publisher_id}!App"))
}

fn package_name_from_app_dir(app_dir: &Path) -> Option<String> {
    let path = app_dir.to_string_lossy().replace('\\', "/");
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    let mut package_name = parts.next_back()?;
    if package_name.eq_ignore_ascii_case("app") {
        package_name = parts.next_back()?;
    }
    Some(package_name.to_string())
}

fn codex_package_version(package_dir: &Path) -> Option<String> {
    let path = package_dir.to_string_lossy().replace('\\', "/");
    let name = path
        .split('/')
        .rev()
        .find(|part| part.starts_with("OpenAI.Codex_"))?;
    let rest = name.strip_prefix("OpenAI.Codex_")?;
    let version = rest.split_once('_')?.0;
    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

fn macos_app_version(app_dir: &Path) -> Option<String> {
    let plist = std::fs::read_to_string(app_dir.join("Contents").join("Info.plist")).ok()?;
    plist_string_value(&plist, "CFBundleShortVersionString")
        .or_else(|| plist_string_value(&plist, "CFBundleVersion"))
}

fn plist_string_value(plist: &str, key: &str) -> Option<String> {
    let (_, after_key) = plist.split_once(&format!("<key>{key}</key>"))?;
    let (_, after_string_open) = after_key.split_once("<string>")?;
    let (value, _) = after_string_open.split_once("</string>")?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn append_user_data_variants(candidates: &mut Vec<PathBuf>, base: &Path) {
    candidates.push(base.join("OpenAI").join("Codex"));
    candidates.push(base.join("OpenAI.Codex"));
    candidates.push(base.join("Codex"));
}

fn macos_app_candidates(root: &Path) -> Vec<PathBuf> {
    if root.extension() == Some(OsStr::new("app")) {
        return vec![root.to_path_buf()];
    }
    ["Codex.app", "OpenAI Codex.app", "OpenAI.Codex.app"]
        .into_iter()
        .map(|name| root.join(name))
        .collect()
}

fn version_tuple(path: &Path) -> Option<Vec<u32>> {
    let name = path.file_name()?.to_str()?;
    let rest = name.strip_prefix("OpenAI.Codex_")?;
    let version = rest.split_once('_')?.0;
    let parts = version
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if parts.is_empty() { None } else { Some(parts) }
}
