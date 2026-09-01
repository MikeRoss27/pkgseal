use pkgseal_domain::PackageSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHealthDto {
    pub app_name: String,
    pub app_version: String,
    pub engine_sources: Vec<String>,
}

impl From<&[PackageSource]> for AppHealthDto {
    fn from(sources: &[PackageSource]) -> Self {
        AppHealthDto {
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            engine_sources: sources.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAvailabilityDto {
    pub source: String,
    pub available: bool,
}
