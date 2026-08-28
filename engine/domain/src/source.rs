use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageSource {
    ArchOfficial,
    Aur,
    Flatpak,
}

impl PackageSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageSource::ArchOfficial => "arch-official",
            PackageSource::Aur => "aur",
            PackageSource::Flatpak => "flatpak",
        }
    }

    pub fn all() -> [PackageSource; 3] {
        [
            PackageSource::ArchOfficial,
            PackageSource::Aur,
            PackageSource::Flatpak,
        ]
    }
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sources_covered() {
        let sources = PackageSource::all();
        assert_eq!(sources.len(), 3);
    }
}
