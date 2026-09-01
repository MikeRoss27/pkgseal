use crate::source::PackageSource;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PackageNameError {
    #[error("package name cannot be empty")]
    Empty,
    #[error("package name must be lowercase alphanumeric with hyphens, dots, or plus")]
    InvalidCharacters,
    #[error("package name cannot start or end with hyphen, dot, or plus")]
    InvalidBoundary,
}

/// Validated package name following Arch Linux naming conventions:
/// lowercase alphanumeric + hyphen, dot, plus; cannot start/end with separator.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageName(String);

impl PackageName {
    pub fn new(s: impl AsRef<str>) -> Result<Self, PackageNameError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(PackageNameError::Empty);
        }
        if !s.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '+'
        }) {
            return Err(PackageNameError::InvalidCharacters);
        }
        if s.starts_with(['-', '.', '+']) || s.ends_with(['-', '.', '+']) {
            return Err(PackageNameError::InvalidBoundary);
        }
        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for PackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PackageName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PackageName::new(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ApplicationId(pub Uuid);

impl ApplicationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for ApplicationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CandidateId(pub Uuid);

impl CandidateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CandidateId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Reference to a package candidate from a specific source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CandidateRef {
    pub candidate_id: CandidateId,
    pub source: PackageSource,
    pub package_name: PackageName,
    pub package_id: String,
}

impl CandidateRef {
    pub fn new(source: PackageSource, package_name: PackageName, package_id: String) -> Self {
        Self {
            candidate_id: CandidateId::new(),
            source,
            package_name,
            package_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() {
        assert!(PackageName::new("brave-bin").is_ok());
        assert!(PackageName::new("vscode").is_ok());
        assert!(PackageName::new("lib32-vulkan-icd-loader").is_ok());
        assert!(PackageName::new("pkg-config+1.2.3").is_ok());
        assert!(PackageName::new("foo.bar").is_ok());
    }

    #[test]
    fn invalid_names() {
        assert!(PackageName::new("").is_err());
        assert!(PackageName::new("Brave").is_err());
        assert!(PackageName::new("brave_bin").is_err());
        assert!(PackageName::new("brave@bin").is_err());
        assert!(PackageName::new("-brave").is_err());
        assert!(PackageName::new("brave-").is_err());
        assert!(PackageName::new(".brave").is_err());
        assert!(PackageName::new("brave.").is_err());
        assert!(PackageName::new("+brave").is_err());
        assert!(PackageName::new("brave+").is_err());
    }

    #[test]
    fn serialization_roundtrip() {
        let name = PackageName::new("test-package").unwrap();
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"test-package\"");
        let parsed: PackageName = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, name);
    }
}
