use pkgseal_domain::PackageName;
use pkgseal_source::dto::{InstalledPackage, PackageSummary};
use pkgseal_source::error::SourceResult;

#[derive(Debug, Default, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub branch: String,
    pub origin: String,
    pub description: Option<String>,
    pub arch: String,
    pub installed: bool,
    pub installed_size: u64,
    pub download_size: u64,
    pub developer_name: Option<String>,
    pub license: Option<String>,
    pub url: Option<String>,
    pub runtime: String,
    pub runtime_version: String,
    pub sdk: String,
    pub commit: String,
    pub ref_: String,
    pub verification: Option<String>,
    pub permissions: Vec<String>,
}

pub fn parse_flatpak_search(output: &str) -> SourceResult<Vec<PackageSummary>> {
    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Name") || line.starts_with("---") {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 5 {
            let app_id = parts[1].trim();
            let version = parts[2].trim();
            let description = parts[3].trim();
            let origin = parts[4].trim();

            let sanitized = app_id.replace('.', "-").to_lowercase();
            let pkg_name = match PackageName::new(&sanitized) {
                Ok(n) => n,
                Err(_) => {
                    // Sanitize fallback failed (rare: empty app_id or boundary chars).
                    // Skip the package to avoid collisions on a generic "invalid" name.
                    tracing::warn!(
                        app_id = %app_id,
                        sanitized = %sanitized,
                        "flatpak search: sanitized app_id is not a valid PackageName, skipping"
                    );
                    continue;
                }
            };
            packages.push(PackageSummary {
                id: format!("flatpak/{}", app_id),
                name: pkg_name,
                version: version.to_string(),
                description: if description.is_empty() {
                    None
                } else {
                    Some(description.to_string())
                },
                source: pkgseal_domain::PackageSource::Flatpak,
                repository: Some(origin.to_string()),
                installed: false,
                download_size: None,
                installed_size: None,
            });
        }
    }

    Ok(packages)
}

pub fn parse_flatpak_info(output: &str) -> SourceResult<FlatpakInfo> {
    let mut info = FlatpakInfo::default();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => info.name = value.to_string(),
                "ID" | "Application ID" => info.application_id = value.to_string(),
                "Version" => info.version = value.to_string(),
                "Branch" => info.branch = value.to_string(),
                "Origin" => info.origin = value.to_string(),
                "Description" => {
                    info.description = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Arch" => info.arch = value.to_string(),
                "Installed" => info.installed = value == "yes",
                "Installed size" => {
                    info.installed_size = parse_size_kib(value).unwrap_or(0);
                }
                "Download size" => {
                    info.download_size = parse_size_kib(value).unwrap_or(0);
                }
                "Developer" => {
                    info.developer_name = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "License" => {
                    info.license = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "URL" | "Homepage" => {
                    info.url = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Runtime" => info.runtime = value.to_string(),
                "Runtime version" => info.runtime_version = value.to_string(),
                "Sdk" => info.sdk = value.to_string(),
                "Commit" => info.commit = value.to_string(),
                "Ref" => info.ref_ = value.to_string(),
                "Verification" => {
                    info.verification = if value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    }
                }
                "Permissions" => info.permissions.push(value.to_string()),
                _ => {}
            }
        }
    }

    if info.application_id.is_empty() {
        return Err(pkgseal_source::error::SourceError::parse(
            "No application ID found".to_string(),
        ));
    }

    Ok(info)
}

pub fn parse_flatpak_list(output: &str) -> SourceResult<Vec<InstalledPackage>> {
    let mut packages = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Application ID") || line.starts_with("---") {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let app_id = parts[0].trim();
            let version = parts[1].trim();
            let origin = parts[2].trim();
            let _installation = parts[3].trim();

            if let Ok(pkg_name) = PackageName::new(app_id.replace('.', "-")) {
                packages.push(InstalledPackage {
                    name: pkg_name,
                    version: version.to_string(),
                    source: pkgseal_domain::PackageSource::Flatpak,
                    repository: Some(origin.to_string()),
                    install_date: None,
                    install_reason: Some("user".to_string()),
                    size: None,
                });
            }
        }
    }

    Ok(packages)
}

fn parse_size_kib(s: &str) -> Result<u64, &'static str> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix(" KB").or_else(|| s.strip_suffix(" kB")) {
        let num: f64 = stripped.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix(" MB") {
        let num: f64 = num_str.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0 * 1024.0) as u64)
    } else if let Some(num_str) = s.strip_suffix(" GB") {
        let num: f64 = num_str.trim().parse().map_err(|_| "parse error")?;
        Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
    } else {
        Err("unknown unit")
    }
}
