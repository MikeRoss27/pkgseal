use crate::dto::policy::{build_candidate_evidence, recommendation_to_dto};
use crate::dto::resolver::{ResolveRequest, ResolveResponse, ResolvedApplicationDto};
use crate::error::{ApiError, internal_err};
use crate::state::AppState;
use pkgseal_domain::PackageSource;
use pkgseal_policy::{Confidence, Policy, evaluate, PolicyCandidate};
use pkgseal_resolver::grouper::GroupingConfig;
use pkgseal_resolver::resolve_applications;
use pkgseal_source::dto::PackageDetails;
use std::sync::Arc;
use tauri::State;
use tauri::command;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Minimum query length before we hit external sources. Shorter queries are
/// too broad (and some sources, like the AUR RPC, reject them outright).
const MIN_QUERY_LEN: usize = 2;

#[command]
pub async fn resolve_applications_command(
    state: State<'_, AppState>,
    request: ResolveRequest,
) -> Result<ResolveResponse, ApiError> {
    let trimmed = request.query.trim();
    if trimmed.chars().count() < MIN_QUERY_LEN {
        return Ok(ResolveResponse {
            applications: Vec::new(),
        });
    }

    // Search all sources for the requested application
    let query = pkgseal_source::dto::SearchQuery::new(trimmed);
    let summaries = state
        .registry
        .search_all(&query)
        .await
        .map_err(internal_err)?;

    if summaries.is_empty() {
        return Ok(ResolveResponse {
            applications: Vec::new(),
        });
    }

    // Get details for all candidates concurrently — sources hit a subprocess
    // or the network per candidate, so fetching sequentially here would make
    // search latency scale with result count instead of the slowest source.
    // Bounded concurrency: at most 8 concurrent `pacman -Si` / HTTP fetches.
    // Without this, N=50 summaries => 50 concurrent pacman processes =>
    // db.lck contention + DoS. Each task acquires a permit at the start of
    // the async block, limiting actual concurrency while still spawning all
    // tasks into the JoinSet (no blocking in the spawn loop).
    // Source-level timeout (10s) is handled inside each adapter; no additional
    // global timeout is added here.
    // TODO: add request deduplication/cancellation if the user retypes a query
    // while a previous resolve is in-flight (e.g. abort previous JoinSet via
    // cancellation token or singleflight).
    let semaphore = Arc::new(Semaphore::new(8));
    let mut detail_fetches = tokio::task::JoinSet::new();
    for summary in &summaries {
        let Some(source) = state.registry.get(summary.source).cloned() else {
            continue;
        };
        let name = summary.name.clone();
        let source_name = summary.source;
        let sem = semaphore.clone();
        detail_fetches.spawn(async move {
            // Acquire a permit for the duration of the details fetch. Using
            // `acquire_owned` with an `Arc<Semaphore>` ensures the permit is
            // `'static` and does not borrow the local `sem` variable.
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    log::warn!(
                        "semaphore closed, aborting details fetch for {}: {}",
                        name,
                        e
                    );
                    return Err(pkgseal_source::error::SourceError::unavailable(format!(
                        "semaphore closed: {e}"
                    )));
                }
            };
            let result = source.details(&name).await;
            if let Err(e) = &result {
                log::warn!(
                    "failed to fetch details for {} from {}: {}",
                    name,
                    source_name,
                    e
                );
            }
            result
        });
    }
    let mut all_details = Vec::new();
    while let Some(result) = detail_fetches.join_next().await {
        match result {
            Ok(Ok(details)) => all_details.push(details),
            Ok(Err(e)) => log::warn!("detail fetch returned error: {}", e),
            Err(join_err) => log::warn!("detail fetch task panicked: {}", join_err),
        }
    }

    // Resolve applications
    let config = GroupingConfig::default();
    let resolved = resolve_applications(&summaries, all_details, config);

    let mut applications = Vec::with_capacity(resolved.len());

    for ra in resolved {
        // Build a map of package_id -> PackageDetails for quick lookup
        let details_by_id: std::collections::HashMap<String, &PackageDetails> = ra
            .candidate_details
            .iter()
            .map(|d| (d.summary.id.clone(), d))
            .collect();

        // Build PolicyCandidate for each candidate in this application identity
        let mut policy_candidates = Vec::with_capacity(ra.identity.candidates.len());
        for candidate_ref in &ra.identity.candidates {
            if let Some(details) = details_by_id.get(&candidate_ref.package_id) {
                let source = match candidate_ref.source.as_str().to_ascii_lowercase().as_str() {
                    "arch" | "arch-official" | "arch_official" => PackageSource::ArchOfficial,
                    "flatpak" | "flathub" => PackageSource::Flatpak,
                    _ => PackageSource::Aur,
                };

                let evidence = build_candidate_evidence(source, details);

                let package_name = pkgseal_domain::PackageName::new(candidate_ref.package_name.as_str())
                    .map_err(|e| {
                        log::warn!(
                            "invalid package name {}: {}",
                            candidate_ref.package_name,
                            e
                        );
                        internal_err("invalid package name")
                    })?;

                let policy_candidate = PolicyCandidate::new(
                    source,
                    package_name,
                    details.summary.version.clone(),
                    evidence,
                ).with_id(pkgseal_domain::CandidateId(
                    Uuid::parse_str(&candidate_ref.candidate_id.to_string())
                        .unwrap_or_else(|_| Uuid::new_v4()),
                ));

                policy_candidates.push(policy_candidate);
            }
        }

        // Evaluate policy with Balanced preset as default
        let recommendation = if policy_candidates.is_empty() {
            pkgseal_policy::Recommendation::none(Confidence::None)
        } else {
            let policy = Policy::balanced();
            evaluate(&policy_candidates, &policy)
        };

        let mut dto: ResolvedApplicationDto = ra.identity.into();
        dto.candidate_details = ra.candidate_details;
        dto.recommendation = Some(recommendation_to_dto(recommendation));
        applications.push(dto);
    }

    Ok(ResolveResponse { applications })
}

#[command]
pub async fn get_resolver_config(
    _state: State<'_, AppState>,
) -> Result<ResolverConfigDto, ApiError> {
    let config = GroupingConfig::default();
    Ok(ResolverConfigDto {
        min_confidence_for_merge: format!("{:?}", config.min_confidence_for_merge),
        require_at_least_one_strong_signal: config.require_at_least_one_strong_signal,
        fuzzy_threshold: config.fuzzy_threshold,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverConfigDto {
    pub min_confidence_for_merge: String,
    pub require_at_least_one_strong_signal: bool,
    pub fuzzy_threshold: f64,
}
