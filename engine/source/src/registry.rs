use crate::dto::SearchQuery;
use crate::error::SourceResult;
use crate::traits::PackageSourceAdapter;
use pkgseal_domain::PackageSource;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SourceRegistry {
    sources: HashMap<PackageSource, Arc<dyn PackageSourceAdapter>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn PackageSourceAdapter>) {
        self.sources.insert(adapter.source(), adapter);
    }

    pub fn get(&self, source: PackageSource) -> Option<&Arc<dyn PackageSourceAdapter>> {
        self.sources.get(&source)
    }

    pub fn sources(&self) -> Vec<Arc<dyn PackageSourceAdapter>> {
        self.sources.values().cloned().collect()
    }

    pub async fn search_all(
        &self,
        query: &SearchQuery,
    ) -> SourceResult<Vec<crate::dto::PackageSummary>> {
        let mut set = tokio::task::JoinSet::new();
        for source in self.sources() {
            let query = query.clone();
            set.spawn(async move {
                if !source.is_available().await {
                    return Vec::new();
                }
                match source.search(&query).await {
                    Ok(pkgs) => pkgs,
                    Err(e) => {
                        tracing::warn!("Search failed for {:?}: {}", source.source(), e);
                        Vec::new()
                    }
                }
            });
        }

        let mut results = Vec::new();
        while let Some(pkgs) = set.join_next().await {
            results.extend(pkgs.unwrap_or_default());
        }
        Ok(results)
    }

    pub async fn details_all(
        &self,
        name: &pkgseal_domain::PackageName,
    ) -> SourceResult<Vec<crate::dto::PackageDetails>> {
        let mut set = tokio::task::JoinSet::new();
        for source in self.sources() {
            let name = name.clone();
            set.spawn(async move {
                if !source.is_available().await {
                    return None;
                }
                match source.details(&name).await {
                    Ok(details) => Some(details),
                    Err(crate::error::SourceError::NotFound(_)) => None,
                    Err(e) => {
                        tracing::warn!("Details failed for {:?}: {}", source.source(), e);
                        None
                    }
                }
            });
        }

        let mut results = Vec::new();
        while let Some(details) = set.join_next().await {
            if let Some(details) = details.unwrap_or_default() {
                results.push(details);
            }
        }
        if results.is_empty() {
            Err(crate::error::SourceError::not_found(format!(
                "Package not found in any source: {}",
                name
            )))
        } else {
            Ok(results)
        }
    }

    pub async fn installed_all(&self) -> SourceResult<Vec<crate::dto::InstalledPackage>> {
        let mut set = tokio::task::JoinSet::new();
        for source in self.sources() {
            set.spawn(async move {
                if !source.is_available().await {
                    return Vec::new();
                }
                match source.installed().await {
                    Ok(pkgs) => pkgs,
                    Err(e) => {
                        tracing::warn!("Installed failed for {:?}: {}", source.source(), e);
                        Vec::new()
                    }
                }
            });
        }

        let mut results = Vec::new();
        while let Some(pkgs) = set.join_next().await {
            results.extend(pkgs.unwrap_or_default());
        }
        Ok(results)
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
