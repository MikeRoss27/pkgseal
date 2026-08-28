use crate::dto::resolver::{ResolveRequest, ResolveResponse, ResolvedApplicationDto};
use crate::error::{ApiError, internal_err};
use crate::state::AppState;
use pkgseal_resolver::grouper::GroupingConfig;
use pkgseal_resolver::resolve_applications;
use tauri::State;
use tauri::command;

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
    let mut detail_fetches = tokio::task::JoinSet::new();
    for summary in &summaries {
        let Some(source) = state.registry.get(summary.source).cloned() else {
            continue;
        };
        let name = summary.name.clone();
        detail_fetches.spawn(async move { source.details(&name).await.ok() });
    }
    let mut all_details = Vec::new();
    while let Some(details) = detail_fetches.join_next().await {
        if let Some(details) = details.unwrap_or_default() {
            all_details.push(details);
        }
    }

    // Resolve applications
    let config = GroupingConfig::default();
    let resolved = resolve_applications(&summaries, all_details, config);

    let applications = resolved
        .into_iter()
        .map(|ra| {
            let mut dto: ResolvedApplicationDto = ra.identity.into();
            dto.candidate_details = ra.candidate_details;
            dto
        })
        .collect();

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
