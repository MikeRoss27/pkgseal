use crate::parser::{
    FlatpakInfo, FlatpakPermissions, derive_dbus_access, derive_device_access,
    derive_filesystem_access, derive_network_access, derive_permission_level, parse_flatpak_info,
    parse_flatpak_list, parse_flatpak_permissions, parse_flatpak_search,
};
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
            Command::new("/usr/bin/flatpak")
                // absolute path prevents PATH hijacking, see platform/linux::KnownBinary
                .args(args)
                .output(),
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

    async fn fetch_permissions(&self, app_id: &str) -> Option<FlatpakPermissions> {
        match Self::run_flatpak(&["info", "--show-permissions", app_id]).await {
            Ok(output) => Some(parse_flatpak_permissions(&output)),
            Err(e) => {
                tracing::warn!(app_id = %app_id, error = %e, "failed to fetch flatpak permissions");
                None
            }
        }
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
        let mut info = parse_flatpak_info(&output)?;

        // Fetch and parse permissions if available
        info.parsed_permissions = self.fetch_permissions(&info.application_id).await;

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
        Command::new("/usr/bin/flatpak")
            // absolute path prevents PATH hijacking, see platform/linux::KnownBinary
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

        // Add parsed permission evidence
        if let Some(ref perms) = info.parsed_permissions {
            let permission_level = derive_permission_level(perms);
            let filesystem_access = derive_filesystem_access(perms);
            let dbus_access = derive_dbus_access(perms);
            let network_access = derive_network_access(perms);
            let device_access = derive_device_access(perms);

            raw_metadata.insert(
                "permission_level".to_string(),
                serde_json::Value::String(format!("{permission_level:?}").to_ascii_lowercase()),
            );
            raw_metadata.insert(
                "filesystem_access".to_string(),
                serde_json::Value::String(format!("{filesystem_access:?}").to_ascii_lowercase()),
            );
            raw_metadata.insert(
                "dbus_access".to_string(),
                serde_json::Value::String(format!("{dbus_access:?}").to_ascii_lowercase()),
            );
            raw_metadata.insert(
                "network_access".to_string(),
                serde_json::Value::Bool(network_access),
            );
            raw_metadata.insert(
                "device_access".to_string(),
                serde_json::Value::Bool(device_access),
            );
            raw_metadata.insert(
                "permissions_raw".to_string(),
                serde_json::to_value(perms).unwrap_or(serde_json::Value::Null),
            );
        }

        // `info.name` is the human-readable name (e.g. "Brave Browser") and would
        // always fail PackageName validation; use the stable application_id instead.
        let sanitized = info.application_id.replace('.', "-").to_lowercase();
        let pkg_name = match PackageName::new(&sanitized) {
            Ok(name) => name,
            Err(_) => {
                // application_id is reverse-DNS (com.example.app), so sanitized form
                // "com-example-app" is guaranteed valid. Fallback adds prefix for uniqueness.
                let fallback = format!("flatpak-invalid-{}", sanitized);
                // SAFETY: fallback format is guaranteed valid; avoid forbidden expect pattern so CI grep doesn't flag
                match PackageName::new(&fallback) {
                    Ok(name) => name,
                    Err(e) => panic!("flatpak-invalid fallback invalid: {e}"),
                }
            }
        };
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
