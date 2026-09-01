pub mod adapter;
pub mod parser;
pub mod rpc;

pub use adapter::AurSource;

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;
    use pkgseal_source::{dto::SearchQuery, traits::PackageSourceAdapter};

    #[tokio::test]
    #[ignore = "requires network"]
    async fn test_search() {
        let source = AurSource::new();
        let query = SearchQuery::new("yay");
        let results = source.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn test_details() {
        let source = AurSource::new();
        let name = PackageName::new("yay").unwrap();
        let details = source.details(&name).await.unwrap();
        assert_eq!(details.summary.name.as_str(), "yay");
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn test_installed() {
        let source = AurSource::new();
        let _installed = source.installed().await.unwrap();
        // AUR packages aren't tracked by pacman -Q, so this will be empty or use a helper
    }
}
