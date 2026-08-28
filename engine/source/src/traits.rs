use crate::{dto::*, error::SourceResult};
use async_trait::async_trait;
use pkgseal_domain::{PackageName, PackageSource};

#[async_trait]
pub trait PackageSourceAdapter: Send + Sync {
    fn source(&self) -> PackageSource;

    async fn search(&self, query: &SearchQuery) -> SourceResult<Vec<PackageSummary>>;

    async fn details(&self, name: &PackageName) -> SourceResult<PackageDetails>;

    async fn installed(&self) -> SourceResult<Vec<InstalledPackage>>;

    async fn is_available(&self) -> bool;
}

#[async_trait]
pub trait PackageSourceRegistry: Send + Sync {
    fn sources(&self) -> Vec<Box<dyn PackageSourceAdapter>>;

    fn get(&self, source: PackageSource) -> Option<&dyn PackageSourceAdapter>;

    async fn search_all(&self, query: &SearchQuery) -> SourceResult<Vec<PackageSummary>> {
        let mut results = Vec::new();
        for source in self.sources() {
            if source.is_available().await {
                match source.search(query).await {
                    Ok(mut pkgs) => results.append(&mut pkgs),
                    Err(e) => {
                        tracing::warn!("Search failed for {:?}: {}", source.source(), e);
                    }
                }
            }
        }
        Ok(results)
    }

    async fn details_all(&self, name: &PackageName) -> SourceResult<Vec<PackageDetails>> {
        let mut results = Vec::new();
        for source in self.sources() {
            if source.is_available().await {
                match source.details(name).await {
                    Ok(details) => results.push(details),
                    Err(crate::error::SourceError::NotFound(_)) => {}
                    Err(e) => {
                        tracing::warn!("Details failed for {:?}: {}", source.source(), e);
                    }
                }
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
}
