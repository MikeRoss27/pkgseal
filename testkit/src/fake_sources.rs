//! In-memory fake [`PackageSourceAdapter`] for tests.
//!
//! No IO, no network, no filesystem beyond the optional fixture directory.
//! Suitable for `#[tokio::test]` and for exercising `SourceRegistry`,
//! `group_candidates` and `engine/policy`.

use pkgseal_domain::{PackageName, PackageSource};
use pkgseal_source::dto::{InstalledPackage, PackageDetails, PackageSummary, SearchQuery};
use pkgseal_source::error::{SourceError, SourceResult};
use pkgseal_source::traits::PackageSourceAdapter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
struct Inner {
    summaries: Vec<PackageSummary>,
    details: HashMap<String, PackageDetails>,
    installed: Vec<InstalledPackage>,
    available: bool,
}

/// In-memory fake source.
///
/// Build it with the fluent helpers and then use it directly or register it
/// in a [`pkgseal_source::registry::SourceRegistry`].
#[derive(Debug, Clone)]
pub struct FakeSource {
    source: PackageSource,
    inner: Arc<RwLock<Inner>>,
}

impl FakeSource {
    pub fn new(source: PackageSource) -> Self {
        Self {
            source,
            inner: Arc::new(RwLock::new(Inner {
                summaries: Vec::new(),
                details: HashMap::new(),
                installed: Vec::new(),
                available: true,
            })),
        }
    }

    pub fn arch() -> Self {
        Self::new(PackageSource::ArchOfficial)
    }

    pub fn aur() -> Self {
        Self::new(PackageSource::Aur)
    }

    pub fn flatpak() -> Self {
        Self::new(PackageSource::Flatpak)
    }

    /// Override availability (default `true`).
    pub fn with_available(self, available: bool) -> Self {
        if let Ok(mut g) = self.inner.write() {
            g.available = available;
        }
        self
    }

    pub fn set_available(&self, available: bool) {
        if let Ok(mut g) = self.inner.write() {
            g.available = available;
        }
    }

    /// Replace summaries wholesale.
    pub fn with_summaries(self, summaries: Vec<PackageSummary>) -> Self {
        if let Ok(mut g) = self.inner.write() {
            g.summaries = summaries;
        }
        self
    }

    /// Replace details wholesale. Details are keyed by both `summary.id` and
    /// `summary.name` for convenient lookup in [`PackageSourceAdapter::details`].
    pub fn with_details(self, details: Vec<PackageDetails>) -> Self {
        if let Ok(mut g) = self.inner.write() {
            g.details.clear();
            for d in details {
                g.details.insert(d.summary.id.clone(), d.clone());
                g.details
                    .insert(d.summary.name.as_str().to_string(), d.clone());
                // Also key by lowercased name for case-insensitive lookup.
                g.details.insert(d.summary.name.as_str().to_lowercase(), d);
            }
        }
        self
    }

    pub fn with_installed(self, installed: Vec<InstalledPackage>) -> Self {
        if let Ok(mut g) = self.inner.write() {
            g.installed = installed;
        }
        self
    }

    /// Insert or overwrite a single summary.
    pub fn insert_summary(&self, summary: PackageSummary) {
        if let Ok(mut g) = self.inner.write() {
            // Replace if id already present, else push.
            if let Some(pos) = g.summaries.iter().position(|s| s.id == summary.id) {
                g.summaries[pos] = summary;
            } else {
                g.summaries.push(summary);
            }
        }
    }

    /// Insert or overwrite a single detail (also ensures a summary exists).
    pub fn insert_details(&self, details: PackageDetails) {
        if let Ok(mut g) = self.inner.write() {
            let summary = details.summary.clone();
            g.details
                .insert(details.summary.id.clone(), details.clone());
            g.details
                .insert(details.summary.name.as_str().to_string(), details.clone());
            g.details.insert(
                details.summary.name.as_str().to_lowercase(),
                details.clone(),
            );
            // Ensure summaries contains this package
            if !g.summaries.iter().any(|s| s.id == summary.id) {
                g.summaries.push(summary);
            }
        }
    }

    pub fn insert_installed(&self, pkg: InstalledPackage) {
        if let Ok(mut g) = self.inner.write() {
            g.installed.push(pkg);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().map(|g| g.summaries.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clone the summaries for inspection in tests.
    pub fn summaries_snapshot(&self) -> Vec<PackageSummary> {
        self.inner
            .read()
            .map(|g| g.summaries.clone())
            .unwrap_or_default()
    }
}

#[async_trait::async_trait]
impl PackageSourceAdapter for FakeSource {
    fn source(&self) -> PackageSource {
        self.source
    }

    async fn search(&self, query: &SearchQuery) -> SourceResult<Vec<PackageSummary>> {
        let q = query.query.to_lowercase();
        let limit = query.limit.unwrap_or(50);

        let guard = self
            .inner
            .read()
            .map_err(|_| SourceError::internal("FakeSource lock poisoned"))?;

        if !guard.available {
            return Err(SourceError::unavailable(format!(
                "fake source {} unavailable",
                self.source
            )));
        }

        if q.trim_start().starts_with('-') {
            return Err(SourceError::validation(
                "search query must not start with '-'",
            ));
        }

        if q.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<PackageSummary> = guard
            .summaries
            .iter()
            .filter(|s| {
                let name = s.name.as_str().to_lowercase();
                let desc = s.description.as_deref().unwrap_or("").to_lowercase();
                let id = s.id.to_lowercase();
                name.contains(&q) || desc.contains(&q) || id.contains(&q)
            })
            .take(limit)
            .cloned()
            .collect();

        // Deterministic ordering for tests.
        results.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
        Ok(results)
    }

    async fn details(&self, name: &PackageName) -> SourceResult<PackageDetails> {
        let guard = self
            .inner
            .read()
            .map_err(|_| SourceError::internal("FakeSource lock poisoned"))?;

        if !guard.available {
            return Err(SourceError::unavailable(format!(
                "fake source {} unavailable",
                self.source
            )));
        }

        let key = name.as_str().to_string();
        let lower = key.to_lowercase();

        guard
            .details
            .get(&key)
            .or_else(|| guard.details.get(&lower))
            .cloned()
            .ok_or_else(|| SourceError::not_found(format!("package not found: {key}")))
    }

    async fn installed(&self) -> SourceResult<Vec<InstalledPackage>> {
        let guard = self
            .inner
            .read()
            .map_err(|_| SourceError::internal("FakeSource lock poisoned"))?;
        if !guard.available {
            return Ok(Vec::new());
        }
        Ok(guard.installed.clone())
    }

    async fn is_available(&self) -> bool {
        self.inner.read().map(|g| g.available).unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Helpers for quickly building a small, realistic FakeSource from fixtures
// ---------------------------------------------------------------------------

/// Build a `FakeSource` seeded with the `brave` fixture for the given source,
/// if the fixture exists; otherwise return an empty but available FakeSource.
///
/// This is offline — it reads from `fixtures/` only.
pub fn fake_source_seeded(source: PackageSource) -> FakeSource {
    let fake = FakeSource::new(source);
    // Best-effort: if fixtures are present, seed from them; otherwise stay empty.
    if let Ok(details) = crate::fixtures::load_details(source, "brave") {
        fake.insert_details(details);
    }
    fake
}

/// Build a `FakeSource` pre-populated with the provided summaries/details pair.
/// Convenience for tests that already have builders.
pub fn fake_source_from_details(source: PackageSource, details: Vec<PackageDetails>) -> FakeSource {
    FakeSource::new(source).with_details(details)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builders::{package_details, package_summary};
    use pkgseal_domain::PackageName;

    #[tokio::test]
    async fn search_filters_by_name() {
        let fake = FakeSource::aur().with_summaries(vec![
            package_summary()
                .aur()
                .name("brave-bin")
                .with_description("Brave web browser")
                .build(),
            package_summary()
                .aur()
                .name("firefox")
                .with_description("Mozilla Firefox web browser")
                .build(),
        ]);

        let results = fake
            .search(&SearchQuery::new("brave"))
            .await
            .expect("search succeeds");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name.as_str(), "brave-bin");
    }

    #[tokio::test]
    async fn details_resolves_by_name() {
        let details = package_details().aur().name("brave-bin").build();
        let fake = FakeSource::aur().with_details(vec![details.clone()]);

        let got = fake
            .details(&PackageName::new("brave-bin").unwrap())
            .await
            .unwrap();
        assert_eq!(got.summary.name.as_str(), "brave-bin");
    }

    #[tokio::test]
    async fn unavailable_source_returns_error() {
        let fake = FakeSource::arch().with_available(false);
        let res = fake.search(&SearchQuery::new("brave")).await;
        assert!(res.is_err());
        assert!(!fake.is_available().await);
    }

    #[tokio::test]
    async fn search_rejects_dash_prefix_like_real_adapter() {
        let fake = FakeSource::aur();
        let err = fake
            .search(&SearchQuery::new("-evil"))
            .await
            .expect_err("dash prefix should be rejected");
        assert!(err.to_string().contains("must not start with"));
    }

    #[tokio::test]
    async fn insert_details_also_adds_summary() {
        let fake = FakeSource::aur();
        assert!(fake.is_empty());
        let details = package_details().aur().name("brave-bin").build();
        fake.insert_details(details);
        assert_eq!(fake.len(), 1);
        let summaries = fake.summaries_snapshot();
        assert_eq!(summaries[0].name.as_str(), "brave-bin");
    }
}
