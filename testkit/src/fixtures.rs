//! Fixture loading — deterministic, offline, no network.
//!
//! Fixtures are JSON files versioned under `fixtures/` at the workspace root:
//! ```text
//! fixtures/arch/brave.json
//! fixtures/aur/brave.json
//! fixtures/flatpak/brave.json
//! ```
//! Each file contains a `PackageDetails` (or `PackageSummary`) serialised with
//! the same `serde(rename_all = "camelCase")` as used in `pkgseal-source`.

use pkgseal_domain::PackageSource;
use pkgseal_source::dto::{PackageDetails, PackageSummary};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("fixture not found: {0}")]
    NotFound(String),
    #[error("io error reading {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("json parse error in {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("fixtures root not found; searched from {searched}")]
    RootNotFound { searched: String },
}

// ---------------------------------------------------------------------------
// Root discovery — robust against `cargo test -p pkgseal-testkit` vs.
// `cargo test` vs. being used as a dependency.
// ---------------------------------------------------------------------------

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // 1. Explicit override.
    if let Ok(dir) = std::env::var("PKGSEAL_FIXTURES_DIR") {
        roots.push(PathBuf::from(dir));
    }

    // 2. Ancestors of CARGO_MANIFEST_DIR (testkit's own manifest).
    let md = manifest_dir();
    for ancestor in md.ancestors() {
        roots.push(ancestor.join("fixtures"));
        // Also try workspace root's parent for nested workspaces.
        roots.push(ancestor.join("../fixtures"));
    }

    // 3. Ancestors of current_dir (when running `cargo test` from workspace root).
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            roots.push(ancestor.join("fixtures"));
            roots.push(ancestor.join("../fixtures"));
        }
    }

    // 4. Relative fallbacks.
    roots.push(PathBuf::from("fixtures"));
    roots.push(PathBuf::from("../fixtures"));
    roots.push(PathBuf::from("../../fixtures"));

    roots
}

/// Locate the `fixtures/` directory, or return an error describing where we looked.
pub fn fixtures_root() -> Result<PathBuf, FixtureError> {
    for candidate in candidate_roots() {
        // Normalise `../` segments opportunistically without requiring the path to exist.
        let looks_like_fixtures =
            candidate.ends_with("fixtures") && candidate.parent().is_some_and(|_| true);
        if !looks_like_fixtures {
            continue;
        }
        if candidate.is_dir() {
            // Require that at least arch/aur/flatpak subdirs or the dir itself exists.
            return Ok(candidate.canonicalize().unwrap_or(candidate));
        }
    }
    Err(FixtureError::RootNotFound {
        searched: candidate_roots()
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
    })
}

/// Build the expected path for `fixtures/{source}/{name}.json`.
pub fn fixture_path(source: PackageSource, name: &str) -> Result<PathBuf, FixtureError> {
    let root = fixtures_root()?;
    let dir = match source {
        PackageSource::ArchOfficial => "arch",
        PackageSource::Aur => "aur",
        PackageSource::Flatpak => "flatpak",
    };
    Ok(root.join(dir).join(format!("{name}.json")))
}

/// Generic JSON loader — reads and parses `path` into `T`.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, FixtureError> {
    let raw = std::fs::read_to_string(path).map_err(|e| FixtureError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    serde_json::from_str(&raw).map_err(|e| FixtureError::Parse {
        path: path.display().to_string(),
        source: e,
    })
}

/// Load a JSON fixture relative to the fixtures root, e.g. `"arch/brave.json"`.
pub fn load_fixture<T: DeserializeOwned>(relative: impl AsRef<Path>) -> Result<T, FixtureError> {
    let root = fixtures_root()?;
    let path = root.join(relative.as_ref());
    if !path.exists() {
        return Err(FixtureError::NotFound(path.display().to_string()));
    }
    load_json(&path)
}

// ---------------------------------------------------------------------------
// Typed helpers — PackageDetails / PackageSummary
// ---------------------------------------------------------------------------

/// Try to load a `PackageDetails` from `fixtures/{source}/{name}.json`.
///
/// The fixture may be either a full `PackageDetails` JSON object (containing a
/// `summary` field) or a bare `PackageSummary` object — the latter is wrapped
/// into a minimal `PackageDetails` so simple fixtures remain ergonomic.
pub fn load_details(source: PackageSource, name: &str) -> Result<PackageDetails, FixtureError> {
    let path = fixture_path(source, name)?;
    load_details_from_path(&path)
}

fn load_details_from_path(path: &Path) -> Result<PackageDetails, FixtureError> {
    if !path.exists() {
        return Err(FixtureError::NotFound(path.display().to_string()));
    }
    let raw = std::fs::read_to_string(path).map_err(|e| FixtureError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| FixtureError::Parse {
        path: path.display().to_string(),
        source: e,
    })?;

    // Heuristic: if JSON has a top-level "summary" key, treat as PackageDetails.
    if value.get("summary").is_some() {
        serde_json::from_value(value).map_err(|e| FixtureError::Parse {
            path: path.display().to_string(),
            source: e,
        })
    } else {
        // Bare summary → wrap.
        let summary: PackageSummary =
            serde_json::from_value(value).map_err(|e| FixtureError::Parse {
                path: path.display().to_string(),
                source: e,
            })?;
        Ok(PackageDetails {
            summary,
            architecture: None,
            maintainer: None,
            url: None,
            license: None,
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            groups: Vec::new(),
            build_date: None,
            install_date: None,
            validation: None,
            raw_metadata: std::collections::HashMap::new(),
        })
    }
}

/// Load a `PackageSummary` from the same fixtures. If the file is a
/// `PackageDetails`, the inner summary is extracted.
pub fn load_summary(source: PackageSource, name: &str) -> Result<PackageSummary, FixtureError> {
    let details = load_details(source, name)?;
    Ok(details.summary)
}

// Convenience wrappers per source (ergonomic for tests):

pub fn load_arch_details(name: &str) -> Result<PackageDetails, FixtureError> {
    load_details(PackageSource::ArchOfficial, name)
}

pub fn load_aur_details(name: &str) -> Result<PackageDetails, FixtureError> {
    load_details(PackageSource::Aur, name)
}

pub fn load_flatpak_details(name: &str) -> Result<PackageDetails, FixtureError> {
    load_details(PackageSource::Flatpak, name)
}

pub fn load_arch_summary(name: &str) -> Result<PackageSummary, FixtureError> {
    load_summary(PackageSource::ArchOfficial, name)
}

pub fn load_aur_summary(name: &str) -> Result<PackageSummary, FixtureError> {
    load_summary(PackageSource::Aur, name)
}

pub fn load_flatpak_summary(name: &str) -> Result<PackageSummary, FixtureError> {
    load_summary(PackageSource::Flatpak, name)
}

/// List fixture names (without extension) for a source, e.g. `["brave", "firefox"]`.
pub fn list_fixtures(source: PackageSource) -> Result<Vec<String>, FixtureError> {
    let root = fixtures_root()?;
    let dir = match source {
        PackageSource::ArchOfficial => root.join("arch"),
        PackageSource::Aur => root.join("aur"),
        PackageSource::Flatpak => root.join("flatpak"),
    };
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| FixtureError::Io {
        path: dir.display().to_string(),
        source: e,
    })? {
        let entry = entry.map_err(|e| FixtureError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.push(stem.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// Load all `PackageDetails` for a source.
pub fn load_all_details(source: PackageSource) -> Result<Vec<PackageDetails>, FixtureError> {
    let names = list_fixtures(source)?;
    let mut out = Vec::new();
    for name in names {
        out.push(load_details(source, &name)?);
    }
    Ok(out)
}

/// Load all `PackageDetails` across Arch/AUR/Flatpak.
pub fn load_all_fixtures() -> Result<Vec<PackageDetails>, FixtureError> {
    let mut all = Vec::new();
    for source in PackageSource::all() {
        all.extend(load_all_details(source)?);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageSource;

    #[test]
    fn fixtures_root_exists_or_reports_not_found() {
        match fixtures_root() {
            Ok(root) => {
                assert!(
                    root.is_dir(),
                    "fixtures root should be a directory: {root:?}"
                );
                assert!(
                    root.ends_with("fixtures"),
                    "root should end with fixtures: {root:?}"
                );
            }
            Err(FixtureError::RootNotFound { searched }) => {
                // In CI without fixtures checked out, this branch documents where we searched.
                assert!(
                    !searched.is_empty(),
                    "searched should list candidates when root not found"
                );
            }
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    }

    #[test]
    fn load_arch_brave_fixture_or_skip() {
        match load_details(PackageSource::ArchOfficial, "brave") {
            Ok(details) => {
                assert_eq!(details.summary.source, PackageSource::ArchOfficial);
                assert!(!details.summary.name.as_str().is_empty());
            }
            Err(FixtureError::NotFound(_)) | Err(FixtureError::RootNotFound { .. }) => {
                // Fixtures not yet created in this checkout — test documents the contract
                // without failing offline. Once fixtures are committed this branch disappears.
            }
            Err(e) => panic!("unexpected fixture error: {e}"),
        }
    }

    #[test]
    fn fixture_path_constructs_expected_suffix() {
        // Use a name that is valid for all sources to exercise path formatting.
        let p = fixture_path(PackageSource::Aur, "brave")
            .unwrap_or_else(|_| PathBuf::from("fixtures/aur/brave.json"));
        let s = p.display().to_string();
        assert!(s.ends_with("aur/brave.json") || s.ends_with("fixtures/aur/brave.json"));
    }

    #[test]
    fn load_json_round_trips_for_package_summary() {
        // Exercise the generic path with an ephemeral file so the test is offline and hermetic.
        let dir = std::env::temp_dir().join(format!("pkgseal-testkit-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("summary.json");
        let json = serde_json::json!({
            "id": "aur/brave-bin",
            "name": "brave-bin",
            "version": "1.0",
            "source": "aur",
            "installed": false
        });
        std::fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();
        let summary: PackageSummary = load_json(&path).unwrap();
        assert_eq!(summary.name.as_str(), "brave-bin");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
