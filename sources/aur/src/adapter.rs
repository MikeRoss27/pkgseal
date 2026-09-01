use crate::parser::ParsedPkgbuild;
use crate::rpc::{AurPackage, get_package_info, search_packages};
use pkgseal_domain::PackageName;
use pkgseal_source::dto::{InstalledPackage, PackageDetails, PackageSummary};
use pkgseal_source::error::SourceResult;
use pkgseal_source::traits::PackageSourceAdapter;
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Bounds how long a single AUR HTTP call may take. Without this, a stalled
/// connection (e.g. broken IPv6 routing falling back to IPv4) can hang for
/// the platform's TCP connect timeout — tens of seconds — and freeze the
/// whole search behind one slow source instead of just failing it.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone)]
pub struct AurSource {
    client: reqwest::Client,
}

impl Default for AurSource {
    fn default() -> Self {
        Self::new()
    }
}

impl AurSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .unwrap_or_default(),
        }
    }

    fn aur_package_to_summary(pkg: AurPackage) -> PackageSummary {
        let sanitized = pkg.name.to_lowercase().replace(['_', '.'], "-");
        let pkg_name = PackageName::new(&sanitized).unwrap_or_else(|_| {
            // Fallback must be unique to avoid collisions on "invalid".
            let fallback = format!("invalid-{}", pkg.id);
            // `fallback` is controlled (prefix + numeric id) and always valid.
            PackageName::new(&fallback).unwrap()
        });
        PackageSummary {
            id: format!("aur/{}", pkg.name),
            name: pkg_name,
            version: pkg.version.clone(),
            description: pkg.description.clone(),
            source: pkgseal_domain::PackageSource::Aur,
            repository: Some("aur".to_string()),
            installed: false,
            download_size: None,
            installed_size: None,
        }
    }

    fn build_details(
        &self,
        aur_pkg: &AurPackage,
        pkgbuild: Option<&ParsedPkgbuild>,
    ) -> PackageDetails {
        let mut raw_metadata = HashMap::new();
        raw_metadata.insert(
            "num_votes".to_string(),
            serde_json::Value::Number(aur_pkg.num_votes.into()),
        );
        // popularity comes from the network; NaN/Inf would panic with unwrap().
        let popularity_number = if aur_pkg.popularity.is_finite() {
            serde_json::Number::from_f64(aur_pkg.popularity)
                .unwrap_or_else(|| serde_json::Number::from(0))
        } else {
            serde_json::Number::from(0)
        };
        raw_metadata.insert(
            "popularity".to_string(),
            serde_json::Value::Number(popularity_number),
        );
        raw_metadata.insert(
            "maintainer".to_string(),
            serde_json::Value::String(aur_pkg.maintainer.clone().unwrap_or_default()),
        );
        raw_metadata.insert(
            "first_submitted".to_string(),
            serde_json::Value::Number(aur_pkg.first_submitted.into()),
        );
        raw_metadata.insert(
            "last_modified".to_string(),
            serde_json::Value::Number(aur_pkg.last_modified.into()),
        );

        if let Some(pkgbuild) = pkgbuild {
            raw_metadata.insert(
                "makedepends".to_string(),
                serde_json::Value::Array(
                    pkgbuild
                        .makedepends()
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
            raw_metadata.insert(
                "checkdepends".to_string(),
                serde_json::Value::Array(
                    pkgbuild
                        .checkdepends()
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
            raw_metadata.insert(
                "optdepends".to_string(),
                serde_json::Value::Array(
                    pkgbuild
                        .optdepends()
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
            // TODO: restore real PKGBUILD content analysis once raw PKGBUILD fetch
            // is re-enabled. `build_details` currently has no raw PKGBUILD text
            // (fetch deliberately disabled, see details() comment), so the previous
            // `find_findings(&pkgbuild.pkgname)` was a BUG — it scanned the package
            // name string, not the PKGBUILD content, yielding false negatives.
            // Return empty findings until content is available.
            raw_metadata.insert("findings".to_string(), serde_json::Value::Array(Vec::new()));
        }

        let (dependencies, provides, conflicts, replaces, groups) = if let Some(pkgbuild) = pkgbuild
        {
            (
                pkgbuild.depends(),
                pkgbuild.provides(),
                pkgbuild.conflicts(),
                pkgbuild.replaces(),
                pkgbuild.groups(),
            )
        } else {
            (
                aur_pkg.depends.clone(),
                aur_pkg.provides.clone(),
                aur_pkg.conflicts.clone(),
                aur_pkg.replaces.clone(),
                aur_pkg.groups.clone(),
            )
        };

        PackageDetails {
            summary: Self::aur_package_to_summary(aur_pkg.clone()),
            architecture: Some("any".to_string()),
            maintainer: aur_pkg.maintainer.clone(),
            url: aur_pkg.url.clone(),
            license: aur_pkg.license.first().cloned(),
            dependencies,
            optional_dependencies: aur_pkg.opt_depends.clone(),
            provides,
            conflicts,
            replaces,
            groups,
            build_date: None,
            install_date: None,
            validation: None,
            raw_metadata,
        }
    }
}

#[async_trait::async_trait]
impl PackageSourceAdapter for AurSource {
    fn source(&self) -> pkgseal_domain::PackageSource {
        pkgseal_domain::PackageSource::Aur
    }

    async fn search(
        &self,
        query: &pkgseal_source::dto::SearchQuery,
    ) -> SourceResult<Vec<PackageSummary>> {
        let results = search_packages(&self.client, &query.query)
            .await
            .map_err(|e| pkgseal_source::error::SourceError::network(e.to_string()))?;

        let limit = query.limit.unwrap_or(50);
        Ok(results
            .into_iter()
            .take(limit)
            .map(Self::aur_package_to_summary)
            .collect())
    }

    async fn details(&self, name: &PackageName) -> SourceResult<PackageDetails> {
        // The RPC `info` response already carries dependencies, provides,
        // license, etc. Fetching and parsing the PKGBUILD too is an extra
        // network round-trip per candidate against an endpoint that rate-limits
        // concurrent callers (aur.archlinux.org's cgit); it's only worth paying
        // for once PKGBUILD-derived evidence (security findings) is actually
        // surfaced in the UI, so it's deliberately not done here yet.
        let results = get_package_info(&self.client, &[name.as_str().to_string()])
            .await
            .map_err(|e| pkgseal_source::error::SourceError::network(e.to_string()))?;

        let aur_pkg = results.into_iter().next().ok_or_else(|| {
            pkgseal_source::error::SourceError::not_found(format!(
                "AUR package not found: {}",
                name
            ))
        })?;

        Ok(self.build_details(&aur_pkg, None))
    }

    async fn installed(&self) -> SourceResult<Vec<InstalledPackage>> {
        let output = timeout(
            Duration::from_secs(10),
            Command::new("pacman").args(["-Qm"]).output(),
        )
        .await
        .map_err(|_| pkgseal_source::error::SourceError::unavailable("pacman timeout"))?
        .map_err(|e| {
            pkgseal_source::error::SourceError::unavailable(format!("pacman failed: {}", e))
        })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0];
                let version = parts[1];

                if let Ok(pkg_name) = PackageName::new(name) {
                    packages.push(InstalledPackage {
                        name: pkg_name,
                        version: version.to_string(),
                        source: pkgseal_domain::PackageSource::Aur,
                        repository: Some("aur".to_string()),
                        install_date: None,
                        install_reason: Some("explicit".to_string()),
                        size: None,
                    });
                }
            }
        }

        Ok(packages)
    }

    async fn is_available(&self) -> bool {
        self.client
            .get("https://aur.archlinux.org")
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
