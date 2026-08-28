use crate::dto::app::{AppHealthDto, SourceAvailabilityDto};
use crate::error::ApiError;
use crate::state::AppState;
use pkgseal_domain::PackageSource;
use tauri::State;
use tauri::command;

#[command]
pub async fn app_health() -> Result<AppHealthDto, ApiError> {
    let sources = PackageSource::all();
    Ok(AppHealthDto::from(&sources[..]))
}

/// Reports whether each package source's underlying tool (`pacman`, network
/// access to the AUR, `flatpak`) is actually usable on this machine, so the
/// UI can show a source as unavailable instead of silently returning nothing.
#[command]
pub async fn source_availability(
    state: State<'_, AppState>,
) -> Result<Vec<SourceAvailabilityDto>, ApiError> {
    let mut results = Vec::new();
    for source in state.registry.sources() {
        results.push(SourceAvailabilityDto {
            source: source.source().to_string(),
            available: source.is_available().await,
        });
    }
    Ok(results)
}
