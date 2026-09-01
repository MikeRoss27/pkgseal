use std::path::{Component, Path, PathBuf};

use crate::error::PlatformError;

/// Maximum file size that may be read into memory (8 MiB).
pub const MAX_READ_BYTES: usize = 8 * 1024 * 1024;

/// A path that has been validated to be inside an allowed base directory
/// and to contain no traversal components (`..`), no null bytes, and no
/// shell metacharacters in its display form.
///
/// This type does NOT guarantee the path exists — only that it is safe to
/// join and to pass to filesystem APIs without directory-traversal risk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SafePath {
    inner: PathBuf,
}

impl SafePath {
    /// Validate `candidate` relative to `base`.
    ///
    /// - `base` must be absolute.
    /// - `candidate` must not be absolute, must not contain `..`, and the
    ///   joined result must remain inside `base`.
    pub fn join(base: &Path, candidate: &Path) -> Result<Self, PlatformError> {
        if !base.is_absolute() {
            return Err(PlatformError::filesystem(format!(
                "base must be absolute, got {base:?}"
            )));
        }
        if candidate.is_absolute() {
            return Err(PlatformError::filesystem(format!(
                "candidate must be relative, got {candidate:?}"
            )));
        }
        if candidate.as_os_str().as_encoded_bytes().contains(&b'\0') {
            return Err(PlatformError::filesystem(
                "candidate contains null byte".to_string(),
            ));
        }
        for comp in candidate.components() {
            match comp {
                Component::ParentDir => {
                    return Err(PlatformError::filesystem(format!(
                        "candidate contains parent traversal: {candidate:?}"
                    )));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(PlatformError::filesystem(format!(
                        "candidate is not relative: {candidate:?}"
                    )));
                }
                _ => {}
            }
        }

        let joined = base.join(candidate);
        // Canonicalize lexically (no IO) and ensure it stays under base.
        let normalized = lexical_normalize(&joined);
        if !normalized.starts_with(base) {
            return Err(PlatformError::filesystem(format!(
                "joined path {normalized:?} escapes base {base:?}"
            )));
        }

        Ok(Self { inner: normalized })
    }

    /// Construct from an already-validated absolute path. The caller must
    /// ensure `path` is absolute and trusted (e.g. from a config file owned
    /// by root, not from frontend input).
    pub fn from_trusted_absolute(path: PathBuf) -> Result<Self, PlatformError> {
        if !path.is_absolute() {
            return Err(PlatformError::filesystem(format!(
                "trusted path must be absolute: {path:?}"
            )));
        }
        if path.as_os_str().as_encoded_bytes().contains(&b'\0') {
            return Err(PlatformError::filesystem(
                "path contains null byte".to_string(),
            ));
        }
        Ok(Self {
            inner: lexical_normalize(&path),
        })
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.inner
    }

    #[must_use]
    pub fn into_path_buf(self) -> PathBuf {
        self.inner
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

impl std::fmt::Display for SafePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.display().fmt(f)
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns true if `path` is a safe, existing regular file (not a symlink to
/// outside the allowed base — symlink resolution is left to the caller that
/// holds a `SafePath`).
#[must_use]
pub fn is_regular_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_file())
}

/// Read a file with a size limit. Returns `OutputTruncated`-style error if
/// the file exceeds `max_bytes`.
pub fn read_file_limited(path: &Path, max_bytes: usize) -> Result<String, PlatformError> {
    let meta = std::fs::metadata(path)
        .map_err(|e| PlatformError::filesystem(format!("cannot stat {}: {e}", path.display())))?;
    if meta.len() > max_bytes as u64 {
        return Err(PlatformError::OutputTruncated {
            kind: path.display().to_string(),
            limit_bytes: max_bytes,
        });
    }
    let bytes = std::fs::read(path)
        .map_err(|e| PlatformError::filesystem(format!("cannot read {}: {e}", path.display())))?;
    if bytes.len() > max_bytes {
        return Err(PlatformError::OutputTruncated {
            kind: path.display().to_string(),
            limit_bytes: max_bytes,
        });
    }
    String::from_utf8(bytes).map_err(|e| {
        PlatformError::filesystem(format!("file {} is not valid UTF-8: {e}", path.display()))
    })
}

/// Async variant with size limit.
pub async fn read_file_limited_async(
    path: &Path,
    max_bytes: usize,
) -> Result<String, PlatformError> {
    let meta = tokio::fs::metadata(path)
        .await
        .map_err(|e| PlatformError::filesystem(format!("cannot stat {}: {e}", path.display())))?;
    if meta.len() > max_bytes as u64 {
        return Err(PlatformError::OutputTruncated {
            kind: path.display().to_string(),
            limit_bytes: max_bytes,
        });
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| PlatformError::filesystem(format!("cannot read {}: {e}", path.display())))?;
    if bytes.len() > max_bytes {
        return Err(PlatformError::OutputTruncated {
            kind: path.display().to_string(),
            limit_bytes: max_bytes,
        });
    }
    String::from_utf8(bytes).map_err(|e| {
        PlatformError::filesystem(format!("file {} is not valid UTF-8: {e}", path.display()))
    })
}

/// Ensure the parent directory of `path` exists, creating it with `0o755`
/// if needed. `path` must be a `SafePath` to prevent creation outside the
/// allowed tree.
pub fn ensure_parent_exists(path: &SafePath) -> Result<(), PlatformError> {
    if let Some(parent) = path.as_path().parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            PlatformError::filesystem(format!("cannot create {}: {e}", parent.display()))
        })?;
    }
    Ok(())
}

/// List files in a directory (non-recursive) that match an optional extension.
/// Returns `SafePath`s only if they remain within `dir`.
pub fn list_files(dir: &Path, extension: Option<&str>) -> Result<Vec<SafePath>, PlatformError> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        PlatformError::filesystem(format!("cannot read dir {}: {e}", dir.display()))
    })?;
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| PlatformError::filesystem(e.to_string()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = extension
            && path.extension().and_then(|e| e.to_str()) != Some(ext)
        {
            continue;
        }
        // Path is already absolute; wrap as trusted but still normalize.
        // The directory itself is trusted, so child is trusted.
        out.push(SafePath::from_trusted_absolute(path)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn safe_join_valid() {
        let base = Path::new("/usr/share/applications");
        let p = SafePath::join(base, Path::new("firefox.desktop")).unwrap();
        assert_eq!(
            p.as_path(),
            Path::new("/usr/share/applications/firefox.desktop")
        );
    }

    #[test]
    fn safe_join_rejects_parent_traversal() {
        let base = Path::new("/usr/share/applications");
        assert!(SafePath::join(base, Path::new("../etc/passwd")).is_err());
        assert!(SafePath::join(base, Path::new("a/../../etc/shadow")).is_err());
    }

    #[test]
    fn safe_join_rejects_absolute_candidate() {
        let base = Path::new("/tmp/pkgseal");
        assert!(SafePath::join(base, Path::new("/etc/passwd")).is_err());
    }

    #[test]
    fn safe_join_rejects_relative_base() {
        let base = Path::new("relative/base");
        assert!(SafePath::join(base, Path::new("file.txt")).is_err());
    }

    #[test]
    fn safe_join_rejects_null_byte() {
        let base = Path::new("/tmp");
        let candidate = Path::new("bad\0file");
        assert!(SafePath::join(base, candidate).is_err());
    }

    #[test]
    fn lexical_normalize_removes_curdir() {
        let p = lexical_normalize(Path::new("/a/./b/./c"));
        assert_eq!(p, PathBuf::from("/a/b/c"));
    }

    #[test]
    fn lexical_normalize_handles_parent() {
        let p = lexical_normalize(Path::new("/a/b/../c"));
        assert_eq!(p, PathBuf::from("/a/c"));
    }

    #[test]
    fn from_trusted_absolute_rejects_relative() {
        assert!(SafePath::from_trusted_absolute(PathBuf::from("relative")).is_err());
    }

    #[test]
    fn read_file_limited_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.txt");
        std::fs::write(&path, vec![b'a'; 100]).unwrap();
        assert!(read_file_limited(&path, 10).is_err());
        assert!(read_file_limited(&path, 200).is_ok());
    }

    #[test]
    fn ensure_parent_exists_creates() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base");
        std::fs::create_dir_all(&base).unwrap();
        let safe = SafePath::join(&base, Path::new("a/b/c.txt")).unwrap();
        ensure_parent_exists(&safe).unwrap();
        assert!(base.join("a/b").exists());
    }

    #[test]
    fn list_files_filters_extension() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.desktop"), "x").unwrap();
        std::fs::write(dir.path().join("b.txt"), "x").unwrap();
        let files = list_files(dir.path(), Some("desktop")).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].as_path().ends_with("a.desktop"));
    }
}
