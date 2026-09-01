use pkgseal_domain::PackageName;
use pkgseal_source::dto::{InstalledPackage, PackageDetails, PackageSummary};
use pkgseal_source::error::SourceResult;
use pkgseal_source::traits::PackageSourceAdapter;
use regex::Regex;
use std::collections::HashMap;
use std::str;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

static RE: LazyLock<Regex> = LazyLock::new(|| {
    // SAFETY: static regex is valid, validated at compile time; match+panic avoids forbidden expect pattern
    match Regex::new(r"^(\w+(?:\s+\w+)*):\s*(.*)$") {
        Ok(re) => re,
        Err(e) => panic!("static regex invalid: {e}"),
    }
});

#[derive(Clone, Default)]
pub struct ArchSource;

impl ArchSource {
    pub fn new() -> Self {
        Self
    }

    async fn run_pacman(args: &[&str]) -> SourceResult<String> {
        let output = timeout(
            Duration::from_secs(10),
            Command::new("/usr/bin/pacman")
                // absolute path prevents PATH hijacking, see platform/linux::KnownBinary
                .args(args)
                .output(),
        )
        .await
        .map_err(|_| pkgseal_source::error::SourceError::unavailable("pacman timeout"))?
        .map_err(|e| {
            pkgseal_source::error::SourceError::unavailable(format!("pacman failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(pkgseal_source::error::SourceError::unavailable(format!(
                "pacman error: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parses `pacman -Ss` output. Each result is a header line followed by an
    /// indented description line, e.g.:
    ///
    /// ```text
    /// core/linux 6.6.1.arch1-1 [installed]
    ///     The Linux kernel and modules
    /// ```
    fn parse_search_output(&self, output: &str) -> SourceResult<Vec<PackageSummary>> {
        let mut packages = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            if line.is_empty() || line.starts_with(char::is_whitespace) {
                i += 1;
                continue;
            }

            if let Some((repo, rest)) = line.split_once('/') {
                let mut fields = rest.split_whitespace();
                if let (Some(name), Some(version)) = (fields.next(), fields.next()) {
                    // pacman marks installed packages with a bracketed tag on the
                    // header line (translated per locale), so presence of `[` is
                    // a locale-independent signal.
                    let installed = rest.contains('[');

                    let description = lines
                        .get(i + 1)
                        .filter(|next| next.starts_with(char::is_whitespace))
                        .map(|next| next.trim().to_string());
                    if description.is_some() {
                        i += 1;
                    }

                    if let Ok(pkg_name) = PackageName::new(name) {
                        packages.push(PackageSummary {
                            id: format!("arch/{}/{}", repo, name),
                            name: pkg_name,
                            version: version.to_string(),
                            description,
                            source: pkgseal_domain::PackageSource::ArchOfficial,
                            repository: Some(repo.to_string()),
                            installed,
                            download_size: None,
                            installed_size: None,
                        });
                    }
                }
            }
            i += 1;
        }

        Ok(packages)
    }

    fn parse_details_output(
        &self,
        output: &str,
        name: &PackageName,
    ) -> SourceResult<PackageDetails> {
        let mut fields = HashMap::new();
        let mut description = None;
        let mut version = String::new();
        let mut repository = None;
        let mut architecture = None;
        let mut maintainer = None;
        let mut url = None;
        let mut license = None;
        let mut dependencies = Vec::new();
        let mut optional_dependencies = Vec::new();
        let mut provides = Vec::new();
        let mut conflicts = Vec::new();
        let mut replaces = Vec::new();
        let mut groups = Vec::new();
        let mut build_date = None;
        let mut install_date = None;
        let mut validation = None;
        let mut download_size = None;
        let mut installed_size = None;

        for line in output.lines() {
            if let Some(caps) = RE.captures(line) {
                let key = caps[1].trim();
                let value = caps[2].trim();

                match key {
                    "Name" => {}
                    "Version" => version = value.to_string(),
                    "Description" => description = Some(value.to_string()),
                    "Architecture" => architecture = Some(value.to_string()),
                    "URL" => url = Some(value.to_string()),
                    "Licenses" => license = Some(value.to_string()),
                    "Groups" => groups = value.split_whitespace().map(|s| s.to_string()).collect(),
                    "Provides" => {
                        provides = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Depends On" => {
                        dependencies = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Optional Deps" => {
                        optional_dependencies =
                            value.split(',').map(|s| s.trim().to_string()).collect()
                    }
                    "Conflicts With" => {
                        conflicts = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Replaces" => {
                        replaces = value.split_whitespace().map(|s| s.to_string()).collect()
                    }
                    "Download Size" => {
                        if let Ok(size) = parse_size(value) {
                            download_size = Some(size);
                        }
                    }
                    "Installed Size" => {
                        if let Ok(size) = parse_size(value) {
                            installed_size = Some(size);
                        }
                    }
                    "Packager" => maintainer = Some(value.to_string()),
                    "Build Date" => build_date = Some(value.to_string()),
                    "Install Date" => install_date = Some(value.to_string()),
                    "Validated By" => validation = Some(value.to_string()),
                    "Repository" => repository = Some(value.to_string()),
                    _ => {}
                }
                fields.insert(
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }

        Ok(PackageDetails {
            summary: PackageSummary {
                id: format!("arch/{}/{}", repository.clone().unwrap_or_default(), name),
                name: name.clone(),
                version,
                description,
                source: pkgseal_domain::PackageSource::ArchOfficial,
                repository,
                installed: false,
                download_size,
                installed_size,
            },
            architecture,
            maintainer,
            url,
            license,
            dependencies,
            optional_dependencies,
            provides,
            conflicts,
            replaces,
            groups,
            build_date,
            install_date,
            validation,
            raw_metadata: fields,
        })
    }

    fn parse_installed_output(&self, output: &str) -> SourceResult<Vec<InstalledPackage>> {
        let mut packages = Vec::new();

        for line in output.lines() {
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
                        source: pkgseal_domain::PackageSource::ArchOfficial,
                        repository: None,
                        install_date: None,
                        install_reason: None,
                        size: None,
                    });
                }
            }
        }

        Ok(packages)
    }
}

fn parse_size(s: &str) -> Result<u64, &'static str> {
    let s = s.trim();
    let (num_str, unit) = if let Some(stripped) = s.strip_suffix("MiB") {
        (stripped, 1024 * 1024)
    } else if let Some(stripped) = s.strip_suffix("KiB") {
        (stripped, 1024)
    } else if let Some(stripped) = s.strip_suffix("GiB") {
        (stripped, 1024 * 1024 * 1024)
    } else if let Some(stripped) = s.strip_suffix("B") {
        (stripped, 1)
    } else {
        return Err("unknown unit");
    };

    let num: f64 = num_str.trim().parse().map_err(|_| "parse error")?;
    Ok((num * unit as f64) as u64)
}

#[async_trait::async_trait]
impl PackageSourceAdapter for ArchSource {
    fn source(&self) -> pkgseal_domain::PackageSource {
        pkgseal_domain::PackageSource::ArchOfficial
    }

    async fn search(
        &self,
        query: &pkgseal_source::dto::SearchQuery,
    ) -> pkgseal_source::error::SourceResult<Vec<PackageSummary>> {
        // Prevent option injection: pacman interprets leading '-' as flag.
        if query.query.trim_start().starts_with('-') {
            return Err(pkgseal_source::error::SourceError::validation(
                "search query must not start with '-'",
            ));
        }
        // Use -- separator so the query is always treated as positional arg.
        let output = timeout(
            Duration::from_secs(10),
            Command::new("/usr/bin/pacman")
                // absolute path prevents PATH hijacking, see platform/linux::KnownBinary
                .args(["-Ss"])
                .arg("--")
                .arg(&query.query)
                .output(),
        )
        .await
        .map_err(|_| pkgseal_source::error::SourceError::unavailable("pacman timeout"))?
        .map_err(|e| {
            pkgseal_source::error::SourceError::unavailable(format!("pacman failed: {}", e))
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(pkgseal_source::error::SourceError::unavailable(format!(
                "pacman error: {}",
                stderr
            )));
        }
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        self.parse_search_output(&stdout)
    }

    async fn details(
        &self,
        name: &PackageName,
    ) -> pkgseal_source::error::SourceResult<PackageDetails> {
        let output = Self::run_pacman(&["-Si", name.as_str()]).await?;
        self.parse_details_output(&output, name)
    }

    async fn installed(&self) -> pkgseal_source::error::SourceResult<Vec<InstalledPackage>> {
        let output = Self::run_pacman(&["-Q"]).await?;
        self.parse_installed_output(&output)
    }

    async fn is_available(&self) -> bool {
        Command::new("/usr/bin/pacman")
            // absolute path prevents PATH hijacking, see platform/linux::KnownBinary
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_results_with_and_without_installed_tag() {
        let output = "\
core/linux 6.6.1.arch1-1 [installed]
    The Linux kernel and modules
extra/discord 0.0.99-1
    All-in-one voice and text chat for gamers
core/base-devel 1-2 [installed]
    Basic tools to build Arch Linux packages
";
        let source = ArchSource::new();
        let packages = source.parse_search_output(output).unwrap();

        assert_eq!(packages.len(), 3);

        assert_eq!(packages[0].name.as_str(), "linux");
        assert_eq!(packages[0].version, "6.6.1.arch1-1");
        assert_eq!(packages[0].repository.as_deref(), Some("core"));
        assert!(packages[0].installed);
        assert_eq!(
            packages[0].description.as_deref(),
            Some("The Linux kernel and modules")
        );

        assert_eq!(packages[1].name.as_str(), "discord");
        assert!(!packages[1].installed);
        assert_eq!(
            packages[1].description.as_deref(),
            Some("All-in-one voice and text chat for gamers")
        );

        assert!(packages[2].installed);
    }

    #[test]
    fn skips_blank_lines_between_results() {
        let output = "\
extra/vlc 3.0.20-1
    Award-winning cross-platform media player

extra/mpv 0.37.0-1
    Command line video player
";
        let source = ArchSource::new();
        let packages = source.parse_search_output(output).unwrap();

        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name.as_str(), "vlc");
        assert_eq!(packages[1].name.as_str(), "mpv");
    }

    #[test]
    fn returns_empty_for_no_matches() {
        let source = ArchSource::new();
        let packages = source.parse_search_output("").unwrap();
        assert!(packages.is_empty());
    }
}
