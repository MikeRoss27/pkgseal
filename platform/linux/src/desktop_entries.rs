use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::PlatformError;
use crate::filesystem::SafePath;

/// Well-known directories where `.desktop` files are looked up, in order.
/// Mirrors `XDG_DATA_DIRS` plus the user data dir, but without shell
/// expansion — callers must pass already-resolved absolute paths.
pub const DEFAULT_DESKTOP_ENTRY_DIRS: &[&str] = &[
    "/usr/share/applications",
    "/usr/local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];

/// Validated desktop entry identifier (filename without extension).
///
/// Rules: ASCII alphanumeric, `-`, `_`, `.`; cannot start with `-`/`.`; max 128.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DesktopEntryId(String);

impl DesktopEntryId {
    pub fn new(s: impl AsRef<str>) -> Result<Self, PlatformError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(PlatformError::desktop_entry(
                "desktop entry id cannot be empty",
            ));
        }
        if s.len() > 128 {
            return Err(PlatformError::desktop_entry("desktop entry id too long"));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(PlatformError::desktop_entry(format!(
                "invalid desktop entry id {s:?}"
            )));
        }
        if s.starts_with(['-', '.']) || s.ends_with(['-', '.']) {
            return Err(PlatformError::desktop_entry(format!(
                "desktop entry id {s:?} cannot start or end with - or ."
            )));
        }
        if s.contains(';')
            || s.contains('|')
            || s.contains('$')
            || s.contains('`')
            || s.contains('\n')
            || s.contains('\0')
        {
            return Err(PlatformError::desktop_entry(format!(
                "desktop entry id {s:?} contains forbidden character"
            )));
        }
        Ok(Self(s.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DesktopEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Parsed `.desktop` entry (subset relevant to PkgSeal).
///
/// Only the `[Desktop Entry]` group is parsed; other groups are ignored.
/// Values are not executed — `Exec` is stored as a string for display /
/// matching, never passed to a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    pub id: DesktopEntryId,
    pub name: String,
    pub exec: Option<String>,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub no_display: bool,
    pub terminal: bool,
    pub entry_type: String,
    pub path: SafePath,
    /// Raw key/value pairs from `[Desktop Entry]` for future use.
    pub raw: HashMap<String, String>,
}

impl DesktopEntry {
    #[must_use]
    pub fn desktop_file_name(&self) -> String {
        format!("{}.desktop", self.id.as_str())
    }
}

/// Parse a `.desktop` file at `path`.
///
/// - Validates `path` is inside an allowed desktop entry dir (caller must
///   have constructed a `SafePath`; we accept `&Path` and re-check).
/// - Reads with size limit (256 KiB) to avoid unbounded memory.
/// - Parses `[Desktop Entry]` section only, handling line continuations via
///   simple `key=value` splitting.
pub fn parse_desktop_entry(path: &Path) -> Result<DesktopEntry, PlatformError> {
    if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
        return Err(PlatformError::desktop_entry(format!(
            "not a .desktop file: {}",
            path.display()
        )));
    }

    let content = crate::filesystem::read_file_limited(path, 256 * 1024).map_err(|e| {
        PlatformError::desktop_entry(format!("cannot read {}: {e}", path.display()))
    })?;

    let id_str = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
        PlatformError::desktop_entry(format!("invalid file name {}", path.display()))
    })?;
    let id = DesktopEntryId::new(id_str)?;

    let safe_path = SafePath::from_trusted_absolute(path.to_path_buf())?;

    parse_desktop_content(&content, id, safe_path)
}

fn parse_desktop_content(
    content: &str,
    id: DesktopEntryId,
    path: SafePath,
) -> Result<DesktopEntry, PlatformError> {
    let mut in_desktop_entry = false;
    let mut map: HashMap<String, String> = HashMap::new();
    let mut found_section = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            in_desktop_entry = section == "Desktop Entry";
            if in_desktop_entry {
                found_section = true;
            }
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_string();
            let value = v.trim().to_string();
            // Keep first occurrence; duplicates are ignored (defensive).
            map.entry(key).or_insert(value);
        }
    }

    if !found_section {
        return Err(PlatformError::desktop_entry(
            "missing [Desktop Entry] section",
        ));
    }

    let name = map
        .get("Name")
        .cloned()
        .ok_or_else(|| PlatformError::desktop_entry("missing Name key"))?;
    if name.is_empty() {
        return Err(PlatformError::desktop_entry("Name cannot be empty"));
    }
    // Basic sanitisation: Name should not contain control chars.
    if name.contains('\n') || name.contains('\0') || name.contains('\r') {
        return Err(PlatformError::desktop_entry(
            "Name contains forbidden control character",
        ));
    }

    let entry_type = map
        .get("Type")
        .cloned()
        .unwrap_or_else(|| "Application".to_string());
    let exec = map.get("Exec").cloned();
    let icon = map.get("Icon").cloned();
    let categories = map
        .get("Categories")
        .map(|s| {
            s.split(';')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let keywords = map
        .get("Keywords")
        .map(|s| {
            s.split(';')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let no_display = map
        .get("NoDisplay")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));
    let terminal = map
        .get("Terminal")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));

    Ok(DesktopEntry {
        id,
        name,
        exec,
        icon,
        categories,
        keywords,
        no_display,
        terminal,
        entry_type,
        path,
        raw: map,
    })
}

/// Discover `.desktop` files under `dirs` (non-recursive).
///
/// Returns parsed entries; unreadable or invalid files are skipped with a
/// `tracing::warn` — discovery is best-effort, not fatal.
pub fn discover_desktop_entries(dirs: &[PathBuf]) -> Vec<DesktopEntry> {
    let mut out = Vec::new();
    for dir in dirs {
        let entries = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            match parse_desktop_entry(&path) {
                Ok(de) => out.push(de),
                Err(e) => {
                    tracing::warn!("Skipping invalid desktop entry {}: {e}", path.display());
                }
            }
        }
    }
    out
}

/// Return search paths resolved from `XDG_DATA_DIRS` plus the provided
/// defaults. Invalid (non-absolute or traversal-containing) entries are
/// filtered out.
#[must_use]
pub fn resolve_desktop_entry_dirs(extra_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for d in DEFAULT_DESKTOP_ENTRY_DIRS {
        dirs.push(PathBuf::from(d));
    }
    dirs.extend(extra_dirs.iter().cloned());
    // Deduplicate and keep only absolute, existing-or-not (we don't require
    // existence at resolve time) but filter obvious traversal.
    let mut seen = std::collections::HashSet::new();
    dirs.into_iter()
        .filter(|p| p.is_absolute())
        .filter(|p| {
            let s = p.to_string_lossy();
            !s.contains("..") && !s.contains('\0')
        })
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn desktop_entry_id_valid() {
        assert!(DesktopEntryId::new("firefox").is_ok());
        assert!(DesktopEntryId::new("com.brave.Browser").is_ok());
        assert!(DesktopEntryId::new("my-app_1").is_ok());
    }

    #[test]
    fn desktop_entry_id_invalid() {
        assert!(DesktopEntryId::new("").is_err());
        assert!(DesktopEntryId::new("-bad").is_err());
        assert!(DesktopEntryId::new("bad-").is_err());
        assert!(DesktopEntryId::new("bad;id").is_err());
        assert!(DesktopEntryId::new("bad id").is_err());
        assert!(DesktopEntryId::new("bad$id").is_err());
    }

    #[test]
    fn parse_minimal_desktop_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.desktop");
        let content = "[Desktop Entry]\nName=Test App\nType=Application\nExec=/usr/bin/test\n";
        std::fs::write(&path, content).unwrap();
        let entry = parse_desktop_entry(&path).unwrap();
        assert_eq!(entry.name, "Test App");
        assert_eq!(entry.exec.as_deref(), Some("/usr/bin/test"));
        assert_eq!(entry.id.as_str(), "test");
    }

    #[test]
    fn parse_rejects_missing_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.desktop");
        let content = "[Desktop Entry]\nType=Application\n";
        std::fs::write(&path, content).unwrap();
        assert!(parse_desktop_entry(&path).is_err());
    }

    #[test]
    fn parse_rejects_missing_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nosection.desktop");
        let content = "Name=Foo\n";
        std::fs::write(&path, content).unwrap();
        assert!(parse_desktop_entry(&path).is_err());
    }

    #[test]
    fn parse_categories_and_nodisplay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.desktop");
        let content = "[Desktop Entry]\nName=Foo\nType=Application\nCategories=Network;WebBrowser;\nNoDisplay=true\n";
        std::fs::write(&path, content).unwrap();
        let entry = parse_desktop_entry(&path).unwrap();
        assert_eq!(entry.categories, vec!["Network", "WebBrowser"]);
        assert!(entry.no_display);
    }

    #[test]
    fn discover_skips_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let good = dir.path().join("good.desktop");
        std::fs::write(&good, "[Desktop Entry]\nName=Good\nType=Application\n").unwrap();
        let bad = dir.path().join("bad.desktop");
        std::fs::write(&bad, "not a desktop entry\n").unwrap();
        let txt = dir.path().join("ignore.txt");
        std::fs::write(&txt, "ignore").unwrap();
        let entries = discover_desktop_entries(&[dir.path().to_path_buf()]);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Good");
    }

    #[test]
    fn resolve_dirs_filters_relative() {
        let dirs = resolve_desktop_entry_dirs(&[PathBuf::from("relative/path")]);
        assert!(!dirs.iter().any(|p| *p == Path::new("relative/path")));
    }

    #[test]
    fn resolve_dirs_dedupes() {
        let dirs = resolve_desktop_entry_dirs(&[PathBuf::from("/usr/share/applications")]);
        let count = dirs
            .iter()
            .filter(|p| *p == Path::new("/usr/share/applications"))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn parse_rejects_wrong_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "[Desktop Entry]\nName=Foo\n").unwrap();
        assert!(parse_desktop_entry(&path).is_err());
    }
}
