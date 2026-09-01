use crate::dto::policy::RecommendationDto;
use pkgseal_resolver::identity::{ApplicationIdentity, CandidateRef, MatchSignal};
use pkgseal_source::dto::PackageDetails;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRequest {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveResponse {
    pub applications: Vec<ResolvedApplicationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedApplicationDto {
    pub id: String,
    pub canonical_name: String,
    pub display_name: String,
    pub candidates: Vec<CandidateRefDto>,
    pub primary_source: Option<String>,
    pub confidence: String,
    pub signals: Vec<SignalDto>,
    pub candidate_details: Vec<PackageDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<RecommendationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateRefDto {
    pub candidate_id: String,
    pub source: String,
    pub package_name: String,
    pub package_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalDto {
    pub signal_type: String,
    pub value: String,
}

impl From<ApplicationIdentity> for ResolvedApplicationDto {
    fn from(identity: ApplicationIdentity) -> Self {
        Self {
            id: identity.id.to_string(),
            canonical_name: identity.canonical_name,
            display_name: identity.display_name,
            candidates: identity.candidates.into_iter().map(|c| c.into()).collect(),
            primary_source: identity.primary_source.map(|s| s.to_string()),
            confidence: format!("{:?}", identity.confidence),
            signals: identity.signals.into_iter().map(|s| s.into()).collect(),
            candidate_details: Vec::new(), // Will be filled by caller
            recommendation: None,
        }
    }
}

impl From<CandidateRef> for CandidateRefDto {
    fn from(c: CandidateRef) -> Self {
        Self {
            candidate_id: c.candidate_id.to_string(),
            source: c.source.to_string(),
            package_name: c.package_name.as_str().to_string(),
            package_id: c.package_id,
        }
    }
}

impl From<MatchSignal> for SignalDto {
    fn from(s: MatchSignal) -> Self {
        let (signal_type, value) = match s {
            MatchSignal::KnownAppId(v) => ("KnownAppId", v),
            MatchSignal::ReverseDomainId(v) => ("ReverseDomainId", v),
            MatchSignal::Homepage(v) => ("Homepage", v),
            MatchSignal::SourceRepository(v) => ("SourceRepository", v),
            MatchSignal::Publisher(v) => ("Publisher", v),
            MatchSignal::DesktopFileId(v) => ("DesktopFileId", v),
            MatchSignal::BinaryName(v) => ("BinaryName", v),
            MatchSignal::ProductName(v) => ("ProductName", v),
            MatchSignal::FuzzyName(v) => ("FuzzyName", v),
        };
        Self {
            signal_type: signal_type.to_string(),
            value,
        }
    }
}
