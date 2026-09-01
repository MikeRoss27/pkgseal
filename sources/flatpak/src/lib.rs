pub mod adapter;
pub mod parser;

pub use adapter::FlatpakSource;

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_source::{dto::SearchQuery, traits::PackageSourceAdapter};

    #[tokio::test]
    #[ignore = "requires flatpak"]
    async fn test_search() {
        let source = FlatpakSource::new();
        let query = SearchQuery::new("org.mozilla.firefox");
        let results = source.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires flatpak"]
    async fn test_installed() {
        let source = FlatpakSource::new();
        let _installed = source.installed().await.unwrap();
    }
}
