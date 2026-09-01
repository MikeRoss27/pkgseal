pub use crate::identity::MatchSignal;
use crate::normalize::*;
use pkgseal_domain::PackageSource;
use pkgseal_source::dto::{PackageDetails, PackageSummary};

pub trait SignalExtractor: Send + Sync {
    fn extract(&self, details: &PackageDetails, summary: &PackageSummary) -> Vec<MatchSignal>;
    fn source(&self) -> PackageSource;
}

pub fn extract_signals(
    extractors: &[Box<dyn SignalExtractor>],
    details: &PackageDetails,
    summary: &PackageSummary,
) -> Vec<MatchSignal> {
    let mut signals = Vec::new();
    for extractor in extractors {
        if extractor.source() == summary.source {
            signals.extend(extractor.extract(details, summary));
        }
    }
    signals
}

pub struct ArchSignalExtractor;

impl SignalExtractor for ArchSignalExtractor {
    fn source(&self) -> PackageSource {
        PackageSource::ArchOfficial
    }

    fn extract(&self, details: &PackageDetails, summary: &PackageSummary) -> Vec<MatchSignal> {
        let mut signals = Vec::new();

        // Binary name from package name
        signals.push(MatchSignal::BinaryName(summary.name.as_str().to_string()));

        // Product name from package name
        signals.push(MatchSignal::ProductName(extract_product_name_from_package(
            &summary.name,
        )));

        // Source repository
        if let Some(repo) = &summary.repository {
            signals.push(MatchSignal::SourceRepository(format!("arch/{}", repo)));
        }

        // Maintainer as publisher hint
        if let Some(maintainer) = &details.maintainer {
            signals.push(MatchSignal::Publisher(normalize_vendor_name(maintainer)));
        }

        // URL/Homepage
        if let Some(url) = &details.url {
            if let Some(domain) = extract_reverse_domain_id(url) {
                signals.push(MatchSignal::KnownAppId(domain));
            }
            signals.push(MatchSignal::Homepage(normalize_homepage(url)));
        }

        // Desktop file ID from metadata
        if let Some(s) = details
            .raw_metadata
            .get("DesktopFile")
            .and_then(|v| v.as_str())
        {
            signals.push(MatchSignal::DesktopFileId(s.to_string()));
        }

        signals
    }
}

pub struct AurSignalExtractor;

impl SignalExtractor for AurSignalExtractor {
    fn source(&self) -> PackageSource {
        PackageSource::Aur
    }

    fn extract(&self, details: &PackageDetails, summary: &PackageSummary) -> Vec<MatchSignal> {
        let mut signals = Vec::new();

        // Binary name from package name
        signals.push(MatchSignal::BinaryName(summary.name.as_str().to_string()));

        // Product name from package name
        signals.push(MatchSignal::ProductName(extract_product_name_from_package(
            &summary.name,
        )));

        // Source repository
        signals.push(MatchSignal::SourceRepository("aur".to_string()));

        // Maintainer as publisher hint
        if let Some(maintainer) = &details.maintainer {
            signals.push(MatchSignal::Publisher(normalize_vendor_name(maintainer)));
        }

        // URL/Homepage
        if let Some(url) = &details.url {
            if let Some(domain) = extract_reverse_domain_id(url) {
                signals.push(MatchSignal::KnownAppId(domain));
            }
            signals.push(MatchSignal::Homepage(normalize_homepage(url)));
        }

        signals
    }
}

pub struct FlatpakSignalExtractor;

impl SignalExtractor for FlatpakSignalExtractor {
    fn source(&self) -> PackageSource {
        PackageSource::Flatpak
    }

    fn extract(&self, details: &PackageDetails, summary: &PackageSummary) -> Vec<MatchSignal> {
        let mut signals = Vec::new();

        // Flatpak application ID is the strongest signal
        if let Some(s) = details
            .raw_metadata
            .get("application_id")
            .or_else(|| {
                details
                    .raw_metadata
                    .get("ID")
                    .or_else(|| details.raw_metadata.get("Ref"))
            })
            .and_then(|v| v.as_str())
        {
            signals.push(MatchSignal::KnownAppId(s.to_string()));
            // Also try to extract reverse domain
            if let Some(domain) = extract_reverse_domain_id(s) {
                signals.push(MatchSignal::ReverseDomainId(domain));
            }
        }

        // Product name from summary name
        signals.push(MatchSignal::ProductName(extract_product_name_from_package(
            &summary.name,
        )));

        // Source repository (origin)
        if let Some(origin) = &summary.repository {
            signals.push(MatchSignal::SourceRepository(format!("flatpak/{}", origin)));
        }

        // Publisher/developer
        if let Some(developer) = &details.maintainer {
            signals.push(MatchSignal::Publisher(normalize_vendor_name(developer)));
        }

        // URL/Homepage
        if let Some(url) = &details.url {
            if let Some(domain) = extract_reverse_domain_id(url) {
                signals.push(MatchSignal::KnownAppId(domain));
            }
            signals.push(MatchSignal::Homepage(normalize_homepage(url)));
        }

        // Desktop file ID
        if let Some(s) = details
            .raw_metadata
            .get("desktop_file")
            .and_then(|v| v.as_str())
        {
            signals.push(MatchSignal::DesktopFileId(s.to_string()));
        }

        signals
    }
}

pub fn default_extractors() -> Vec<Box<dyn SignalExtractor>> {
    vec![
        Box::new(ArchSignalExtractor),
        Box::new(AurSignalExtractor),
        Box::new(FlatpakSignalExtractor),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pkgseal_domain::PackageName;
    use pkgseal_source::dto::{PackageDetails, PackageSummary};
    use std::collections::HashMap;

    fn make_summary(name: &str, source: PackageSource) -> PackageSummary {
        PackageSummary {
            id: format!("{}/{}", source.as_str(), name),
            name: PackageName::new(name).unwrap(),
            version: "1.0".to_string(),
            description: None,
            source,
            repository: Some(source.as_str().to_string()),
            installed: false,
            download_size: None,
            installed_size: None,
        }
    }

    fn make_details() -> PackageDetails {
        PackageDetails {
            summary: make_summary("test", PackageSource::ArchOfficial),
            architecture: None,
            maintainer: Some("Brave Software Inc.".to_string()),
            url: Some("https://brave.com".to_string()),
            license: None,
            dependencies: vec![],
            optional_dependencies: vec![],
            provides: vec![],
            conflicts: vec![],
            replaces: vec![],
            groups: vec![],
            build_date: None,
            install_date: None,
            validation: None,
            raw_metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_arch_extractor_signals() {
        let mut details = make_details();
        details.summary = make_summary("brave-bin", PackageSource::ArchOfficial);
        details.summary.repository = Some("extra".to_string());
        details.maintainer = Some("Brave Software Inc.".to_string());
        details.url = Some("https://brave.com".to_string());

        let summary = details.summary.clone();
        let extractor = ArchSignalExtractor;
        let signals = extractor.extract(&details, &summary);

        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::ProductName(p) if p == "brave"))
        );
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::BinaryName(b) if b == "brave-bin"))
        );
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::Publisher(p) if p == "brave software"))
        );
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::Homepage(h) if h == "brave.com"))
        );
    }

    #[test]
    fn test_flatpak_extractor_known_app_id() {
        let mut details = make_details();
        details.summary = make_summary("org.mozilla.firefox", PackageSource::Flatpak);
        details.summary.repository = Some("flathub".to_string());
        details.raw_metadata.insert(
            "application_id".to_string(),
            serde_json::Value::String("org.mozilla.firefox".to_string()),
        );
        details.maintainer = Some("Mozilla Foundation".to_string());
        details.url = Some("https://mozilla.org".to_string());

        let summary = details.summary.clone();
        let extractor = FlatpakSignalExtractor;
        let signals = extractor.extract(&details, &summary);

        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::KnownAppId(a) if a == "org.mozilla.firefox"))
        );
        assert!(
            signals.iter().any(
                |s| matches!(s, MatchSignal::ReverseDomainId(d) if d == "org.mozilla.firefox")
            )
        );
        assert!(
            signals
                .iter()
                .any(|s| matches!(s, MatchSignal::Publisher(p) if p == "mozilla"))
        );
    }
}
