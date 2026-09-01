use pkgseal_domain::{PackageName, PackageSource};
use pkgseal_transactions::{TransactionMetadata, TransactionOperation, TransactionPlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTransactionRequest {
    pub source: String,
    pub package_name: String,
    pub version: String,
    /// For Flatpak: reverse-DNS app id (e.g. com.brave.Browser). If None, derived from package_name.
    pub app_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTransactionResponse {
    pub plan: TransactionPlanDto,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPlanDto {
    pub id: String,
    pub source: String,
    pub package_name: String,
    pub package_version: String,
    pub privileges_required: bool,
    pub expected_download_size: Option<u64>,
    pub expected_disk_change: Option<i64>,
    pub operations: Vec<OperationDto>,
    pub state: String,
    pub created_at: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationDto {
    pub kind: String,
    pub summary: String,
    pub requires_privileges: bool,
}

pub fn parse_source(s: &str) -> Result<PackageSource, String> {
    match s.to_ascii_lowercase().as_str() {
        "arch" | "arch-official" | "arch_official" => Ok(PackageSource::ArchOfficial),
        "aur" => Ok(PackageSource::Aur),
        "flatpak" => Ok(PackageSource::Flatpak),
        other => Err(format!(
            "unknown source: {other} (expected arch/aur/flatpak)"
        )),
    }
}

pub fn build_plan(req: &PreviewTransactionRequest) -> Result<TransactionPlan, String> {
    let source = parse_source(&req.source)?;
    let name =
        PackageName::new(&req.package_name).map_err(|e| format!("invalid package_name: {e}"))?;
    let version = req.version.trim();
    if version.is_empty() {
        return Err("version cannot be empty".to_string());
    }

    let operations = match source {
        PackageSource::Flatpak => {
            let app_id = req
                .app_id
                .clone()
                .unwrap_or_else(|| req.package_name.clone());
            // Validate app_id shape for flatpak (must contain a dot for reverse-DNS)
            if !app_id.contains('.') {
                return Err(format!(
                    "flatpak app_id must be reverse-DNS (e.g. com.example.App), got: {app_id}"
                ));
            }
            vec![TransactionOperation::InstallFlatpak {
                app_id,
                version: Some(version.to_string()),
            }]
        }
        PackageSource::ArchOfficial | PackageSource::Aur => {
            vec![TransactionOperation::InstallPackage {
                name: name.clone(),
                version: version.to_string(),
            }]
        }
    };

    let privileges_required = !matches!(source, PackageSource::Flatpak);

    let mut metadata = TransactionMetadata::new();
    if let Some(reason) = &req.reason {
        metadata = metadata.with_reason(reason);
    }
    metadata = metadata.with_extra("source", source.to_string());

    let plan = TransactionPlan::new(source, name, version, operations, privileges_required)
        .map_err(|e| e.to_string())?
        .with_metadata(metadata);
    Ok(plan)
}

pub fn plan_to_dto(plan: &TransactionPlan) -> TransactionPlanDto {
    TransactionPlanDto {
        id: plan.id.to_string(),
        source: plan.source.to_string(),
        package_name: plan.package_name.as_str().to_string(),
        package_version: plan.package_version.clone(),
        privileges_required: plan.privileges_required,
        expected_download_size: plan.expected_download_size,
        expected_disk_change: plan.expected_disk_change,
        operations: plan
            .operations
            .iter()
            .map(|op| OperationDto {
                kind: format!("{:?}", op).to_ascii_lowercase(),
                summary: op.summary(),
                requires_privileges: op.requires_privileges(),
            })
            .collect(),
        state: format!("{:?}", plan.state).to_ascii_lowercase(),
        created_at: plan.created_at.to_string(),
        summary: plan.summary(),
    }
}
