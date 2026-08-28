use pkgseal_source::error::SourceResult;
use std::collections::HashMap;

pub fn parse_pkgbuild(content: &str) -> SourceResult<ParsedPkgbuild> {
    let mut pkg = ParsedPkgbuild::default();
    let mut current_array: Option<String> = None;
    let mut array_buffer = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(array_name) = &current_array {
            if line.ends_with(')') {
                let parts: Vec<&str> = line.trim_end_matches(')').split_whitespace().collect();
                array_buffer.extend(parts.iter().map(|s| s.to_string()));
                pkg.arrays.insert(array_name.clone(), array_buffer.clone());
                current_array = None;
                array_buffer.clear();
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                array_buffer.extend(parts.iter().map(|s| s.to_string()));
            }
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            match key {
                "pkgname" => pkg.pkgname = value.to_string(),
                "pkgver" => pkg.pkgver = value.to_string(),
                "pkgrel" => pkg.pkgrel = value.to_string(),
                "pkgdesc" => pkg.pkgdesc = Some(value.to_string()),
                "url" => pkg.url = Some(value.to_string()),
                "license" => pkg.license = Some(value.to_string()),
                "arch" => pkg.arch = Some(value.to_string()),
                "maintainer" => pkg.maintainer = Some(value.to_string()),
                _ if key.ends_with("depends")
                    || key.ends_with("provides")
                    || key.ends_with("conflicts")
                    || key.ends_with("replaces")
                    || key == "groups" =>
                {
                    if value.starts_with('(') {
                        current_array = Some(key.to_string());
                        let parts: Vec<&str> =
                            value.trim_start_matches('(').split_whitespace().collect();
                        array_buffer.extend(parts.iter().map(|s| s.to_string()));
                        if value.ends_with(')') {
                            pkg.arrays.insert(key.to_string(), array_buffer.clone());
                            current_array = None;
                            array_buffer.clear();
                        }
                    } else {
                        pkg.arrays.insert(key.to_string(), vec![value.to_string()]);
                    }
                }
                _ => {
                    pkg.other.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    Ok(pkg)
}

#[derive(Debug, Default, Clone)]
pub struct ParsedPkgbuild {
    pub pkgname: String,
    pub pkgver: String,
    pub pkgrel: String,
    pub pkgdesc: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub arch: Option<String>,
    pub maintainer: Option<String>,
    pub arrays: HashMap<String, Vec<String>>,
    pub other: HashMap<String, String>,
}

impl ParsedPkgbuild {
    pub fn depends(&self) -> Vec<String> {
        self.arrays.get("depends").cloned().unwrap_or_default()
    }

    pub fn makedepends(&self) -> Vec<String> {
        self.arrays.get("makedepends").cloned().unwrap_or_default()
    }

    pub fn checkdepends(&self) -> Vec<String> {
        self.arrays.get("checkdepends").cloned().unwrap_or_default()
    }

    pub fn optdepends(&self) -> Vec<String> {
        self.arrays.get("optdepends").cloned().unwrap_or_default()
    }

    pub fn provides(&self) -> Vec<String> {
        self.arrays.get("provides").cloned().unwrap_or_default()
    }

    pub fn conflicts(&self) -> Vec<String> {
        self.arrays.get("conflicts").cloned().unwrap_or_default()
    }

    pub fn replaces(&self) -> Vec<String> {
        self.arrays.get("replaces").cloned().unwrap_or_default()
    }

    pub fn groups(&self) -> Vec<String> {
        self.arrays.get("groups").cloned().unwrap_or_default()
    }
}

pub fn find_findings(content: &str) -> Vec<String> {
    let mut findings = Vec::new();
    let lower = content.to_lowercase();

    if lower.contains("curl | sh") || lower.contains("wget | sh") {
        findings.push("pipe_to_shell".to_string());
    }
    if lower.contains("eval ") {
        findings.push("eval_usage".to_string());
    }
    if lower.contains("sudo ") {
        findings.push("sudo_usage".to_string());
    }
    if lower.contains("chmod +s") || lower.contains("chmod 4755") {
        findings.push("setuid_binary".to_string());
    }
    if lower.contains("chown root") {
        findings.push("chown_root".to_string());
    }
    if lower.contains("base64 -d") || lower.contains("base64 --decode") {
        findings.push("base64_decode".to_string());
    }
    if content.contains("install=") {
        findings.push("install_script".to_string());
    }
    if lower.contains("source=(") && (lower.contains("http://") || lower.contains("https://")) {
        findings.push("network_fetch_in_build".to_string());
    }

    findings
}
