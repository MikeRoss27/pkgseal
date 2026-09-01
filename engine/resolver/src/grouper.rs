use crate::identity::{
    ApplicationIdentity, CandidateRef, MatchConfidence, MatchSignal, ResolvedApplication,
};
use crate::signal::{default_extractors, extract_signals};
use indexmap::IndexMap;
use pkgseal_domain::PackageSource;
use pkgseal_source::dto::{PackageDetails, PackageSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingConfig {
    pub min_confidence_for_merge: MatchConfidence,
    pub require_at_least_one_strong_signal: bool,
    pub fuzzy_threshold: f64,
}

impl Default for GroupingConfig {
    fn default() -> Self {
        Self {
            min_confidence_for_merge: MatchConfidence::Medium,
            require_at_least_one_strong_signal: true,
            fuzzy_threshold: 0.8,
        }
    }
}

/// Result of grouping candidates
#[derive(Debug, Clone)]
pub struct GroupingResult {
    pub applications: Vec<ApplicationIdentity>,
    pub unmatched: Vec<CandidateRef>,
}

fn signal_priority(signal: &MatchSignal) -> u8 {
    match signal {
        MatchSignal::KnownAppId(_) => 50,
        MatchSignal::ReverseDomainId(_) => 40,
        // Homepage was 30 (Certain) — downgraded to 20 (High) to avoid
        // github.com-like collisions granting Certain confidence.
        MatchSignal::Homepage(_) => 20,
        MatchSignal::SourceRepository(_) => 20,
        MatchSignal::Publisher(_) => 15,
        MatchSignal::DesktopFileId(_) => 10,
        MatchSignal::BinaryName(_) => 8,
        MatchSignal::ProductName(_) => 5,
        MatchSignal::FuzzyName(_) => 2,
    }
}

fn is_strong_signal(signal: &MatchSignal) -> bool {
    matches!(
        signal,
        MatchSignal::KnownAppId(_) | MatchSignal::ReverseDomainId(_)
    )
}

/// Word-level containment check for normalized product names.
///
/// Sources name the same application at different granularity (e.g. Arch's
/// "brave" vs. Flatpak's "brave browser"). If every word of the shorter name
/// appears in the longer one, treat it as a product name match.
///
/// Guard against short-word false positives (e.g. "code" ⊆ "visual studio code"):
/// when the smaller name is a single short word (≤4 chars) require strict
/// equality instead of containment.
fn product_names_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let words_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b.split_whitespace().collect();
    if words_a.is_empty() || words_b.is_empty() {
        return false;
    }
    let (small, big) = if words_a.len() <= words_b.len() {
        (&words_a, &words_b)
    } else {
        (&words_b, &words_a)
    };
    // Avoid "code" (single 4-char word) matching "visual studio code" via containment.
    if small.len() == 1 && small.iter().next().is_some_and(|w| w.len() <= 4) {
        return false;
    }
    small.is_subset(big)
}

fn signal_match(a: &MatchSignal, b: &MatchSignal) -> bool {
    match (a, b) {
        (MatchSignal::KnownAppId(a), MatchSignal::KnownAppId(b)) => a == b,
        (MatchSignal::ReverseDomainId(a), MatchSignal::ReverseDomainId(b)) => a == b,
        (MatchSignal::Homepage(a), MatchSignal::Homepage(b)) => a == b,
        (MatchSignal::SourceRepository(a), MatchSignal::SourceRepository(b)) => a == b,
        (MatchSignal::Publisher(a), MatchSignal::Publisher(b)) => a == b,
        (MatchSignal::DesktopFileId(a), MatchSignal::DesktopFileId(b)) => a == b,
        (MatchSignal::BinaryName(a), MatchSignal::BinaryName(b)) => a == b,
        (MatchSignal::ProductName(a), MatchSignal::ProductName(b)) => product_names_match(a, b),
        (MatchSignal::FuzzyName(a), MatchSignal::FuzzyName(b)) => a == b,
        _ => false,
    }
}

/// Confidence tier implied by the single strongest signal type that matched.
/// Corroboration from additional matching signals is applied afterward.
fn confidence_tier_for_priority(priority: u8) -> MatchConfidence {
    match priority {
        40..=u8::MAX => MatchConfidence::Certain, // KnownAppId (50), ReverseDomainId (40)
        20..=39 => MatchConfidence::High,         // Homepage (20), SourceRepository (20)
        5..=19 => MatchConfidence::Medium, // Publisher (15), DesktopFileId (10), BinaryName (8), ProductName (5)
        _ => MatchConfidence::Speculative, // FuzzyName (2)
    }
}

/// Outcome of comparing two candidates' signal sets.
struct MatchOutcome {
    confidence: MatchConfidence,
    /// Whether a strong-identity signal type (KnownAppId/ReverseDomainId)
    /// was itself part of the match, as opposed to only weaker corroborating signals.
    matched_strong: bool,
}

fn compute_match_confidence(
    signals_a: &[MatchSignal],
    signals_b: &[MatchSignal],
    fuzzy_threshold: f64,
) -> MatchOutcome {
    let mut match_count = 0u32;
    let mut max_priority = 0u8;
    let mut matched_strong = false;

    for sa in signals_a {
        for sb in signals_b {
            if signal_match(sa, sb) {
                match_count += 1;
                let p = signal_priority(sa);
                if p > max_priority {
                    max_priority = p;
                }
                if is_strong_signal(sa) {
                    matched_strong = true;
                }
            } else if let (MatchSignal::ProductName(pa), MatchSignal::ProductName(pb)) = (sa, sb) {
                // Controlled fuzzy fallback: the lowest-priority signal (typo-tolerant
                // name comparison). Only considered when the pair didn't already match
                // exactly/by containment above, and it can never out-rank a real signal.
                if fuzzy_match(pa, pb, fuzzy_threshold) {
                    match_count += 1;
                    max_priority =
                        max_priority.max(signal_priority(&MatchSignal::FuzzyName(String::new())));
                }
            }
        }
    }

    if match_count == 0 {
        return MatchOutcome {
            confidence: MatchConfidence::Speculative,
            matched_strong: false,
        };
    }

    let base = confidence_tier_for_priority(max_priority);
    let confidence = if match_count >= 2 {
        base.boost_one_tier()
    } else {
        base
    };

    MatchOutcome {
        confidence,
        matched_strong,
    }
}

fn fuzzy_match(a: &str, b: &str, threshold: f64) -> bool {
    if a == b {
        return true;
    }
    let len_a = a.len();
    let len_b = b.len();
    if len_a == 0 || len_b == 0 {
        return false;
    }
    let max_len = len_a.max(len_b) as f64;
    let dist = levenshtein_distance(a, b) as f64;
    let similarity = 1.0 - (dist / max_len);
    similarity >= threshold
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut prev: Vec<usize> = (0..=len_b).collect();
    let mut curr = vec![0; len_b + 1];

    for i in 1..=len_a {
        curr[0] = i;
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = std::cmp::min(
                std::cmp::min(prev[j] + 1, curr[j - 1] + 1),
                prev[j - 1] + cost,
            );
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[len_b]
}

pub fn group_candidates(
    summaries: &[PackageSummary],
    details_map: &IndexMap<String, PackageDetails>,
    config: GroupingConfig,
) -> GroupingResult {
    if summaries.is_empty() {
        return GroupingResult {
            applications: Vec::new(),
            unmatched: Vec::new(),
        };
    }

    let extractors = default_extractors();

    // Extract signals for each candidate
    let mut candidate_signals: IndexMap<String, Vec<MatchSignal>> = IndexMap::new();
    let mut candidate_refs: IndexMap<String, CandidateRef> = IndexMap::new();

    for summary in summaries {
        let key = summary.id.clone();
        let details = details_map.get(&key);
        let signals = if let Some(details) = details {
            extract_signals(&extractors, details, summary)
        } else {
            Vec::new()
        };
        candidate_signals.insert(key.clone(), signals);

        let candidate_ref =
            CandidateRef::new(summary.source, summary.name.clone(), summary.id.clone());
        candidate_refs.insert(key, candidate_ref);
    }

    // Group candidates by comparing signals
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut assigned: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (key_a, _) in &candidate_signals {
        if assigned.contains(key_a) {
            continue;
        }

        let mut group = vec![key_a.clone()];
        assigned.insert(key_a.clone());

        let signals_a = candidate_signals.get(key_a).cloned().unwrap_or_default();

        for (key_b, _) in &candidate_signals {
            if assigned.contains(key_b) || key_a == key_b {
                continue;
            }

            let signals_b = candidate_signals.get(key_b).cloned().unwrap_or_default();
            let outcome = compute_match_confidence(&signals_a, &signals_b, config.fuzzy_threshold);

            if outcome.confidence >= config.min_confidence_for_merge {
                // A strong-identity signal (KnownAppId/ReverseDomainId) directly
                // participating in the match always satisfies the requirement. Otherwise,
                // corroboration from multiple weaker signals reaching at least `Medium`
                // confidence is accepted as sufficient evidence on its own.
                let strength_ok = !config.require_at_least_one_strong_signal
                    || outcome.matched_strong
                    || outcome.confidence >= MatchConfidence::Medium;

                if strength_ok {
                    group.push(key_b.clone());
                    assigned.insert(key_b.clone());
                }
            }
        }

        groups.push(group);
    }

    // Build ApplicationIdentity from groups
    let mut applications = Vec::new();
    let mut unmatched = Vec::new();

    for group in groups {
        if group.len() == 1 {
            let key = &group[0];
            if let Some(candidate_ref) = candidate_refs.get(key) {
                unmatched.push(candidate_ref.clone());
            }
        } else {
            let identity =
                build_identity_from_group(&group, &candidate_signals, &candidate_refs, &config);
            applications.push(identity);
        }
    }

    GroupingResult {
        applications,
        unmatched,
    }
}

fn build_identity_from_group(
    group: &[String],
    candidate_signals: &IndexMap<String, Vec<MatchSignal>>,
    candidate_refs: &IndexMap<String, CandidateRef>,
    config: &GroupingConfig,
) -> ApplicationIdentity {
    let mut all_signals = Vec::new();
    let mut candidates = Vec::new();
    let mut product_names = Vec::new();
    let mut sources: Vec<PackageSource> = Vec::new();

    for key in group {
        if let Some(signals) = candidate_signals.get(key) {
            all_signals.extend(signals.iter().cloned());
        }
        if let Some(candidate_ref) = candidate_refs.get(key) {
            candidates.push(candidate_ref.clone());
            product_names.push(candidate_ref.package_name.as_str().to_string());
            sources.push(candidate_ref.source);
        }
    }

    // Deduplicate signals
    all_signals.sort();
    all_signals.dedup();

    // Determine canonical name (prefer longest product name, then known app ID)
    let canonical_name = product_names
        .iter()
        .max_by_key(|s| s.len())
        .cloned()
        .unwrap_or_else(|| candidates[0].package_name.as_str().to_string());

    // Find known app ID for display name
    let display_name = all_signals
        .iter()
        .find_map(|s| {
            if let MatchSignal::KnownAppId(id) = s {
                Some(id.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| canonical_name.clone());

    // Policy decides recommendation; resolver does not rank sources. TODO: delegate to engine/policy
    // No hardcoded Arch > Flatpak > AUR ordering — keep first source as placeholder without preference.
    let primary_source = sources.into_iter().next();

    // Overall confidence is the minimum pairwise confidence in the group
    let mut overall_confidence = MatchConfidence::Certain;
    for i in 0..group.len() {
        for j in i + 1..group.len() {
            let signals_a = candidate_signals
                .get(&group[i])
                .cloned()
                .unwrap_or_default();
            let signals_b = candidate_signals
                .get(&group[j])
                .cloned()
                .unwrap_or_default();
            let outcome = compute_match_confidence(&signals_a, &signals_b, config.fuzzy_threshold);
            if outcome.confidence < overall_confidence {
                overall_confidence = outcome.confidence;
            }
        }
    }

    let mut identity = ApplicationIdentity::new(canonical_name, display_name);
    identity.candidates = candidates;
    identity.primary_source = primary_source;
    identity.confidence = overall_confidence.max(config.min_confidence_for_merge);
    identity.signals = all_signals;

    identity
}

/// Resolve all candidates into applications with full details
pub fn resolve_applications(
    summaries: &[PackageSummary],
    details: Vec<PackageDetails>,
    config: GroupingConfig,
) -> Vec<ResolvedApplication> {
    let mut details_map = IndexMap::new();
    for d in details {
        details_map.insert(d.summary.id.clone(), d);
    }

    let result = group_candidates(summaries, &details_map, config);

    let mut resolved = Vec::new();
    for identity in result.applications {
        let mut resolved_app = ResolvedApplication::new(identity);
        // Find matching details for each candidate
        for candidate in &resolved_app.identity.candidates {
            if let Some(details) = details_map.get(&candidate.package_id) {
                resolved_app.candidate_details.push(details.clone());
            }
        }
        resolved.push(resolved_app);
    }

    // Add unmatched as singleton resolved applications
    for candidate in result.unmatched {
        let mut identity = ApplicationIdentity::new(
            candidate.package_name.as_str().to_string(),
            candidate.package_name.as_str().to_string(),
        );
        identity.add_candidate(candidate.clone());
        identity.primary_source = Some(candidate.source);
        identity.confidence = MatchConfidence::Speculative;

        let mut resolved_app = ResolvedApplication::new(identity);
        if let Some(details) = details_map.get(&candidate.package_id) {
            resolved_app.candidate_details.push(details.clone());
        }
        resolved.push(resolved_app);
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
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

    fn make_details(name: &str, source: PackageSource) -> PackageDetails {
        let summary = make_summary(name, source);
        let mut raw_metadata = HashMap::new();
        if source == PackageSource::Flatpak {
            raw_metadata.insert(
                "application_id".to_string(),
                serde_json::Value::String(format!("org.example.{}", name)),
            );
        }
        // Add common metadata for brave to enable grouping
        let (maintainer, url) = if name.contains("brave") {
            (
                Some("Brave Software Inc.".to_string()),
                Some("https://brave.com".to_string()),
            )
        } else if name.contains("firefox") {
            (
                Some("Mozilla Foundation".to_string()),
                Some("https://mozilla.org".to_string()),
            )
        } else {
            (None, None)
        };
        // Add KnownAppId for brave packages to enable cross-source matching
        if name.contains("brave") {
            raw_metadata.insert(
                "application_id".to_string(),
                serde_json::Value::String("org.example.brave".to_string()),
            );
        }
        PackageDetails {
            summary,
            architecture: None,
            maintainer,
            url,
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
            raw_metadata,
        }
    }

    #[test]
    fn test_group_identical_names_different_sources() {
        let summaries = vec![
            make_summary("brave-bin", PackageSource::ArchOfficial),
            make_summary("brave-bin", PackageSource::Aur),
            make_summary("brave-browser", PackageSource::Flatpak),
        ];

        let mut details_map = IndexMap::new();
        for s in &summaries {
            details_map.insert(s.id.clone(), make_details(s.name.as_str(), s.source));
        }

        let config = GroupingConfig::default();
        let result = group_candidates(&summaries, &details_map, config);

        // Should group into 1 application
        assert_eq!(result.applications.len(), 1);
        assert_eq!(result.unmatched.len(), 0);

        let app = &result.applications[0];
        assert_eq!(app.candidates.len(), 3);
        assert!(app.confidence >= MatchConfidence::Medium);
    }

    #[test]
    fn test_unmatched_singletons() {
        let summaries = vec![
            make_summary("totally-unique-package", PackageSource::ArchOfficial),
            make_summary("another-unique", PackageSource::Aur),
        ];

        let mut details_map = IndexMap::new();
        for s in &summaries {
            details_map.insert(s.id.clone(), make_details(s.name.as_str(), s.source));
        }

        let config = GroupingConfig::default();
        let result = group_candidates(&summaries, &details_map, config);

        // Each should be unmatched (singleton)
        assert_eq!(result.applications.len(), 0);
        assert_eq!(result.unmatched.len(), 2);
    }

    #[test]
    fn test_flatpak_known_app_id_match() {
        let summaries = vec![
            make_summary("firefox", PackageSource::ArchOfficial),
            make_summary("firefox", PackageSource::Flatpak),
        ];

        let mut details_map = IndexMap::new();
        for s in &summaries {
            let mut details = make_details(s.name.as_str(), s.source);
            if s.source == PackageSource::Flatpak {
                details.raw_metadata.insert(
                    "application_id".to_string(),
                    serde_json::Value::String("org.mozilla.firefox".to_string()),
                );
            }
            details_map.insert(s.id.clone(), details);
        }

        let config = GroupingConfig::default();
        let result = group_candidates(&summaries, &details_map, config);

        assert_eq!(result.applications.len(), 1);
        assert_eq!(result.applications[0].candidates.len(), 2);
    }

    #[test]
    fn test_compute_match_confidence() {
        let signals_a = vec![
            MatchSignal::KnownAppId("com.brave.browser".to_string()),
            MatchSignal::ProductName("brave".to_string()),
        ];
        let signals_b = vec![
            MatchSignal::KnownAppId("com.brave.browser".to_string()),
            MatchSignal::ProductName("brave".to_string()),
        ];
        let outcome = compute_match_confidence(&signals_a, &signals_b, 0.8);
        assert_eq!(outcome.confidence, MatchConfidence::Certain);

        let signals_c = vec![
            MatchSignal::ProductName("brave".to_string()),
            MatchSignal::BinaryName("brave-bin".to_string()),
        ];
        let signals_d = vec![
            MatchSignal::ProductName("brave".to_string()),
            MatchSignal::BinaryName("brave-bin".to_string()),
        ];
        let outcome2 = compute_match_confidence(&signals_c, &signals_d, 0.8);
        assert!(outcome2.confidence >= MatchConfidence::High);
    }

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("brave", "brave", 0.8));
        assert!(fuzzy_match("brave-browser", "brave browser", 0.8));
        assert!(!fuzzy_match("brave", "firefox", 0.8));
        assert!(!fuzzy_match("", "test", 0.8));
    }
}
