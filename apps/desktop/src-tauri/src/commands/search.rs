use crate::dto::search::{
    DetailsRequest, DetailsResponse, InstalledRequest, InstalledResponse, SearchRequest,
    SearchResponse,
};
use crate::error::{ApiError, internal_err, validation_err};
use crate::state::AppState;
use pkgseal_domain::PackageName;
use pkgseal_source::dto::SearchQuery;
use tauri::State;
use tauri::command;

#[command]
pub async fn search_packages(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<SearchResponse, ApiError> {
    let query = SearchQuery::new(request.query);
    let results = state
        .registry
        .search_all(&query)
        .await
        .map_err(internal_err)?;
    Ok(SearchResponse { packages: results })
}

#[command]
pub async fn get_package_details(
    state: State<'_, AppState>,
    request: DetailsRequest,
) -> Result<DetailsResponse, ApiError> {
    let name = PackageName::new(&request.name).map_err(validation_err)?;
    let details = state
        .registry
        .details_all(&name)
        .await
        .map_err(internal_err)?;
    Ok(DetailsResponse { details })
}

#[command]
pub async fn get_installed_packages(
    state: State<'_, AppState>,
    _request: InstalledRequest,
) -> Result<InstalledResponse, ApiError> {
    let installed = state.registry.installed_all().await.map_err(internal_err)?;
    Ok(InstalledResponse {
        packages: installed,
    })
}
