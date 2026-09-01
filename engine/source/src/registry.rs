use crate::dto::SearchQuery;
use crate::error::SourceResult;
use crate::traits::PackageSourceAdapter;
use pkgseal_domain::PackageSource;
use std::collections::HashMap;
use std::sync::Arc;

// Concurrency is bounded to the number of registered sources (currently 3:
// Arch/AUR/Flatpak), so an additional Semaphore is unnecessary here.
// `is_available` is called per-operation without caching; if thundering-herd
// becomes observable under repeated `search_all` calls, add a short-TTL cache
// (e.g. 30s) for availability.  TODO: cache availability 30s if needed.
// Detail-level fan-out (N summaries, up to 50) is bounded in
// `apps/desktop/src-tauri/src/commands/resolver.rs` via `Semaphore(8)`.

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
        while let Some(join_result) = set.join_next().await {
            match join_result {
                Ok(pkgs) => results.extend(pkgs),
                Err(e) => tracing::warn!("Search task panicked or was cancelled: {}", e),
            }
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
        while let Some(join_result) = set.join_next().await {
            match join_result {
                Ok(Some(details)) => results.push(details),
                Ok(None) => {}
                Err(e) => tracing::warn!("Details task panicked or was cancelled: {}", e),
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
        while let Some(join_result) = set.join_next().await {
            match join_result {
                Ok(pkgs) => results.extend(pkgs),
                Err(e) => tracing::warn!("Installed task panicked or was cancelled: {}", e),
            }
        }
        Ok(results)
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
