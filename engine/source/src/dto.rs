use pkgseal_domain::{PackageName, PackageSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: Some(50),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSummary {
    pub id: String,
    pub name: PackageName,
    pub version: String,
    pub description: Option<String>,
    pub source: PackageSource,
    pub repository: Option<String>,
    pub installed: bool,
    pub download_size: Option<u64>,
    pub installed_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetails {
    pub summary: PackageSummary,
    pub architecture: Option<String>,
    pub maintainer: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub groups: Vec<String>,
    pub build_date: Option<String>,
    pub install_date: Option<String>,
    pub validation: Option<String>,
    pub raw_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPackage {
    pub name: PackageName,
    pub version: String,
    pub source: PackageSource,
    pub repository: Option<String>,
    pub install_date: Option<String>,
    pub install_reason: Option<String>,
    pub size: Option<u64>,
}
