use pkgseal_domain::PackageSource;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

// Canonical definitions now live in pkgseal-domain; re-export for backward compatibility.
pub use pkgseal_domain::{ApplicationId, CandidateId, CandidateRef};

/// Deterministic signal used for matching candidates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum MatchSignal {
    /// Known application ID (e.g., Flatpak app ID, desktop file ID)
    KnownAppId(String),
    /// Reverse-domain identifier (e.g., com.brave.Browser)
    ReverseDomainId(String),
    /// Normalized homepage URL
    Homepage(String),
    /// Source repository (e.g., "arch/extra", "aur", "flathub")
    SourceRepository(String),
    /// Publisher/vendor name (normalized)
    Publisher(String),
    /// Package metadata hint (e.g., from .desktop file)
    DesktopFileId(String),
    /// Binary/executable name
    BinaryName(String),
    /// Normalized product name
    ProductName(String),
    /// Fuzzy name match (controlled, low confidence)
    FuzzyName(String),
}

/// Confidence level for a match
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MatchConfidence {
    Certain = 100,
    High = 80,
    Medium = 60,
    Low = 40,
    Speculative = 20,
}

impl MatchConfidence {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => MatchConfidence::Certain,
            70..=89 => MatchConfidence::High,
            50..=69 => MatchConfidence::Medium,
            30..=49 => MatchConfidence::Low,
            _ => MatchConfidence::Speculative,
        }
    }

    /// Raise confidence by one tier, capped at `Certain`. Used when multiple
    /// independent signals corroborate the same match.
    pub fn boost_one_tier(self) -> Self {
        match self {
            MatchConfidence::Speculative => MatchConfidence::Low,
            MatchConfidence::Low => MatchConfidence::Medium,
            MatchConfidence::Medium => MatchConfidence::High,
            MatchConfidence::High | MatchConfidence::Certain => MatchConfidence::Certain,
        }
    }
}

/// A match between two candidates with supporting signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateMatch {
    pub candidate_a: CandidateRef,
    pub candidate_b: CandidateRef,
    pub signals: Vec<MatchSignal>,
    pub confidence: MatchConfidence,
}

/// A resolved application identity grouping multiple candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationIdentity {
    pub id: ApplicationId,
    pub canonical_name: String,
    pub display_name: String,
    pub candidates: Vec<CandidateRef>,
    pub primary_source: Option<PackageSource>,
    pub confidence: MatchConfidence,
    pub signals: Vec<MatchSignal>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ApplicationIdentity {
    pub fn new(canonical_name: String, display_name: String) -> Self {
        Self {
            id: ApplicationId::new(),
            canonical_name,
            display_name,
            candidates: Vec::new(),
            primary_source: None,
            confidence: MatchConfidence::Speculative,
            signals: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_candidate(&mut self, candidate: CandidateRef) {
        self.candidates.push(candidate);
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    pub fn sources(&self) -> Vec<PackageSource> {
        self.candidates
            .iter()
            .map(|c| c.source)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

/// Fully resolved application with all its candidates and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedApplication {
    pub identity: ApplicationIdentity,
    pub candidate_details: Vec<pkgseal_source::dto::PackageDetails>,
}

impl ResolvedApplication {
    pub fn new(identity: ApplicationIdentity) -> Self {
        Self {
            identity,
            candidate_details: Vec::new(),
        }
    }
}
