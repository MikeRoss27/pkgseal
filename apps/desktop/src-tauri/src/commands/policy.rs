use crate::dto::policy::{
    EvaluatePolicyRequest, EvaluatePolicyResponse, dto_to_candidate, map_policy_preset,
    recommendation_to_dto,
};
use crate::error::{ApiError, validation_err};
use pkgseal_policy::{Policy, evaluate};
use tauri::command;

#[command]
pub async fn evaluate_policy(
    request: EvaluatePolicyRequest,
) -> Result<EvaluatePolicyResponse, ApiError> {
    if request.candidates.is_empty() {
        return Err(validation_err("candidates cannot be empty"));
    }
    if request.candidates.len() > 64 {
        return Err(validation_err("too many candidates (max 64)"));
    }

    let preset = map_policy_preset(&request.preset);
    let policy = Policy::from_preset(preset);

    let mut candidates = Vec::with_capacity(request.candidates.len());
    for dto in &request.candidates {
        let c = dto_to_candidate(dto).map_err(validation_err)?;
        candidates.push(c);
    }

    let recommendation = evaluate(&candidates, &policy);
    log::info!(
        "policy evaluate preset={} candidates={} recommended={:?} confidence={} score={}",
        policy.preset,
        candidates.len(),
        recommendation
            .recommended
            .as_ref()
            .map(|c| c.package_name.as_str()),
        recommendation.confidence,
        recommendation.score
    );

    Ok(EvaluatePolicyResponse {
        recommendation: recommendation_to_dto(recommendation),
    })
}

#[command]
pub async fn list_policy_presets() -> Result<Vec<PresetDto>, ApiError> {
    use pkgseal_policy::PolicyPreset;
    let presets = PolicyPreset::all()
        .iter()
        .map(|p| PresetDto {
            id: p.as_str().to_string(),
            description: p.description().to_string(),
        })
        .collect();
    Ok(presets)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetDto {
    pub id: String,
    pub description: String,
}
