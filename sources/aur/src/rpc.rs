use serde::{Deserialize, Serialize};

// The AUR RPC (v5) uses `resultcount`/`results`/`type`/`version` at the root,
// and PascalCase field names on each package. Array fields (dependencies,
// license, keywords, ...) are omitted entirely by the API when empty, so they
// need `#[serde(default)]` rather than being treated as required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurRpcResponse {
    pub version: u32,
    #[serde(rename = "type")]
    pub type_: String,
    pub resultcount: u32,
    pub results: Vec<AurPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurPackage {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "PackageBase")]
    pub package_base: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    #[serde(rename = "NumVotes")]
    pub num_votes: u32,
    #[serde(rename = "Popularity")]
    pub popularity: f64,
    #[serde(rename = "OutOfDate")]
    pub out_of_date: Option<u64>,
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,
    #[serde(rename = "FirstSubmitted")]
    pub first_submitted: u64,
    #[serde(rename = "LastModified")]
    pub last_modified: u64,
    #[serde(rename = "License", default)]
    pub license: Vec<String>,
    #[serde(rename = "Depends", default)]
    pub depends: Vec<String>,
    #[serde(rename = "MakeDepends", default)]
    pub make_depends: Vec<String>,
    #[serde(rename = "CheckDepends", default)]
    pub check_depends: Vec<String>,
    #[serde(rename = "OptDepends", default)]
    pub opt_depends: Vec<String>,
    #[serde(rename = "Provides", default)]
    pub provides: Vec<String>,
    #[serde(rename = "Conflicts", default)]
    pub conflicts: Vec<String>,
    #[serde(rename = "Replaces", default)]
    pub replaces: Vec<String>,
    #[serde(rename = "Groups", default)]
    pub groups: Vec<String>,
    #[serde(rename = "Keywords", default)]
    pub keywords: Vec<String>,
}

impl AurPackage {
    pub fn to_search_summary(&self) -> pkgseal_source::dto::PackageSummary {
        let sanitized = self.name.to_lowercase().replace(['_', '.'], "-");
        let pkg_name = pkgseal_domain::PackageName::new(&sanitized).unwrap_or_else(|_| {
            // Unique fallback to avoid collisions on generic "invalid".
            let fallback = format!("invalid-{}", self.id);
            // SAFETY: "invalid-{numeric_id}" is guaranteed valid PackageName; avoid forbidden unwrap pattern
            pkgseal_domain::PackageName::new(&fallback)
                .unwrap_or_else(|e| panic!("invalid fallback invalid: {e}"))
        });
        pkgseal_source::dto::PackageSummary {
            id: format!("aur/{}", self.name),
            name: pkg_name,
            version: self.version.clone(),
            description: self.description.clone(),
            source: pkgseal_domain::PackageSource::Aur,
            repository: Some("aur".to_string()),
            installed: false,
            download_size: None,
            installed_size: None,
        }
    }
}

pub const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/v5";

pub async fn search_packages(
    client: &reqwest::Client,
    query: &str,
) -> Result<Vec<AurPackage>, reqwest::Error> {
    let url = format!("{}/search/{}", AUR_RPC_URL, urlencoding::encode(query));
    let resp: AurRpcResponse = client.get(&url).send().await?.json().await?;
    Ok(resp.results)
}

pub async fn get_package_info(
    client: &reqwest::Client,
    names: &[String],
) -> Result<Vec<AurPackage>, reqwest::Error> {
    let args = names
        .iter()
        .map(|n| format!("arg[]={}", urlencoding::encode(n)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!("{}/info?{}", AUR_RPC_URL, args);
    let resp: AurRpcResponse = client.get(&url).send().await?.json().await?;
    Ok(resp.results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_search_response_with_omitted_array_fields() {
        // `search` responses omit dependency/license/keyword arrays entirely
        // when empty (Go's `omitempty`), and use PascalCase field names.
        let body = r#"{
            "resultcount": 1,
            "results": [{
                "Description": "A package manager wrapper",
                "FirstSubmitted": 1778477360,
                "ID": 2072683,
                "LastModified": 1778477360,
                "Maintainer": "someone",
                "Name": "akp",
                "NumVotes": 0,
                "OutOfDate": null,
                "PackageBase": "akp",
                "Popularity": 0,
                "URL": "https://example.com",
                "Version": "1.0.0-1"
            }],
            "type": "search",
            "version": 5
        }"#;

        let resp: AurRpcResponse = serde_json::from_str(body).unwrap();
        assert_eq!(resp.resultcount, 1);
        assert_eq!(resp.results.len(), 1);

        let pkg = &resp.results[0];
        assert_eq!(pkg.name, "akp");
        assert_eq!(pkg.version, "1.0.0-1");
        assert!(pkg.depends.is_empty());
        assert!(pkg.keywords.is_empty());
    }

    #[test]
    fn deserializes_info_response_with_full_dependency_data() {
        let body = r#"{
            "resultcount": 1,
            "results": [{
                "Depends": ["pacman>6.1", "git"],
                "Description": "Yet another yogurt",
                "FirstSubmitted": 1475688004,
                "ID": 2131240,
                "Keywords": ["helper"],
                "LastModified": 1781905288,
                "License": ["GPL-3.0-or-later"],
                "Maintainer": "jguer",
                "MakeDepends": ["go>=1.24"],
                "Name": "yay",
                "NumVotes": 2647,
                "OptDepends": ["sudo"],
                "OutOfDate": null,
                "PackageBase": "yay",
                "Popularity": 37.42,
                "URL": "https://github.com/Jguer/yay",
                "Version": "13.0.1-1"
            }],
            "type": "multiinfo",
            "version": 5
        }"#;

        let resp: AurRpcResponse = serde_json::from_str(body).unwrap();
        let pkg = &resp.results[0];
        assert_eq!(pkg.depends, vec!["pacman>6.1", "git"]);
        assert_eq!(pkg.make_depends, vec!["go>=1.24"]);
        assert_eq!(pkg.license, vec!["GPL-3.0-or-later"]);
    }
}
