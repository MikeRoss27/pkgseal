use crate::parser::{FlatpakInfo, parse_flatpak_info, parse_flatpak_list, parse_flatpak_search};
use pkgseal_domain::PackageName;
use pkgseal_source::dto::{InstalledPackage, PackageDetails, PackageSummary};
use pkgseal_source::error::SourceResult;
use pkgseal_source::traits::PackageSourceAdapter;
use serde_json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Clone, Default)]
pub struct FlatpakSource;

impl FlatpakSource {
    pub fn new() -> Self {
        Self
    }

    async fn run_flatpak(args: &[&str]) -> SourceResult<String> {
        let output = timeout(
            Duration::from_secs(10),
            Command::new("flatpak").args(args).output(),
        )
        .await
        .map_err(|_| pkgseal_source::error::SourceError::unavailable("flatpak timeout"))?
        .map_err(|e| {
            pkgseal_source::error::SourceError::unavailable(format!("flatpak failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(pkgseal_source::error::SourceError::unavailable(format!(
                "flatpak error: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait::async_trait]
impl PackageSourceAdapter for FlatpakSource {
    fn source(&self) -> pkgseal_domain::PackageSource {
        pkgseal_domain::PackageSource::Flatpak
    }

    async fn search(
        &self,
        query: &pkgseal_source::dto::SearchQuery,
    ) -> SourceResult<Vec<PackageSummary>> {
        if query.query.trim_start().starts_with('-') {
            return Err(pkgseal_source::error::SourceError::validation(
                "search query must not start with '-'",
            ));
        }
        let output = Self::run_flatpak(&[
            "search",
            "--columns=name,application,version,description,origin",
            &query.query,
        ])
        .await?;
        parse_flatpak_search(&output)
    }

    async fn details(&self, name: &PackageName) -> SourceResult<PackageDetails> {
        let output = Self::run_flatpak(&["info", name.as_str()]).await?;
        let info = parse_flatpak_info(&output)?;
        Ok(self.info_to_details(info))
    }

    async fn installed(&self) -> SourceResult<Vec<InstalledPackage>> {
        let output = Self::run_flatpak(&[
            "list",
            "--app",
            "--columns=application,version,origin,installation",
        ])
        .await?;
        parse_flatpak_list(&output)
    }

    async fn is_available(&self) -> bool {
        Command::new("flatpak")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl FlatpakSource {
    fn info_to_details(&self, info: FlatpakInfo) -> PackageDetails {
        let mut raw_metadata = HashMap::new();

        raw_metadata.insert(
            "runtime".to_string(),
            serde_json::Value::String(info.runtime.clone()),
        );
        raw_metadata.insert(
            "runtime_version".to_string(),
            serde_json::Value::String(info.runtime_version.clone()),
        );
        raw_metadata.insert(
            "sdk".to_string(),
            serde_json::Value::String(info.sdk.clone()),
        );
        raw_metadata.insert(
            "origin".to_string(),
            serde_json::Value::String(info.origin.clone()),
        );
        raw_metadata.insert(
            "ref".to_string(),
            serde_json::Value::String(info.ref_.clone()),
        );
        raw_metadata.insert(
            "commit".to_string(),
            serde_json::Value::String(info.commit.clone()),
        );
        raw_metadata.insert(
            "installed_size".to_string(),
            serde_json::Value::Number(info.installed_size.into()),
        );
        raw_metadata.insert(
            "download_size".to_string(),
            serde_json::Value::Number(info.download_size.into()),
        );

        if let Some(ref verification) = info.verification {
            raw_metadata.insert(
                "verification".to_string(),
                serde_json::Value::String(verification.clone()),
            );
        }

        if !info.permissions.is_empty() {
            raw_metadata.insert(
                "permissions".to_string(),
                serde_json::Value::Array(
                    info.permissions
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        // `info.name` is the human-readable name (e.g. "Brave Browser") and would
        // always fail PackageName validation; use the stable application_id instead.
        let sanitized = info.application_id.replace('.', "-").to_lowercase();
        let pkg_name = PackageName::new(&sanitized).unwrap_or_else(|_| {
            let fallback = format!(
                "flatpak-invalid-{}",
                info.application_id.replace('.', "-").to_lowercase()
            );
            // fallback is controlled and should be valid; last resort to bare literal.
            PackageName::new(&fallback)
                .unwrap_or_else(|_| PackageName::new("flatpak-invalid").unwrap())
        });
        PackageDetails {
            summary: PackageSummary {
                id: format!("flatpak/{}", info.application_id),
                name: pkg_name,
                version: info.version,
                description: info.description,
                source: pkgseal_domain::PackageSource::Flatpak,
                repository: Some(info.origin),
                installed: info.installed,
                download_size: Some(info.download_size),
                installed_size: Some(info.installed_size),
            },
            architecture: Some(info.arch.clone()),
            maintainer: info.developer_name,
            url: info.url,
            license: info.license,
            dependencies: vec![],
            optional_dependencies: vec![],
            provides: vec![],
            conflicts: vec![],
            replaces: vec![],
            groups: vec![],
            build_date: None,
            install_date: None,
            validation: info.verification.clone(),
            raw_metadata,
        }
    }
}
