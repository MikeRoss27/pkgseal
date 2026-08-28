pub mod adapter;

pub use adapter::ArchSource;

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;
    use pkgseal_source::{dto::SearchQuery, traits::PackageSourceAdapter};

    #[tokio::test]
    #[ignore = "requires pacman"]
    async fn test_search() {
        let source = ArchSource::new();
        let query = SearchQuery::new("linux");
        let results = source.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires pacman"]
    async fn test_details() {
        let source = ArchSource::new();
        let name = PackageName::new("linux").unwrap();
        let details = source.details(&name).await.unwrap();
        assert_eq!(details.summary.name.as_str(), "linux");
    }

    #[tokio::test]
    #[ignore = "requires pacman"]
    async fn test_installed() {
        let source = ArchSource::new();
        let installed = source.installed().await.unwrap();
        assert!(!installed.is_empty());
    }
}
