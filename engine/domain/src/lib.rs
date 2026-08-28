pub mod error;
pub mod ids;
pub mod source;

pub use error::DomainError;
pub use ids::{ApplicationId, CandidateId, CandidateRef, PackageName};
pub use source::PackageSource;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_valid() {
        let name = PackageName::new("brave-bin").unwrap();
        assert_eq!(name.as_str(), "brave-bin");
    }

    #[test]
    fn package_name_rejects_empty() {
        assert!(PackageName::new("").is_err());
    }

    #[test]
    fn package_name_rejects_uppercase() {
        assert!(PackageName::new("Brave").is_err());
    }

    #[test]
    fn package_name_rejects_invalid_chars() {
        assert!(PackageName::new("brave@bin").is_err());
        assert!(PackageName::new("brave bin").is_err());
    }

    #[test]
    fn package_source_serialization() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(PackageSource::ArchOfficial).unwrap(),
            json!("arch-official")
        );
        assert_eq!(
            serde_json::to_value(PackageSource::Aur).unwrap(),
            json!("aur")
        );
        assert_eq!(
            serde_json::to_value(PackageSource::Flatpak).unwrap(),
            json!("flatpak")
        );
    }

    #[test]
    fn package_source_deserialization() {
        use serde_json::from_str;
        assert_eq!(
            from_str::<PackageSource>("\"arch-official\"").unwrap(),
            PackageSource::ArchOfficial
        );
        assert_eq!(
            from_str::<PackageSource>("\"aur\"").unwrap(),
            PackageSource::Aur
        );
        assert_eq!(
            from_str::<PackageSource>("\"flatpak\"").unwrap(),
            PackageSource::Flatpak
        );
    }
}
