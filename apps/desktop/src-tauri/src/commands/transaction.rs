use crate::dto::transaction::{
    PreviewTransactionRequest, PreviewTransactionResponse, build_plan, plan_to_dto,
};
use crate::error::{ApiError, validation_err};
use tauri::command;

#[command]
pub async fn preview_transaction(
    request: PreviewTransactionRequest,
) -> Result<PreviewTransactionResponse, ApiError> {
    let plan = build_plan(&request).map_err(validation_err)?;
    log::info!("preview_transaction {}", plan.summary());
    let preview = plan.preview();
    let dto = plan_to_dto(&plan);
    Ok(PreviewTransactionResponse { plan: dto, preview })
}

#[command]
pub async fn validate_transaction_request(
    request: PreviewTransactionRequest,
) -> Result<ValidationDto, ApiError> {
    match build_plan(&request) {
        Ok(plan) => Ok(ValidationDto {
            valid: true,
            message: format!("valid plan: {}", plan.summary()),
            privileges_required: plan.privileges_required,
        }),
        Err(e) => Ok(ValidationDto {
            valid: false,
            message: e,
            privileges_required: false,
        }),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationDto {
    pub valid: bool,
    pub message: String,
    pub privileges_required: bool,
}
